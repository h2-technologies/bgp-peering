//! Serving the Trunk-built Yew bundle.
//!
//! Static assets are handled by a `FileServer`; anything it does not match
//! falls through to here. Client-side routes (`/request`, `/admin`, ...) have
//! no file behind them, so they get `index.html` and the SPA router takes over.

use rocket::fs::NamedFile;
use rocket::http::Status;
use rocket::serde::json::Json;
use rocket::{Request, catch};
use shared::ApiError;

use crate::config::AppConfig;

/// Guard rejections (401 from `AuthUser`, 403 from `AdminUser`) and any other
/// non-404 error would otherwise render Rocket's HTML page, which `fetch()`
/// cannot parse. Give every one of them the same JSON shape as `Error`.
#[catch(default)]
pub fn default_catcher(status: Status, _request: &Request<'_>) -> (Status, Json<ApiError>) {
    let code = match status.code {
        401 => "unauthenticated",
        403 => "forbidden",
        422 => "unprocessable",
        _ => "error",
    };

    let message = match status.code {
        401 => "Sign in with PeeringDB to continue.".to_owned(),
        403 => "Your PeeringDB affiliations do not allow that.".to_owned(),
        422 => "The request body was not in the expected shape.".to_owned(),
        _ => status.reason().unwrap_or("Request failed").to_owned(),
    };

    (
        status,
        Json(ApiError {
            error: code.to_owned(),
            message,
        }),
    )
}

#[catch(404)]
pub async fn not_found(
    request: &Request<'_>,
) -> Result<(Status, NamedFile), (Status, Json<ApiError>)> {
    let path = request.uri().path().as_str().to_owned();

    // API and auth misses stay JSON; handing an HTML page to fetch() would
    // just produce a confusing parse error in the browser.
    if path.starts_with("/api/") || path.starts_with("/auth/") {
        return Err((
            Status::NotFound,
            Json(ApiError {
                error: "not_found".to_owned(),
                message: format!("No such endpoint: {path}"),
            }),
        ));
    }

    let index = request
        .rocket()
        .state::<AppConfig>()
        .map(|config| config.static_dir.join("index.html"));

    match index {
        // 200, not 404: `/peering` is a real page, it just lives in the router.
        Some(path) => NamedFile::open(&path).await.map(|file| (Status::Ok, file)).map_err(|_| {
            (
                Status::NotFound,
                Json(ApiError {
                    error: "frontend_missing".to_owned(),
                    message: format!(
                        "The frontend bundle is not built. Run `trunk build` \
                         (expected {}).",
                        path.display()
                    ),
                }),
            )
        }),
        None => Err((
            Status::InternalServerError,
            Json(ApiError {
                error: "misconfigured".to_owned(),
                message: "Server configuration is unavailable.".to_owned(),
            }),
        )),
    }
}
