//! PeeringDB OpenID Connect login and the request guards built on it.
//!
//! PeeringDB is both the identity provider and the authorisation source: the
//! `networks` scope tells us which ASNs a user may act for, which is exactly
//! the question this site needs answered. Nobody has to be provisioned here.

use chrono::{Duration as ChronoDuration, Utc};
use oauth2::basic::BasicClient;
use oauth2::{
    AuthUrl, ClientId, ClientSecret, CsrfToken, EndpointNotSet, EndpointSet, RedirectUrl, TokenUrl,
};
use rocket::http::Status;
// `request::Outcome` is the three-parameter `outcome::Outcome` specialised for
// guards; using the generic one here makes the error type come out wrong.
use rocket::request::{FromRequest, Outcome, Request};
use serde::Deserialize;
use shared::{Affiliation, Asn};
use surrealdb::types::Datetime;

use crate::config::AppConfig;
use crate::db::Database;
use crate::error::{Error, Result};
use crate::models::{AffiliationRecord, SessionRecord};

/// Cookie holding the opaque session token. Encrypted by Rocket's `secrets`
/// feature, so the client cannot read or forge it.
pub const SESSION_COOKIE: &str = "pdb_session";

/// Short-lived cookie holding the CSRF state and PKCE verifier while the user
/// is away at PeeringDB.
pub const FLOW_COOKIE: &str = "pdb_oauth_flow";

/// The OAuth endpoints we set, expressed in oauth2 5.x's typestate.
pub type OidcClient =
    BasicClient<EndpointSet, EndpointNotSet, EndpointNotSet, EndpointNotSet, EndpointSet>;

pub fn build_oidc_client(config: &AppConfig) -> Result<OidcClient> {
    let auth_url = AuthUrl::new(config.authorize_url())
        .map_err(|error| Error::Misconfigured(format!("invalid authorize URL: {error}")))?;
    let token_url = TokenUrl::new(config.token_url())
        .map_err(|error| Error::Misconfigured(format!("invalid token URL: {error}")))?;
    let redirect_url = RedirectUrl::new(config.redirect_uri())
        .map_err(|error| Error::Misconfigured(format!("invalid redirect URI: {error}")))?;

    Ok(
        BasicClient::new(ClientId::new(config.oidc_client_id.clone()))
            .set_client_secret(ClientSecret::new(config.oidc_client_secret.clone()))
            .set_auth_uri(auth_url)
            .set_token_uri(token_url)
            .set_redirect_uri(redirect_url),
    )
}

/// State parked in the flow cookie between the redirect out and back.
#[derive(Debug, serde::Serialize, Deserialize)]
pub struct FlowState {
    pub csrf: String,
    pub pkce_verifier: String,
    /// Same-site path to return to once login completes.
    pub next: String,
}

/// The PeeringDB `/oauth2/userinfo/` response. Every field beyond `sub` is
/// optional so a claim we did not request cannot break login.
#[derive(Debug, Deserialize)]
pub struct UserInfo {
    pub sub: String,
    #[serde(default)]
    pub preferred_username: Option<String>,
    #[serde(default)]
    pub given_name: Option<String>,
    #[serde(default)]
    pub family_name: Option<String>,
    #[serde(default)]
    pub name: Option<String>,
    #[serde(default)]
    pub email: Option<String>,
    /// PeeringDB's own "this person is who they say they are" flag.
    #[serde(default)]
    pub verified_user: Option<bool>,
    #[serde(default)]
    pub networks: Vec<UserNetwork>,
}

#[derive(Debug, Deserialize)]
pub struct UserNetwork {
    pub asn: u32,
    #[serde(default)]
    pub name: String,
    #[serde(default)]
    pub perms: u32,
}

impl UserInfo {
    pub fn display_name(&self) -> String {
        if let Some(name) = self.name.as_deref().filter(|n| !n.trim().is_empty()) {
            return name.to_owned();
        }
        let joined = [self.given_name.as_deref(), self.family_name.as_deref()]
            .into_iter()
            .flatten()
            .collect::<Vec<_>>()
            .join(" ");
        if joined.trim().is_empty() {
            self.username()
        } else {
            joined
        }
    }

    pub fn username(&self) -> String {
        self.preferred_username
            .clone()
            .unwrap_or_else(|| self.sub.clone())
    }

    pub fn affiliations(&self) -> Vec<Affiliation> {
        self.networks
            .iter()
            .map(|network| Affiliation {
                asn: Asn(network.asn),
                name: network.name.clone(),
                perms: network.perms,
            })
            .collect()
    }
}

/// Fetch the OIDC userinfo document with a freshly issued access token.
pub async fn fetch_userinfo(
    http: &reqwest::Client,
    config: &AppConfig,
    access_token: &str,
) -> Result<UserInfo> {
    let response = http
        .get(config.userinfo_url())
        .bearer_auth(access_token)
        .send()
        .await?;

    if !response.status().is_success() {
        return Err(Error::OAuth(format!(
            "userinfo returned HTTP {}",
            response.status()
        )));
    }

    response
        .json::<UserInfo>()
        .await
        .map_err(|error| Error::OAuth(format!("could not read the userinfo response: {error}")))
}

/// Create and persist a session for a freshly authenticated user.
pub async fn create_session(
    db: &Database,
    config: &AppConfig,
    user: &UserInfo,
) -> Result<SessionRecord> {
    let affiliations = user.affiliations();

    // Admin rights follow PeeringDB affiliation with one of our own networks.
    let is_admin = affiliations
        .iter()
        .any(|affiliation| config.is_local_asn(affiliation.asn));

    let now = Utc::now();
    let record = SessionRecord {
        token: CsrfToken::new_random_len(32).secret().clone(),
        subject: user.sub.clone(),
        username: user.username(),
        display_name: user.display_name(),
        email: user.email.clone(),
        verified: user.verified_user.unwrap_or(false),
        affiliations: affiliations.into_iter().map(AffiliationRecord::from).collect(),
        is_admin,
        created_at: Datetime::from(now),
        expires_at: Datetime::from(now + ChronoDuration::hours(config.session_ttl_hours)),
    };

    db.query("CREATE session CONTENT $record")
        .bind(("record", record.clone()))
        .await?
        .check()?;

    Ok(record)
}

pub async fn load_session(db: &Database, token: &str) -> Result<Option<SessionRecord>> {
    let mut response = db
        .query("SELECT * FROM session WHERE token = $token LIMIT 1")
        .bind(("token", token.to_owned()))
        .await?
        .check()?;

    let sessions: Vec<SessionRecord> = response.take(0)?;
    let Some(session) = sessions.into_iter().next() else {
        return Ok(None);
    };

    if session.is_expired(Utc::now()) {
        delete_session(db, token).await?;
        return Ok(None);
    }

    Ok(Some(session))
}

pub async fn delete_session(db: &Database, token: &str) -> Result<()> {
    db.query("DELETE FROM session WHERE token = $token")
        .bind(("token", token.to_owned()))
        .await?
        .check()?;
    Ok(())
}

/// A signed-in user. Requests without a valid session get a 401.
#[derive(Debug, Clone)]
pub struct AuthUser(pub SessionRecord);

#[rocket::async_trait]
impl<'r> FromRequest<'r> for AuthUser {
    type Error = ();

    async fn from_request(request: &'r Request<'_>) -> Outcome<Self, Self::Error> {
        let Some(cookie) = request.cookies().get_private(SESSION_COOKIE) else {
            return Outcome::Error((Status::Unauthorized, ()));
        };

        let Some(db) = request.rocket().state::<Database>() else {
            rocket::error!("database state missing while authenticating");
            return Outcome::Error((Status::InternalServerError, ()));
        };

        match load_session(db, cookie.value()).await {
            Ok(Some(session)) => Outcome::Success(AuthUser(session)),
            Ok(None) => {
                // Stale cookie: clear it so the browser stops sending it.
                request.cookies().remove_private(SESSION_COOKIE);
                Outcome::Error((Status::Unauthorized, ()))
            }
            Err(error) => {
                rocket::error!("session lookup failed: {error:?}");
                Outcome::Error((Status::InternalServerError, ()))
            }
        }
    }
}

/// A user affiliated with one of our own ASNs.
#[derive(Debug, Clone)]
pub struct AdminUser(pub SessionRecord);

#[rocket::async_trait]
impl<'r> FromRequest<'r> for AdminUser {
    type Error = ();

    async fn from_request(request: &'r Request<'_>) -> Outcome<Self, Self::Error> {
        match AuthUser::from_request(request).await {
            Outcome::Success(AuthUser(session)) if session.is_admin => {
                Outcome::Success(AdminUser(session))
            }
            Outcome::Success(_) => Outcome::Error((Status::Forbidden, ())),
            Outcome::Error(error) => Outcome::Error(error),
            Outcome::Forward(status) => Outcome::Forward(status),
        }
    }
}

impl AuthUser {
    /// Whether this user may file or read requests on behalf of `asn`.
    /// Admins can act for anyone so they can file on a peer's behalf.
    pub fn may_act_for(&self, asn: Asn) -> bool {
        self.0.is_admin || self.0.affiliations.iter().any(|a| a.asn == asn.0)
    }
}
