//! Server configuration.
//!
//! Everything here is read out of Rocket's figment, so any field can be set in
//! `Rocket.toml` or overridden by a `ROCKET_`-prefixed environment variable
//! (`ROCKET_OIDC_CLIENT_SECRET=...`, `ROCKET_LOCAL_ASNS='[64500, 64501]'`).

use std::path::PathBuf;

use serde::Deserialize;
use shared::Asn;

#[derive(Debug, Clone, Deserialize)]
pub struct AppConfig {
    /// Our own ASNs. Overlap is computed across all of them, and affiliation
    /// with any one of them grants admin access.
    #[serde(default)]
    pub local_asns: Vec<u32>,

    /// Base URL of the PeeringDB API, without a trailing slash.
    #[serde(default = "default_peeringdb_api")]
    pub peeringdb_api: String,

    /// Optional PeeringDB API key. Anonymous callers get a much lower rate
    /// limit, so this is worth setting even though everything we read is
    /// public.
    #[serde(default)]
    pub peeringdb_api_key: Option<String>,

    /// OIDC issuer. Overridable so a staging deployment can point elsewhere.
    #[serde(default = "default_oidc_issuer")]
    pub oidc_issuer: String,

    #[serde(default)]
    pub oidc_client_id: String,

    #[serde(default)]
    pub oidc_client_secret: String,

    /// Public origin of this site, used to build the OAuth redirect URI. Must
    /// match the redirect URI registered with PeeringDB exactly.
    #[serde(default = "default_public_url")]
    pub public_url: String,

    /// Directory for the embedded SurrealKV datastore.
    #[serde(default = "default_database_path")]
    pub database_path: PathBuf,

    /// How long a cached PeeringDB response stays usable.
    #[serde(default = "default_cache_ttl")]
    pub cache_ttl_seconds: u64,

    /// How long a login lasts before the user must re-authenticate.
    #[serde(default = "default_session_ttl")]
    pub session_ttl_hours: i64,

    /// Directory holding the Trunk-built frontend (`trunk build` output).
    #[serde(default = "default_static_dir")]
    pub static_dir: PathBuf,
}

fn default_peeringdb_api() -> String {
    "https://www.peeringdb.com/api".to_owned()
}

fn default_oidc_issuer() -> String {
    "https://auth.peeringdb.com/oauth2".to_owned()
}

fn default_public_url() -> String {
    "http://localhost:8000".to_owned()
}

fn default_database_path() -> PathBuf {
    PathBuf::from("data/surrealkv")
}

fn default_cache_ttl() -> u64 {
    3600
}

fn default_session_ttl() -> i64 {
    24 * 7
}

fn default_static_dir() -> PathBuf {
    PathBuf::from("dist")
}

impl AppConfig {
    pub fn local_asns(&self) -> Vec<Asn> {
        self.local_asns.iter().copied().map(Asn).collect()
    }

    pub fn is_local_asn(&self, asn: Asn) -> bool {
        self.local_asns.contains(&asn.0)
    }

    /// Where PeeringDB sends the user back after they authorise us.
    pub fn redirect_uri(&self) -> String {
        format!("{}/auth/callback", self.public_url.trim_end_matches('/'))
    }

    pub fn authorize_url(&self) -> String {
        format!("{}/authorize/", self.oidc_issuer.trim_end_matches('/'))
    }

    pub fn token_url(&self) -> String {
        format!("{}/token/", self.oidc_issuer.trim_end_matches('/'))
    }

    pub fn userinfo_url(&self) -> String {
        format!("{}/userinfo/", self.oidc_issuer.trim_end_matches('/'))
    }

    /// Whether OIDC credentials have actually been supplied. The server still
    /// boots without them so the site is browsable, but login will refuse.
    pub fn oidc_configured(&self) -> bool {
        !self.oidc_client_id.is_empty() && !self.oidc_client_secret.is_empty()
    }

    /// Problems worth shouting about at startup.
    pub fn warnings(&self) -> Vec<String> {
        let mut warnings = Vec::new();
        if self.local_asns.is_empty() {
            warnings.push(
                "local_asns is empty - every overlap lookup will come back empty. \
                 Set ROCKET_LOCAL_ASNS='[<your asn>]'."
                    .to_owned(),
            );
        }
        if !self.oidc_configured() {
            warnings.push(
                "oidc_client_id / oidc_client_secret are unset - PeeringDB login is disabled. \
                 Register an OAuth application at https://www.peeringdb.com/oauth2/applications/."
                    .to_owned(),
            );
        }
        if self.public_url.starts_with("http://") && !self.public_url.contains("localhost") {
            warnings.push(format!(
                "public_url {} is plaintext HTTP; session cookies will not be marked Secure.",
                self.public_url
            ));
        }
        warnings
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn config(public_url: &str) -> AppConfig {
        AppConfig {
            local_asns: vec![64500, 64501],
            peeringdb_api: default_peeringdb_api(),
            peeringdb_api_key: None,
            oidc_issuer: default_oidc_issuer(),
            oidc_client_id: String::new(),
            oidc_client_secret: String::new(),
            public_url: public_url.to_owned(),
            database_path: default_database_path(),
            cache_ttl_seconds: default_cache_ttl(),
            session_ttl_hours: default_session_ttl(),
            static_dir: default_static_dir(),
        }
    }

    #[test]
    fn the_redirect_uri_survives_a_trailing_slash() {
        assert_eq!(
            config("https://peering.example.net/").redirect_uri(),
            "https://peering.example.net/auth/callback"
        );
        assert_eq!(
            config("https://peering.example.net").redirect_uri(),
            "https://peering.example.net/auth/callback"
        );
    }

    #[test]
    fn oidc_endpoints_come_off_the_issuer() {
        let config = config("https://peering.example.net");
        assert_eq!(
            config.authorize_url(),
            "https://auth.peeringdb.com/oauth2/authorize/"
        );
        assert_eq!(
            config.token_url(),
            "https://auth.peeringdb.com/oauth2/token/"
        );
        assert_eq!(
            config.userinfo_url(),
            "https://auth.peeringdb.com/oauth2/userinfo/"
        );
    }

    #[test]
    fn local_asn_membership() {
        let config = config("https://peering.example.net");
        assert!(config.is_local_asn(Asn(64500)));
        assert!(!config.is_local_asn(Asn(64999)));
    }

    #[test]
    fn missing_credentials_are_warned_about_not_fatal() {
        let config = config("https://peering.example.net");
        assert!(!config.oidc_configured());
        assert!(config.warnings().iter().any(|w| w.contains("oidc_client_id")));
    }
}
