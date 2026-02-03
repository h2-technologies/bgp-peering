use gloo_net::http::Request;
use serde::{Deserialize, Serialize};

// API base URL - adjust this to match your worker deployment
const API_BASE: &str = "/api";

#[derive(Serialize, Deserialize, Clone)]
pub struct AuthRequest {
    pub username: String,
    pub password: String,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct AuthResponse {
    pub success: bool,
    pub token: Option<String>,
    pub message: String,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct LocationMatchRequest {
    pub our_asn: u32,
    pub requester_asn: u32,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct LocationMatch {
    pub facility_id: String,
    pub name: String,
    pub city: String,
    pub country: String,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct LocationMatchResponse {
    pub matches: Vec<LocationMatch>,
    pub our_locations: Vec<String>,
    pub requester_locations: Vec<String>,
}

pub async fn authenticate(username: String, password: String) -> Result<AuthResponse, String> {
    let request_body = AuthRequest { username, password };
    
    let response = Request::post(&format!("{}/auth", API_BASE))
        .json(&request_body)
        .map_err(|e| format!("Failed to create request: {}", e))?
        .send()
        .await
        .map_err(|e| format!("Failed to send request: {}", e))?;
    
    if response.ok() {
        response
            .json::<AuthResponse>()
            .await
            .map_err(|e| format!("Failed to parse response: {}", e))
    } else {
        Err(format!("Authentication failed with status: {}", response.status()))
    }
}

pub async fn match_locations(our_asn: u32, requester_asn: u32) -> Result<LocationMatchResponse, String> {
    let request_body = LocationMatchRequest {
        our_asn,
        requester_asn,
    };
    
    let response = Request::post(&format!("{}/match-locations", API_BASE))
        .json(&request_body)
        .map_err(|e| format!("Failed to create request: {}", e))?
        .send()
        .await
        .map_err(|e| format!("Failed to send request: {}", e))?;
    
    if response.ok() {
        response
            .json::<LocationMatchResponse>()
            .await
            .map_err(|e| format!("Failed to parse response: {}", e))
    } else {
        Err(format!("Location matching failed with status: {}", response.status()))
    }
}
