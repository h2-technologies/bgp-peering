use yew::prelude::*;
use yew_router::prelude::*;

mod components;
mod pages;
mod api;

use pages::{Home, Login, LocationMatch};

#[derive(Clone, Routable, PartialEq)]
enum Route {
    #[at("/")]
    Home,
    #[at("/login")]
    Login,
    #[at("/match")]
    LocationMatch,
}

fn switch(routes: Route) -> Html {
    match routes {
        Route::Home => html! { <Home /> },
        Route::Login => html! { <Login /> },
        Route::LocationMatch => html! { <LocationMatch /> },
    }
}

#[function_component(App)]
fn app() -> Html {
    html! {
        <BrowserRouter>
            <div class="app">
                <header class="header">
                    <h1>{ "BGP Peering Portal" }</h1>
                    <nav>
                        <Link<Route> to={Route::Home}>{ "Home" }</Link<Route>>
                        <Link<Route> to={Route::Login}>{ "Login" }</Link<Route>>
                        <Link<Route> to={Route::LocationMatch}>{ "Check Locations" }</Link<Route>>
                    </nav>
                </header>
                <main class="main-content">
                    <Switch<Route> render={switch} />
                </main>
                <footer class="footer">
                    <p>{ "BGP Peering Portal - Powered by PeeringDB" }</p>
                </footer>
            </div>
        </BrowserRouter>
    }
}

fn main() {
    yew::Renderer::<App>::new().render();
}
