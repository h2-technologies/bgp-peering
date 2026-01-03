use yew::prelude::*;
use yew_router::prelude::*;
use yew::classes;
use crate::Route;

#[function_component(Header)]
pub fn header() -> Html {
    html! {
    <>
        <style>
            { header_style() }
        </style>
        <header class={classes!("header")}> 
            <nav class={classes!("header-nav")}> 
                <div class={classes!("header-left")}>
                    <Link<Route> to={Route::Home}>
                        { "H2 Technologies Peering Portal" }
                    </Link<Route>>
                </div>
                <ul class={classes!("header-links")}> 
                    <li><Link<Route> to={Route::Home}>{ "Home" }</Link<Route>></li>
                    <li><Link<Route> to={Route::PeeringSessions}>{ "Peering Sessions" }</Link<Route>></li>
                    <li><Link<Route> to={Route::PeeringSetup}>{ "Peering Setup" }</Link<Route>></li>
                    <li><Link<Route> to={Route::PNI}>{ "PNI" }</Link<Route>></li>
                    <li><Link<Route> to={Route::AllPeeringLocations}>{ "All Peering Locations" }</Link<Route>></li>
                </ul>
            </nav>
        </header>
    </>
    }
}

/// Returns a string containing CSS styles for the header.
fn header_style() -> String {
    r#"
    .header {
        background-color: #222;
        color: #fff;
        padding: 1.5rem 0;
        box-shadow: 0 2px 4px rgba(0,0,0,0.05);
    }
    .header-nav {
        display: flex;
        align-items: center;
        justify-content: flex-start;
        max-width: 900px;
        margin: 0 auto;
        padding: 0 2rem;
    }
    .header-left {
        flex: 0 0 auto;
        font-size: 1.5rem;
        font-weight: bold;
        margin-right: 3rem;
    }
    .header-links {
        flex: 1 1 auto;
        list-style: none;
        display: flex;
        gap: 2rem;
        margin: 0;
        padding: 0;
        justify-content: flex-end;
    }
    .header-left {
        font-size: 1.5rem;
        font-weight: bold;
    }
    .header-left a {
        color: #ffd700;
        text-decoration: none;
        font-weight: bold;
        letter-spacing: 2px;
        transition: color 0.2s;
    }
    .header-left a:hover {
        color: #fff;
    }
    .header-links {
        list-style: none;
        display: flex;
        gap: 2rem;
        margin: 0;
        padding: 0;
        justify-content: center;
    }
    .header-links li {
        display: inline;
        font-size: 1.2rem;
    }
    .header-links li a {
        color: #fff;
        text-decoration: none;
        font-weight: 500;
        transition: color 0.2s;
    }
    .header-links li a:hover {
        color: #ffd700;
    }
    "#.to_string()
}