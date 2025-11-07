use yew::prelude::*;
use yew_router::prelude::*;
mod site;

#[derive(Clone, Routable, PartialEq)]
enum Route {
    #[at("/")]
    Home,
    #[at("/peering-sessions")]
    PeeringSessions,
    #[at("/peering-setup")]
    PeeringSetup,
    #[at("/pni")]
    PNI,
    #[at("/all-peering-locations")]
    AllPeeringLocations,
    #[not_found]
    #[at("/404")]
    NotFound,
}


fn switch(routes: Route) -> Html {
    match routes {
        Route::Home => html! { <site::home::Home /> },
        Route::About => html! { <site::about::About /> },
        Route::Contact => html! { <site::contact::Contact /> },
        Route::Pricing => html! { <site::pricing::Pricing /> },
        Route::Policy => html! {
            <iframe src="/Amateur_Virtual_Internet_Exchange_Policy_V1.0.pdf" width="100%" height="800px" title="Exchange Policy PDF" />
        },
        Route::Peers => html! { <site::peers::Peers /> },
        Route::NotFound => html! { <h1>{ "404" }</h1> },
    }
}

#[function_component(Main)]
fn app() -> Html {
    html! {
        <BrowserRouter>
            <Switch<Route> render={switch} /> // <- must be child of <BrowserRouter>
        </BrowserRouter>
    }
}

fn main() {
    yew::Renderer::<Main>::new().render();
}