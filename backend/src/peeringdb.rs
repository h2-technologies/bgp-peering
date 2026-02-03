use worker::*;
use serde::{Deserialize, Serialize};

const PEERINGDB_API_BASE: &str = "https://www.peeringdb.com/api";

#[derive(Serialize, Deserialize, Debug)]
pub struct PeeringDBNet {
    pub id: u32,
    pub asn: u32,
    pub name: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct PeeringDBFacility {
    pub id: u32,
    pub name: String,
    pub city: String,
    pub country: String,
}

/// Authenticate with PeeringDB API
/// Note: PeeringDB uses OAuth2 or API keys in production
/// This is a simplified implementation for demonstration
pub async fn authenticate(username: &str, password: &str) -> std::result::Result<String, String> {
    // For demonstration purposes, we'll do basic validation
    // In production, this would make an actual API call to PeeringDB
    if username.is_empty() || password.is_empty() {
        return Err("Username and password are required".to_string());
    }
    
    // Simulate API key generation
    // In production, this would exchange credentials for an API token
    let token = format!("pdb_token_{}", username);
    Ok(token)
}

/// Fetch network information from PeeringDB by ASN
pub async fn get_network_by_asn(asn: u32) -> std::result::Result<PeeringDBNet, String> {
    let url = format!("{}/net?asn={}", PEERINGDB_API_BASE, asn);
    
    // Use Fetch API for the request
    let headers = Headers::new();
    headers.set("Accept", "application/json").map_err(|e| e.to_string())?;
    
    let request = Request::new_with_init(
        &url,
        RequestInit::new()
            .with_method(Method::Get)
            .with_headers(headers),
    ).map_err(|e| e.to_string())?;
    
    match Fetch::Request(request).send().await {
        Ok(mut response) => {
            if response.status_code() == 200 {
                match response.json::<serde_json::Value>().await {
                    Ok(data) => {
                        // PeeringDB returns data in a "data" array
                        if let Some(nets) = data.get("data").and_then(|d| d.as_array()) {
                            if let Some(net) = nets.first() {
                                let net_obj = PeeringDBNet {
                                    id: net.get("id").and_then(|v| v.as_u64()).unwrap_or(0) as u32,
                                    asn: net.get("asn").and_then(|v| v.as_u64()).unwrap_or(0) as u32,
                                    name: net.get("name").and_then(|v| v.as_str()).unwrap_or("Unknown").to_string(),
                                };
                                return Ok(net_obj);
                            }
                        }
                        Err("Network not found in PeeringDB".to_string())
                    }
                    Err(e) => Err(format!("Failed to parse response: {}", e)),
                }
            } else {
                Err(format!("PeeringDB API error: {}", response.status_code()))
            }
        }
        Err(e) => Err(format!("Failed to fetch from PeeringDB: {}", e)),
    }
}

/// Fetch facilities where a network is present
pub async fn get_network_facilities(net_id: u32) -> std::result::Result<Vec<PeeringDBFacility>, String> {
    let url = format!("{}/netfac?net_id={}", PEERINGDB_API_BASE, net_id);
    
    let headers = Headers::new();
    headers.set("Accept", "application/json").map_err(|e| e.to_string())?;
    
    let request = Request::new_with_init(
        &url,
        RequestInit::new()
            .with_method(Method::Get)
            .with_headers(headers),
    ).map_err(|e| e.to_string())?;
    
    match Fetch::Request(request).send().await {
        Ok(mut response) => {
            if response.status_code() == 200 {
                match response.json::<serde_json::Value>().await {
                    Ok(data) => {
                        let mut facilities = Vec::new();
                        
                        if let Some(netfacs) = data.get("data").and_then(|d| d.as_array()) {
                            for netfac in netfacs {
                                if let Some(fac_id) = netfac.get("fac_id").and_then(|v| v.as_u64()) {
                                    // Fetch facility details
                                    if let Ok(fac) = get_facility_by_id(fac_id as u32).await {
                                        facilities.push(fac);
                                    }
                                }
                            }
                        }
                        
                        Ok(facilities)
                    }
                    Err(e) => Err(format!("Failed to parse netfac response: {}", e)),
                }
            } else {
                Err(format!("PeeringDB API error: {}", response.status_code()))
            }
        }
        Err(e) => Err(format!("Failed to fetch network facilities: {}", e)),
    }
}

/// Fetch facility details by ID
async fn get_facility_by_id(fac_id: u32) -> std::result::Result<PeeringDBFacility, String> {
    let url = format!("{}/fac/{}", PEERINGDB_API_BASE, fac_id);
    
    let headers = Headers::new();
    headers.set("Accept", "application/json").map_err(|e| e.to_string())?;
    
    let request = Request::new_with_init(
        &url,
        RequestInit::new()
            .with_method(Method::Get)
            .with_headers(headers),
    ).map_err(|e| e.to_string())?;
    
    match Fetch::Request(request).send().await {
        Ok(mut response) => {
            if response.status_code() == 200 {
                match response.json::<serde_json::Value>().await {
                    Ok(data) => {
                        if let Some(facs) = data.get("data").and_then(|d| d.as_array()) {
                            if let Some(fac) = facs.first() {
                                let facility = PeeringDBFacility {
                                    id: fac.get("id").and_then(|v| v.as_u64()).unwrap_or(0) as u32,
                                    name: fac.get("name").and_then(|v| v.as_str()).unwrap_or("Unknown").to_string(),
                                    city: fac.get("city").and_then(|v| v.as_str()).unwrap_or("Unknown").to_string(),
                                    country: fac.get("country").and_then(|v| v.as_str()).unwrap_or("Unknown").to_string(),
                                };
                                return Ok(facility);
                            }
                        }
                        Err("Facility not found".to_string())
                    }
                    Err(e) => Err(format!("Failed to parse facility response: {}", e)),
                }
            } else {
                Err(format!("PeeringDB API error: {}", response.status_code()))
            }
        }
        Err(e) => Err(format!("Failed to fetch facility: {}", e)),
    }
}
