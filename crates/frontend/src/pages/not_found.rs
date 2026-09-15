use yew::prelude::*;
use yew_router::prelude::*;

use crate::Route;

#[function_component(NotFound)]
pub fn not_found() -> Html {
    html! {
        <section class="panel">
            <h1>{ "404" }</h1>
            <p class="muted">{ "No route here." }</p>
            <Link<Route> to={Route::Home} classes="btn btn-quiet">{ "Back home" }</Link<Route>>
        </section>
    }
}
