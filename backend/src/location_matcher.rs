use crate::peeringdb;
use crate::{LocationMatch, LocationMatchResponse};

/// Find common locations between two ASNs
pub async fn find_common_locations(
    our_asn: u32,
    requester_asn: u32,
) -> std::result::Result<LocationMatchResponse, String> {
    // Fetch network info for both ASNs
    let our_net = peeringdb::get_network_by_asn(our_asn).await?;
    let requester_net = peeringdb::get_network_by_asn(requester_asn).await?;
    
    // Fetch facilities for both networks
    let our_facilities = peeringdb::get_network_facilities(our_net.id).await?;
    let requester_facilities = peeringdb::get_network_facilities(requester_net.id).await?;
    
    // Find common facilities
    let mut matches = Vec::new();
    let mut our_locations = Vec::new();
    let mut requester_locations = Vec::new();
    
    // Build list of our locations
    for fac in &our_facilities {
        let location = format!("{}, {}", fac.city, fac.country);
        if !our_locations.contains(&location) {
            our_locations.push(location);
        }
    }
    
    // Build list of requester locations and find matches
    for req_fac in &requester_facilities {
        let location = format!("{}, {}", req_fac.city, req_fac.country);
        if !requester_locations.contains(&location) {
            requester_locations.push(location.clone());
        }
        
        // Check if we have a facility at the same location
        for our_fac in &our_facilities {
            if our_fac.id == req_fac.id {
                // Exact facility match
                matches.push(LocationMatch {
                    facility_id: format!("{}", our_fac.id),
                    name: our_fac.name.clone(),
                    city: our_fac.city.clone(),
                    country: our_fac.country.clone(),
                });
            } else if our_fac.city == req_fac.city && our_fac.country == req_fac.country {
                // Same city, different facility
                let match_exists = matches.iter().any(|m| {
                    m.city == our_fac.city && m.country == our_fac.country && m.facility_id == format!("{}", our_fac.id)
                });
                
                if !match_exists {
                    matches.push(LocationMatch {
                        facility_id: format!("{}", our_fac.id),
                        name: our_fac.name.clone(),
                        city: our_fac.city.clone(),
                        country: our_fac.country.clone(),
                    });
                }
            }
        }
    }
    
    Ok(LocationMatchResponse {
        matches,
        our_locations,
        requester_locations,
    })
}
