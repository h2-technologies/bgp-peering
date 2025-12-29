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
        Route::PeeringSessions => html! { <site::peering_sessions::PeeringSessions /> },
        Route::PeeringSetup => html! { <site::peering_setup::PeeringSetup /> },
        Route::PNI => html! { <site::pni::PNI /> },
        Route::AllPeeringLocations => html! { <site::all_peering_locations::AllPeeringLocations /> },
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