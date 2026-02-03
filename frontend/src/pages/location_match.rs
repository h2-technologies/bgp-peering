use yew::prelude::*;
use web_sys::HtmlInputElement;
use wasm_bindgen_futures::spawn_local;
use crate::api;

#[function_component(LocationMatch)]
pub fn location_match() -> Html {
    let our_asn = use_state(|| String::new());
    let requester_asn = use_state(|| String::new());
    let results = use_state(|| None::<api::LocationMatchResponse>);
    let error_message = use_state(|| String::new());
    let is_loading = use_state(|| false);
    
    let our_asn_clone = our_asn.clone();
    let on_our_asn_change = Callback::from(move |e: Event| {
        let input: HtmlInputElement = e.target_unchecked_into();
        our_asn_clone.set(input.value());
    });
    
    let requester_asn_clone = requester_asn.clone();
    let on_requester_asn_change = Callback::from(move |e: Event| {
        let input: HtmlInputElement = e.target_unchecked_into();
        requester_asn_clone.set(input.value());
    });
    
    let our_asn_value = (*our_asn).clone();
    let requester_asn_value = (*requester_asn).clone();
    let results_setter = results.clone();
    let error_setter = error_message.clone();
    let loading_setter = is_loading.clone();
    
    let on_submit = Callback::from(move |e: SubmitEvent| {
        e.prevent_default();
        
        let our_asn_str = our_asn_value.clone();
        let requester_asn_str = requester_asn_value.clone();
        let results = results_setter.clone();
        let error = error_setter.clone();
        let loading = loading_setter.clone();
        
        // Parse ASNs
        let our_asn_parsed: Result<u32, _> = our_asn_str.parse();
        let requester_asn_parsed: Result<u32, _> = requester_asn_str.parse();
        
        match (our_asn_parsed, requester_asn_parsed) {
            (Ok(our), Ok(requester)) => {
                loading.set(true);
                error.set(String::new());
                
                spawn_local(async move {
                    match api::match_locations(our, requester).await {
                        Ok(response) => {
                            results.set(Some(response));
                            error.set(String::new());
                        }
                        Err(e) => {
                            error.set(format!("Error: {}", e));
                            results.set(None);
                        }
                    }
                    loading.set(false);
                });
            }
            _ => {
                error.set("Please enter valid ASN numbers".to_string());
            }
        }
    });
    
    html! {
        <div class="location-match-page">
            <div class="location-match-container">
                <h2>{ "Check Common Locations" }</h2>
                <p class="description">
                    { "Enter ASN numbers to find common peering locations" }
                </p>
                
                <form onsubmit={on_submit}>
                    <div class="form-group">
                        <label for="our_asn">{ "Our ASN:" }</label>
                        <input
                            type="text"
                            id="our_asn"
                            placeholder="e.g., 64512"
                            onchange={on_our_asn_change}
                            disabled={*is_loading}
                        />
                    </div>
                    
                    <div class="form-group">
                        <label for="requester_asn">{ "Requester ASN:" }</label>
                        <input
                            type="text"
                            id="requester_asn"
                            placeholder="e.g., 64513"
                            onchange={on_requester_asn_change}
                            disabled={*is_loading}
                        />
                    </div>
                    
                    <button type="submit" disabled={*is_loading}>
                        { if *is_loading { "Checking..." } else { "Check Locations" } }
                    </button>
                </form>
                
                {if !error_message.is_empty() {
                    html! {
                        <div class="error-message">
                            { (*error_message).clone() }
                        </div>
                    }
                } else {
                    html! {}
                }}
                
                {if let Some(response) = (*results).clone() {
                    html! {
                        <div class="results">
                            <h3>{ "Results" }</h3>
                            
                            <div class="section">
                                <h4>{ "Common Locations" }</h4>
                                {if response.matches.is_empty() {
                                    html! {
                                        <p class="no-results">{ "No common locations found" }</p>
                                    }
                                } else {
                                    html! {
                                        <ul class="location-list">
                                            {for response.matches.iter().map(|location| {
                                                html! {
                                                    <li class="location-item">
                                                        <strong>{ &location.name }</strong>
                                                        <br />
                                                        { format!("{}, {}", &location.city, &location.country) }
                                                        <br />
                                                        <small>{ format!("Facility ID: {}", &location.facility_id) }</small>
                                                    </li>
                                                }
                                            })}
                                        </ul>
                                    }
                                }}
                            </div>
                            
                            <div class="section">
                                <h4>{ "Our Locations" }</h4>
                                <ul class="simple-list">
                                    {for response.our_locations.iter().map(|loc| {
                                        html! { <li>{ loc }</li> }
                                    })}
                                </ul>
                            </div>
                            
                            <div class="section">
                                <h4>{ "Requester Locations" }</h4>
                                <ul class="simple-list">
                                    {for response.requester_locations.iter().map(|loc| {
                                        html! { <li>{ loc }</li> }
                                    })}
                                </ul>
                            </div>
                        </div>
                    }
                } else {
                    html! {}
                }}
            </div>
        </div>
    }
}
