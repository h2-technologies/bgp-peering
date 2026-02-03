use worker::*;
use serde::{Deserialize, Serialize};

mod peeringdb;
mod location_matcher;

#[derive(Serialize, Deserialize)]
struct AuthRequest {
    username: String,
    password: String,
}

#[derive(Serialize, Deserialize)]
struct AuthResponse {
    success: bool,
    token: Option<String>,
    message: String,
}

#[derive(Serialize, Deserialize)]
struct LocationMatchRequest {
    our_asn: u32,
    requester_asn: u32,
}

#[derive(Serialize, Deserialize)]
struct LocationMatch {
    facility_id: String,
    name: String,
    city: String,
    country: String,
}

#[derive(Serialize, Deserialize)]
struct LocationMatchResponse {
    matches: Vec<LocationMatch>,
    our_locations: Vec<String>,
    requester_locations: Vec<String>,
}

#[event(fetch)]
async fn fetch(
    req: Request,
    env: Env,
    _ctx: Context,
) -> Result<Response> {
    let router = Router::new();
    
    router
        .post_async("/api/auth", |mut req, _ctx| async move {
            match req.json::<AuthRequest>().await {
                Ok(auth_req) => {
                    match peeringdb::authenticate(&auth_req.username, &auth_req.password).await {
                        Ok(token) => {
                            Response::from_json(&AuthResponse {
                                success: true,
                                token: Some(token),
                                message: "Authentication successful".to_string(),
                            })
                        }
                        Err(e) => {
                            Response::from_json(&AuthResponse {
                                success: false,
                                token: None,
                                message: format!("Authentication failed: {}", e),
                            })
                        }
                    }
                }
                Err(_) => Response::error("Invalid request body", 400),
            }
        })
        .post_async("/api/match-locations", |mut req, _ctx| async move {
            match req.json::<LocationMatchRequest>().await {
                Ok(match_req) => {
                    match location_matcher::find_common_locations(
                        match_req.our_asn,
                        match_req.requester_asn,
                    )
                    .await
                    {
                        Ok(response) => Response::from_json(&response),
                        Err(e) => Response::error(&format!("Failed to match locations: {}", e), 500),
                    }
                }
                Err(_) => Response::error("Invalid request body", 400),
            }
        })
        .get("/", |_, _| Response::ok("Peering Portal API v1.0"))
        .run(req, env)
        .await
}