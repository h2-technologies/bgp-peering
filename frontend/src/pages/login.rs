use yew::prelude::*;
use web_sys::window;
use wasm_bindgen_futures::spawn_local;
use crate::api;

#[function_component(Login)]
pub fn login() -> Html {
    let message = use_state(|| String::new());
    let is_loading = use_state(|| false);
    
    // Check if we have a token in the URL (from OAuth callback)
    {
        let message = message.clone();
        use_effect_with((), move |_| {
            if let Some(window) = window() {
                if let Ok(location) = window.location().hash() {
                    // Parse URL parameters from hash
                    if location.contains("token=") {
                        // Extract token from URL
                        if let Some(token_part) = location.split("token=").nth(1) {
                            let token = token_part.split('&').next().unwrap_or("");
                            
                            // Decode the token
                            if let Ok(decoded_token) = urlencoding::decode(token) {
                                // Store token in local storage
                                if let Ok(Some(storage)) = window.local_storage() {
                                    let _ = storage.set_item("peeringdb_token", &decoded_token);
                                    message.set("✓ Successfully logged in with PeeringDB!".to_string());
                                }
                            }
                        }
                    } else if location.contains("error=") {
                        if let Some(error_part) = location.split("error=").nth(1) {
                            let error = error_part.split('&').next().unwrap_or("Unknown error");
                            if let Ok(decoded_error) = urlencoding::decode(error) {
                                message.set(format!("✗ {}", decoded_error));
                            }
                        }
                    }
                }
            }
            || ()
        });
    }
    
    let loading_setter = is_loading.clone();
    let message_setter = message.clone();
    
    let on_oauth_login = Callback::from(move |_: MouseEvent| {
        let loading = loading_setter.clone();
        let message = message_setter.clone();
        
        loading.set(true);
        
        spawn_local(async move {
            match api::get_oauth_url().await {
                Ok(response) => {
                    // Redirect to PeeringDB OAuth authorization page
                    if let Some(window) = window() {
                        let _ = window.location().set_href(&response.authorization_url);
                    }
                }
                Err(e) => {
                    message.set(format!("✗ Error: {}", e));
                    loading.set(false);
                }
            }
        });
    });
    
    html! {
        <div class="login-page">
            <div class="login-container">
                <h2>{ "Login with PeeringDB" }</h2>
                <p class="login-description">
                    { "Click the button below to authenticate with your PeeringDB account using OAuth" }
                </p>
                
                <button 
                    class="oauth-button"
                    onclick={on_oauth_login}
                    disabled={*is_loading}
                >
                    { if *is_loading { "Redirecting..." } else { "Login with PeeringDB OAuth" } }
                </button>
                
                {if !message.is_empty() {
                    html! {
                        <div class="message">
                            { (*message).clone() }
                        </div>
                    }
                } else {
                    html! {}
                }}
                
                <p class="note">
                    { "You will be redirected to PeeringDB's secure login page." }
                </p>
            </div>
        </div>
    }
}
