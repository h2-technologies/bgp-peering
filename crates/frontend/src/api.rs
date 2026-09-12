//! Typed wrappers around the backend's JSON API.
//!
//! Session state lives in a `HttpOnly` cookie, which `fetch` sends
//! automatically for same-origin requests, so nothing here handles tokens.

use gloo_net::http::Request;
use serde::Serialize;
use serde::de::DeserializeOwned;
use shared::{
    ApiError, CurrentUser, DecisionRequest, NewPeeringRequest, OverlapReport, PeeringRequest,
    RequestStatus, SiteInfo,
};

/// Why a call failed, in terms the UI can act on.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ApiFailure {
    /// No valid session — the UI should prompt for login rather than show an
    /// error, since this is the normal state for a signed-out visitor.
    Unauthorized,
    /// Anything else, already phrased for a human.
    Message(String),
}

impl ApiFailure {
    pub fn message(&self) -> String {
        match self {
            Self::Unauthorized => "Sign in with PeeringDB to continue.".to_owned(),
            Self::Message(text) => text.clone(),
        }
    }
}

impl From<gloo_net::Error> for ApiFailure {
    fn from(error: gloo_net::Error) -> Self {
        Self::Message(format!("Could not reach the server: {error}"))
    }
}

/// Read a response, turning a non-2xx status into the server's `ApiError`.
async fn read<T: DeserializeOwned>(
    response: gloo_net::http::Response,
) -> Result<T, ApiFailure> {
    let status = response.status();

    if status == 401 {
        return Err(ApiFailure::Unauthorized);
    }

    if !(200..300).contains(&status) {
        // The backend answers every failure with an ApiError body, but a proxy
        // or a panic could still produce something else.
        let message = match response.json::<ApiError>().await {
            Ok(error) => error.message,
            Err(_) => format!("The server returned HTTP {status}."),
        };
        return Err(ApiFailure::Message(message));
    }

    response
        .json::<T>()
        .await
        .map_err(|error| ApiFailure::Message(format!("Unexpected response: {error}")))
}

async fn get<T: DeserializeOwned>(url: &str) -> Result<T, ApiFailure> {
    read(Request::get(url).send().await?).await
}

async fn post<B: Serialize, T: DeserializeOwned>(url: &str, body: &B) -> Result<T, ApiFailure> {
    read(Request::post(url).json(body)?.send().await?).await
}

/// Who we are and whether login is available. Works signed out.
pub async fn site_info() -> Result<SiteInfo, ApiFailure> {
    get("/api/site").await
}

/// The current user, or `None` when signed out. Never an `Unauthorized`
/// failure — this endpoint answers for anonymous callers too.
pub async fn me() -> Result<Option<CurrentUser>, ApiFailure> {
    get("/api/me").await
}

pub async fn overlap(asn: u32) -> Result<OverlapReport, ApiFailure> {
    get(&format!("/api/overlap/{asn}")).await
}

pub async fn my_requests() -> Result<Vec<PeeringRequest>, ApiFailure> {
    get("/api/requests").await
}

pub async fn submit_request(body: &NewPeeringRequest) -> Result<PeeringRequest, ApiFailure> {
    post("/api/requests", body).await
}

pub async fn withdraw_request(id: &str) -> Result<PeeringRequest, ApiFailure> {
    read(
        Request::post(&format!("/api/requests/{id}/withdraw"))
            .send()
            .await?,
    )
    .await
}

pub async fn all_requests() -> Result<Vec<PeeringRequest>, ApiFailure> {
    get("/api/admin/requests").await
}

pub async fn decide_request(
    id: &str,
    status: RequestStatus,
    decision_note: Option<String>,
) -> Result<PeeringRequest, ApiFailure> {
    let body = DecisionRequest {
        status,
        decision_note,
    };
    read(
        Request::patch(&format!("/api/admin/requests/{id}"))
            .json(&body)?
            .send()
            .await?,
    )
    .await
}

pub async fn logout() -> Result<(), ApiFailure> {
    let response = Request::post("/auth/logout").send().await?;
    if (200..300).contains(&response.status()) {
        Ok(())
    } else {
        Err(ApiFailure::Message("Could not sign out.".to_owned()))
    }
}
