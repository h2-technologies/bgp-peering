use worker::*;
use serde::{Deserialize, Serialize};

mod peeringdb;
mod location_matcher;

// OAuth Configuration
const OAUTH_CLIENT_ID: &str = "EzgvjRew4k8Yx8YBW6FkuZAaG4ws9lq4cfRT9kh5";
const OAUTH_CLIENT_SECRET: &str = "Y4DAw45XI16cQpKo97rMdWwiN7WRrCraDgIv6SXmlYrmhRSt9VO0FI6TVmQa3iXk43tjt3NlsIWzq9E3zlMwUMjYgq02uEyaSvlDgmiYVgC0YfpRMFriIYgTppg1iCL8";
const PEERINGDB_OAUTH_AUTHORIZE: &str = "https://auth.peeringdb.com/oauth2/authorize/";
const PEERINGDB_OAUTH_TOKEN: &str = "https://auth.peeringdb.com/oauth2/token/";
const REDIRECT_URI: &str = "https://api.peering.austinh.dev/api/oauth/callback";

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
struct OAuthUrlResponse {
    authorization_url: String,
}

#[derive(Serialize, Deserialize)]
struct PeeringDBTokenResponse {
    access_token: String,
    token_type: String,
    expires_in: u32,
    refresh_token: Option<String>,
}

async fn exchange_code_for_token(code: &str) -> std::result::Result<PeeringDBTokenResponse, String> {
    let token_params = format!(
        "grant_type=authorization_code&code={}&redirect_uri={}&client_id={}&client_secret={}",
        urlencoding::encode(code),
        urlencoding::encode(REDIRECT_URI),
        urlencoding::encode(OAUTH_CLIENT_ID),
        urlencoding::encode(OAUTH_CLIENT_SECRET)
    );
    
    let headers = Headers::new();
    headers.set("Content-Type", "application/x-www-form-urlencoded").map_err(|e| e.to_string())?;
    headers.set("Accept", "application/json").map_err(|e| e.to_string())?;
    
    let request = Request::new_with_init(
        PEERINGDB_OAUTH_TOKEN,
        RequestInit::new()
            .with_method(Method::Post)
            .with_headers(headers)
            .with_body(Some(token_params.into())),
    ).map_err(|e| e.to_string())?;
    
    match Fetch::Request(request).send().await {
        Ok(mut response) => {
            if response.status_code() == 200 {
                response
                    .json::<PeeringDBTokenResponse>()
                    .await
                    .map_err(|e| format!("Failed to parse token response: {}", e))
            } else {
                let error_text = response.text().await.unwrap_or_else(|_| "Unknown error".to_string());
                Err(format!("Token exchange failed with status {}: {}", response.status_code(), error_text))
            }
        }
        Err(e) => Err(format!("Failed to send token request: {}", e)),
    }
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
        // OAuth: Get authorization URL
        .get("/api/oauth/authorize-url", |_req, _ctx| {
            let auth_url = format!(
                "{}?client_id={}&redirect_uri={}&response_type=code&scope=profile email",
                PEERINGDB_OAUTH_AUTHORIZE,
                OAUTH_CLIENT_ID,
                urlencoding::encode(REDIRECT_URI)
            );
            Response::from_json(&OAuthUrlResponse {
                authorization_url: auth_url,
            })
        })
        // OAuth: Callback endpoint
        .get_async("/api/oauth/callback", |req, _ctx| async move {
            let url = req.url()?;
            
            // Extract the authorization code from query parameters
            let query_pairs: std::collections::HashMap<String, String> = url
                .query_pairs()
                .map(|(k, v)| (k.to_string(), v.to_string()))
                .collect();
            
            if let Some(code) = query_pairs.get("code") {
                // Exchange the authorization code for an access token
                match exchange_code_for_token(code).await {
                    Ok(token_response) => {
                        // Redirect to frontend with token in URL fragment
                        let frontend_url = format!(
                            "https://peering.austinh.dev/#/login?token={}",
                            urlencoding::encode(&token_response.access_token)
                        );
                        Response::redirect(Url::parse(&frontend_url)?)
                    }
                    Err(e) => {
                        let error_url = format!(
                            "https://peering.austinh.dev/#/login?error={}",
                            urlencoding::encode(&format!("Token exchange failed: {}", e))
                        );
                        Response::redirect(Url::parse(&error_url)?)
                    }
                }
            } else if let Some(error) = query_pairs.get("error") {
                let error_url = format!(
                    "https://peering.austinh.dev/#/login?error={}",
                    urlencoding::encode(error)
                );
                Response::redirect(Url::parse(&error_url)?)
            } else {
                Response::error("Missing authorization code", 400)
            }
        })
        // Legacy auth endpoint (keep for backwards compatibility)
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