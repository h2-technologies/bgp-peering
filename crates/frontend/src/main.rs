//! Yew single-page app for the peering portal.

mod api;
mod components;
mod pages;

use shared::{CurrentUser, SiteInfo};
use yew::prelude::*;
use yew_router::prelude::*;

#[derive(Clone, Routable, PartialEq)]
pub enum Route {
    #[at("/")]
    Home,
    #[at("/peering")]
    Peering,
    #[at("/requests")]
    Requests,
    #[at("/admin")]
    Admin,
    #[not_found]
    #[at("/404")]
    NotFound,
}

/// Login state, resolved once on boot.
#[derive(Clone, PartialEq)]
pub enum Session {
    Loading,
    Anonymous,
    SignedIn(CurrentUser),
}

impl Session {
    pub fn user(&self) -> Option<&CurrentUser> {
        match self {
            Self::SignedIn(user) => Some(user),
            _ => None,
        }
    }

    pub fn is_admin(&self) -> bool {
        self.user().is_some_and(|user| user.is_admin)
    }
}

pub type SessionContext = UseStateHandle<Session>;
pub type SiteContext = UseStateHandle<Option<SiteInfo>>;

fn switch(route: Route) -> Html {
    match route {
        Route::Home => html! { <pages::home::Home /> },
        Route::Peering => html! { <pages::peering::Peering /> },
        Route::Requests => html! { <pages::requests::MyRequests /> },
        Route::Admin => html! { <pages::admin::Admin /> },
        Route::NotFound => html! { <pages::not_found::NotFound /> },
    }
}

#[function_component(App)]
fn app() -> Html {
    let session = use_state(|| Session::Loading);
    let site = use_state(|| None::<SiteInfo>);

    // Resolve both once, at startup. Everything below reads them from context.
    {
        let session = session.clone();
        let site = site.clone();
        use_effect_with((), move |()| {
            wasm_bindgen_futures::spawn_local(async move {
                let resolved = match api::me().await {
                    Ok(Some(user)) => Session::SignedIn(user),
                    // A failure here is indistinguishable from being signed
                    // out as far as the UI is concerned.
                    Ok(None) | Err(_) => Session::Anonymous,
                };
                session.set(resolved);
            });

            wasm_bindgen_futures::spawn_local(async move {
                if let Ok(info) = api::site_info().await {
                    site.set(Some(info));
                }
            });
        });
    }

    html! {
        <ContextProvider<SessionContext> context={session}>
            <ContextProvider<SiteContext> context={site}>
                <BrowserRouter>
                    <components::nav::Nav />
                    <main class="page">
                        <Switch<Route> render={switch} />
                    </main>
                    <components::nav::Footer />
                </BrowserRouter>
            </ContextProvider<SiteContext>>
        </ContextProvider<SessionContext>>
    }
}

fn main() {
    yew::Renderer::<App>::new().render();
}
