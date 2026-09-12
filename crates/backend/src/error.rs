//! One error type for the whole server, with a uniform JSON representation.

use rocket::Request;
use rocket::http::Status;
use rocket::response::{self, Responder};
use rocket::serde::json::Json;
use shared::ApiError;

pub type Result<T, E = Error> = std::result::Result<T, E>;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("{0}")]
    Forbidden(String),

    #[error("{0}")]
    NotFound(String),

    #[error("{0}")]
    BadRequest(String),

    /// PeeringDB answered, but not with something we can use.
    #[error("PeeringDB returned an unexpected response: {0}")]
    PeeringDb(String),

    #[error("OAuth flow failed: {0}")]
    OAuth(String),

    #[error("server is missing configuration: {0}")]
    Misconfigured(String),

    #[error(transparent)]
    Http(#[from] reqwest::Error),

    #[error(transparent)]
    Db(#[from] surrealdb::Error),

    #[error(transparent)]
    Json(#[from] serde_json::Error),

    #[error(transparent)]
    Io(#[from] std::io::Error),
}

impl Error {
    /// The HTTP status and stable machine-readable code for this error.
    fn parts(&self) -> (Status, &'static str) {
        match self {
            Self::Forbidden(_) => (Status::Forbidden, "forbidden"),
            Self::NotFound(_) => (Status::NotFound, "not_found"),
            Self::BadRequest(_) => (Status::BadRequest, "bad_request"),
            Self::PeeringDb(_) => (Status::BadGateway, "peeringdb_error"),
            Self::OAuth(_) => (Status::BadGateway, "oauth_error"),
            Self::Misconfigured(_) => (Status::InternalServerError, "misconfigured"),
            Self::Http(_) => (Status::BadGateway, "upstream_unreachable"),
            Self::Db(_) => (Status::InternalServerError, "database_error"),
            Self::Json(_) => (Status::InternalServerError, "serialization_error"),
            Self::Io(_) => (Status::InternalServerError, "io_error"),
        }
    }

    /// What we are willing to tell the caller. Internal failures are logged in
    /// full but reported generically so we do not leak paths or queries.
    fn public_message(&self) -> String {
        match self {
            Self::Db(_) | Self::Json(_) | Self::Io(_) => "Internal server error.".to_owned(),
            Self::Http(_) => "Could not reach PeeringDB.".to_owned(),
            other => other.to_string(),
        }
    }
}

impl<'r> Responder<'r, 'static> for Error {
    fn respond_to(self, request: &'r Request<'_>) -> response::Result<'static> {
        let (status, code) = self.parts();

        if status.code >= 500 {
            rocket::error!("{}: {self:?}", request.uri());
        } else {
            rocket::warn!("{}: {self}", request.uri());
        }

        let body = Json(ApiError {
            error: code.to_owned(),
            message: self.public_message(),
        });

        let mut response = body.respond_to(request)?;
        response.set_status(status);
        Ok(response)
    }
}
