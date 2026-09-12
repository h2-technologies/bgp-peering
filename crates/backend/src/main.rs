//! bgp-peering: a peering request portal that uses PeeringDB as its source of
//! truth for both identity and network presence.

mod auth;
mod config;
mod db;
mod error;
mod models;
mod peeringdb;
mod routes;

use std::time::Duration;

use rocket::fs::FileServer;
use rocket::{catchers, routes};

use crate::config::AppConfig;

#[rocket::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Convenience for local development; in production the environment is
    // expected to carry the configuration already.
    let _ = dotenvy::dotenv();

    let rocket = rocket::build();
    let config: AppConfig = rocket.figment().extract()?;

    for warning in config.warnings() {
        rocket::warn!("{warning}");
    }

    let database = db::connect(&config.database_path).await?;
    db::prune_expired_sessions(&database).await?;

    let peeringdb = peeringdb::PeeringDb::new(&config, database.clone())?;
    let oidc = auth::build_oidc_client(&config)?;

    // Shared client for the OAuth token exchange and the userinfo call.
    let http = reqwest::Client::builder()
        .user_agent(concat!("bgp-peering/", env!("CARGO_PKG_VERSION")))
        .timeout(Duration::from_secs(20))
        .build()?;

    let mut server = rocket
        .manage(database)
        .manage(peeringdb)
        .manage(oidc)
        .manage(http)
        .mount(
            "/",
            routes![
                routes::api::site,
                routes::api::me,
                routes::api::overlap,
                routes::api::my_requests,
                routes::api::create_request,
                routes::api::withdraw_request,
                routes::api::all_requests,
                routes::api::decide_request,
                routes::auth::login,
                routes::auth::callback,
                routes::auth::logout,
            ],
        )
        .register(
            "/",
            catchers![routes::spa::not_found, routes::spa::default_catcher],
        );

    if config.static_dir.is_dir() {
        server = server.mount("/", FileServer::from(&config.static_dir));
    } else {
        rocket::warn!(
            "{} does not exist - the API will run but no frontend will be served. \
             Build it with `trunk build --release`.",
            config.static_dir.display()
        );
    }

    server.manage(config).launch().await?;
    Ok(())
}
