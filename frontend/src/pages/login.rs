use yew::prelude::*;
use web_sys::HtmlInputElement;
use wasm_bindgen_futures::spawn_local;
use crate::api;

#[function_component(Login)]
pub fn login() -> Html {
    let username = use_state(|| String::new());
    let password = use_state(|| String::new());
    let message = use_state(|| String::new());
    let is_loading = use_state(|| false);
    
    let username_clone = username.clone();
    let on_username_change = Callback::from(move |e: Event| {
        let input: HtmlInputElement = e.target_unchecked_into();
        username_clone.set(input.value());
    });
    
    let password_clone = password.clone();
    let on_password_change = Callback::from(move |e: Event| {
        let input: HtmlInputElement = e.target_unchecked_into();
        password_clone.set(input.value());
    });
    
    let username_value = (*username).clone();
    let password_value = (*password).clone();
    let message_setter = message.clone();
    let loading_setter = is_loading.clone();
    
    let on_submit = Callback::from(move |e: SubmitEvent| {
        e.prevent_default();
        
        let username = username_value.clone();
        let password = password_value.clone();
        let message = message_setter.clone();
        let loading = loading_setter.clone();
        
        loading.set(true);
        
        spawn_local(async move {
            match api::authenticate(username, password).await {
                Ok(response) => {
                    if response.success {
                        message.set(format!("✓ {}", response.message));
                        // Store token in local storage
                        if let Some(token) = response.token {
                            if let Some(window) = web_sys::window() {
                                if let Ok(Some(storage)) = window.local_storage() {
                                    let _ = storage.set_item("peeringdb_token", &token);
                                }
                            }
                        }
                    } else {
                        message.set(format!("✗ {}", response.message));
                    }
                }
                Err(e) => {
                    message.set(format!("✗ Error: {}", e));
                }
            }
            loading.set(false);
        });
    });
    
    html! {
        <div class="login-page">
            <div class="login-container">
                <h2>{ "Login with PeeringDB" }</h2>
                <p class="login-description">
                    { "Enter your PeeringDB credentials to access the portal" }
                </p>
                
                <form onsubmit={on_submit}>
                    <div class="form-group">
                        <label for="username">{ "Username:" }</label>
                        <input
                            type="text"
                            id="username"
                            placeholder="Enter your PeeringDB username"
                            onchange={on_username_change}
                            disabled={*is_loading}
                        />
                    </div>
                    
                    <div class="form-group">
                        <label for="password">{ "Password:" }</label>
                        <input
                            type="password"
                            id="password"
                            placeholder="Enter your password"
                            onchange={on_password_change}
                            disabled={*is_loading}
                        />
                    </div>
                    
                    <button type="submit" disabled={*is_loading}>
                        { if *is_loading { "Logging in..." } else { "Login" } }
                    </button>
                </form>
                
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
                    { "Note: This is a demonstration. In production, use OAuth2 authentication." }
                </p>
            </div>
        </div>
    }
}
