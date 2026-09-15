//! The PeeringDB OIDC authorization-code flow.

use oauth2::{
    AuthorizationCode, CsrfToken, PkceCodeChallenge, PkceCodeVerifier, Scope, TokenResponse,
};
use rocket::http::{Cookie, CookieJar, SameSite};
use rocket::response::Redirect;
use rocket::serde::json::Json;
use rocket::{State, get, post};
use serde_json::json;

use crate::auth::{
    AuthUser, FLOW_COOKIE, FlowState, SESSION_COOKIE, OidcClient, create_session, delete_session,
    fetch_userinfo,
};
use crate::config::AppConfig;
use crate::db::Database;
use crate::error::{Error, Result};

/// Only ever redirect back to a path on this site, never to an absolute URL a
/// caller supplied — otherwise `?next=` is an open redirect.
fn safe_next(next: Option<String>) -> String {
    next.filter(|path| path.starts_with('/') && !path.starts_with("//"))
        .unwrap_or_else(|| "/".to_owned())
}

fn flow_cookie(value: String, secure: bool) -> Cookie<'static> {
    Cookie::build((FLOW_COOKIE, value))
        .path("/")
        .http_only(true)
        .secure(secure)
        // Lax, not Strict: the browser must still send this cookie on the
        // top-level redirect back from PeeringDB.
        .same_site(SameSite::Lax)
        .build()
}

fn session_cookie(value: String, secure: bool) -> Cookie<'static> {
    Cookie::build((SESSION_COOKIE, value))
        .path("/")
        .http_only(true)
        .secure(secure)
        .same_site(SameSite::Lax)
        .build()
}

#[get("/auth/login?<next>")]
pub fn login(
    config: &State<AppConfig>,
    client: &State<OidcClient>,
    jar: &CookieJar<'_>,
    next: Option<String>,
) -> Result<Redirect> {
    if !config.oidc_configured() {
        return Err(Error::Misconfigured(
            "PeeringDB login is not configured on this server.".to_owned(),
        ));
    }

    let (challenge, verifier) = PkceCodeChallenge::new_random_sha256();
    let (authorize_url, csrf) = client
        .authorize_url(CsrfToken::new_random)
        .add_scope(Scope::new("openid".to_owned()))
        .add_scope(Scope::new("profile".to_owned()))
        .add_scope(Scope::new("email".to_owned()))
        // The scope that makes this site work: which ASNs the user may act for.
        .add_scope(Scope::new("networks".to_owned()))
        .set_pkce_challenge(challenge)
        .url();

    let flow = FlowState {
        csrf: csrf.secret().clone(),
        pkce_verifier: verifier.secret().clone(),
        next: safe_next(next),
    };

    let secure = config.public_url.starts_with("https://");
    jar.add_private(flow_cookie(serde_json::to_string(&flow)?, secure));

    Ok(Redirect::to(authorize_url.to_string()))
}

#[get("/auth/callback?<code>&<state>&<error>&<error_description>")]
pub async fn callback(
    config: &State<AppConfig>,
    client: &State<OidcClient>,
    http: &State<reqwest::Client>,
    db: &State<Database>,
    jar: &CookieJar<'_>,
    code: Option<String>,
    state: Option<String>,
    error: Option<String>,
    error_description: Option<String>,
) -> Result<Redirect> {
    // The flow cookie is single-use whatever happens next.
    let stored = jar.get_private(FLOW_COOKIE).map(|c| c.value().to_owned());
    jar.remove_private(Cookie::from(FLOW_COOKIE));

    if let Some(error) = error {
        let detail = error_description.unwrap_or_else(|| error.clone());
        return Err(Error::OAuth(format!("PeeringDB refused the login: {detail}")));
    }

    let stored = stored.ok_or_else(|| {
        Error::OAuth("the login attempt expired; please start again".to_owned())
    })?;
    let flow: FlowState = serde_json::from_str(&stored)
        .map_err(|_| Error::OAuth("the login attempt was malformed".to_owned()))?;

    let state = state.ok_or_else(|| Error::OAuth("missing state parameter".to_owned()))?;
    if state != flow.csrf {
        return Err(Error::OAuth(
            "state mismatch - the login may have been tampered with".to_owned(),
        ));
    }

    let code = code.ok_or_else(|| Error::OAuth("missing authorization code".to_owned()))?;

    let token = client
        .exchange_code(AuthorizationCode::new(code))
        .set_pkce_verifier(PkceCodeVerifier::new(flow.pkce_verifier))
        .request_async(http.inner())
        .await
        .map_err(|error| Error::OAuth(format!("token exchange failed: {error}")))?;

    let userinfo = fetch_userinfo(http.inner(), config, token.access_token().secret()).await?;
    let session = create_session(db.inner(), config, &userinfo).await?;

    rocket::info!(
        "{} ({}) signed in with {} affiliation(s){}",
        session.display_name,
        session.subject,
        session.affiliations.len(),
        if session.is_admin { ", admin" } else { "" }
    );

    let secure = config.public_url.starts_with("https://");
    jar.add_private(session_cookie(session.token, secure));

    Ok(Redirect::to(flow.next))
}

#[post("/auth/logout")]
pub async fn logout(
    db: &State<Database>,
    jar: &CookieJar<'_>,
    user: Option<AuthUser>,
) -> Result<Json<serde_json::Value>> {
    if let Some(AuthUser(session)) = user {
        delete_session(db.inner(), &session.token).await?;
    }
    jar.remove_private(Cookie::from(SESSION_COOKIE));
    Ok(Json(json!({ "ok": true })))
}
