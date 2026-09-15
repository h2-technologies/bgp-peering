//! Site header and footer.

use yew::prelude::*;
use yew_router::prelude::*;

use crate::{Route, Session, SessionContext, SiteContext, api};

/// Login is a server round trip, not a client route, so it has to be a plain
/// anchor rather than a `<Link>`.
fn login_href() -> String {
    let next = web_sys::window()
        .and_then(|window| window.location().pathname().ok())
        .unwrap_or_else(|| "/".to_owned());
    format!("/auth/login?next={next}")
}

#[function_component(Nav)]
pub fn nav() -> Html {
    let session = use_context::<SessionContext>().expect("session context");
    let site = use_context::<SiteContext>().expect("site context");

    let login_enabled = site
        .as_ref()
        .map(|info| info.login_enabled)
        .unwrap_or(false);

    let on_logout = {
        let session = session.clone();
        Callback::from(move |_: MouseEvent| {
            let session = session.clone();
            wasm_bindgen_futures::spawn_local(async move {
                let _ = api::logout().await;
                session.set(Session::Anonymous);
                // Full reload so no stale per-user data survives the sign-out.
                if let Some(window) = web_sys::window() {
                    let _ = window.location().set_href("/");
                }
            });
        })
    };

    html! {
        <header class="topbar">
            <Link<Route> to={Route::Home} classes="brand">
                <span class="brand-mark">{ "AS" }</span>
                <span>{ "Peering Portal" }</span>
            </Link<Route>>

            <nav class="topbar-links">
                <Link<Route> to={Route::Peering}>{ "Find peering" }</Link<Route>>
                if session.user().is_some() {
                    <Link<Route> to={Route::Requests}>{ "My requests" }</Link<Route>>
                }
                if session.is_admin() {
                    <Link<Route> to={Route::Admin}>{ "Queue" }</Link<Route>>
                }
            </nav>

            <div class="topbar-user">
                {
                    match &*session {
                        Session::Loading => html! { <span class="muted">{ "Checking session…" }</span> },
                        Session::SignedIn(user) => html! {
                            <>
                                <span class="user-name" title={user.username.clone()}>
                                    { &user.display_name }
                                </span>
                                if user.is_admin {
                                    <span class="pill pill-admin">{ "admin" }</span>
                                }
                                <button class="btn btn-quiet" onclick={on_logout}>
                                    { "Sign out" }
                                </button>
                            </>
                        },
                        Session::Anonymous if login_enabled => html! {
                            <a class="btn btn-primary" href={login_href()}>
                                { "Sign in with PeeringDB" }
                            </a>
                        },
                        Session::Anonymous => html! {
                            <span class="muted" title="The server has no OIDC credentials configured.">
                                { "Login unavailable" }
                            </span>
                        },
                    }
                }
            </div>
        </header>
    }
}

#[function_component(Footer)]
pub fn footer() -> Html {
    html! {
        <footer class="footer">
            <span>
                { "Presence data from " }
                <a href="https://www.peeringdb.com/" target="_blank" rel="noreferrer noopener">
                    { "PeeringDB" }
                </a>
                { ". Keep your records current there and this page follows." }
            </span>
        </footer>
    }
}
