//! Landing page: who we are and how to get peered.

use yew::prelude::*;
use yew_router::prelude::*;

use crate::{Route, SessionContext, SiteContext};

#[function_component(Home)]
pub fn home() -> Html {
    let session = use_context::<SessionContext>().expect("session context");
    let site = use_context::<SiteContext>().expect("site context");

    html! {
        <>
            <section class="hero">
                <h1>{ "Let's peer." }</h1>
                <p class="lede">
                    { "Sign in with PeeringDB and we'll show you every exchange and \
                       facility where our networks already meet. Pick one, send a \
                       request, and track it here." }
                </p>
                <div class="hero-actions">
                    <Link<Route> to={Route::Peering} classes="btn btn-primary">
                        { "Find where we meet" }
                    </Link<Route>>
                    if session.user().is_some() {
                        <Link<Route> to={Route::Requests} classes="btn btn-quiet">
                            { "My requests" }
                        </Link<Route>>
                    }
                </div>
            </section>

            <section class="panel">
                <h2>{ "Our networks" }</h2>
                {
                    match site.as_ref() {
                        None => html! { <p class="muted">{ "Loading…" }</p> },
                        Some(info) if info.local_networks.is_empty() => html! {
                            <p class="muted">
                                { "No PeeringDB records are configured yet. Set " }
                                <code>{ "local_asns" }</code>
                                { " and make sure each ASN has a net object in PeeringDB." }
                            </p>
                        },
                        Some(info) => html! {
                            <div class="card-grid">
                                { for info.local_networks.iter().map(|network| html! {
                                    <article class="card" key={network.asn.0}>
                                        <h3>
                                            { &network.name }
                                            <span class="asn">{ network.asn.to_string() }</span>
                                        </h3>
                                        <dl>
                                            <dt>{ "Peering policy" }</dt>
                                            <dd>{ &network.policy_general }</dd>

                                            if let Some(as_set) = &network.irr_as_set {
                                                <dt>{ "IRR AS-SET" }</dt>
                                                <dd><code>{ as_set }</code></dd>
                                            }

                                            if let Some(prefixes) = network.info_prefixes4 {
                                                <dt>{ "IPv4 prefixes" }</dt>
                                                <dd>{ prefixes }</dd>
                                            }
                                            if let Some(prefixes) = network.info_prefixes6 {
                                                <dt>{ "IPv6 prefixes" }</dt>
                                                <dd>{ prefixes }</dd>
                                            }
                                        </dl>
                                        if let Some(url) = &network.policy_url {
                                            <a class="card-link" href={url.clone()}
                                               target="_blank" rel="noreferrer noopener">
                                                { "Peering policy" }
                                            </a>
                                        }
                                    </article>
                                }) }
                            </div>
                        },
                    }
                }
            </section>

            <section class="panel">
                <h2>{ "How it works" }</h2>
                <ol class="steps">
                    <li>
                        <strong>{ "Sign in with PeeringDB." }</strong>
                        { " We use your PeeringDB affiliations to confirm you can \
                            speak for your ASN. There is no separate account." }
                    </li>
                    <li>
                        <strong>{ "We compare presence." }</strong>
                        { " Your netixlan and netfac records are matched against \
                            ours to find every shared fabric and facility." }
                    </li>
                    <li>
                        <strong>{ "Send a request." }</strong>
                        { " Pick a location, confirm your addressing and prefix \
                            limits, and it lands in our queue." }
                    </li>
                    <li>
                        <strong>{ "Track it." }</strong>
                        { " Approvals, declines and provisioning all show up on \
                            your requests page." }
                    </li>
                </ol>
                <p class="muted">
                    { "Keeping your PeeringDB records accurate is the fastest way to \
                       make this go smoothly — everything on this site reads from them." }
                </p>
            </section>
        </>
    }
}
