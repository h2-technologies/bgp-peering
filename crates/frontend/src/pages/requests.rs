//! The signed-in user's own requests.

use shared::PeeringRequest;
use wasm_bindgen_futures::spawn_local;
use yew::prelude::*;

use crate::api::{self, ApiFailure};
use crate::components::status::{Notice, NoticeKind, StatusBadge};
use crate::SessionContext;

#[function_component(MyRequests)]
pub fn my_requests() -> Html {
    let session = use_context::<SessionContext>().expect("session context");

    let requests = use_state(Vec::<PeeringRequest>::new);
    let error = use_state(|| None::<String>);
    let needs_login = use_state(|| false);
    let loading = use_state(|| true);
    // Bumped after a withdrawal to force a refetch.
    let generation = use_state(|| 0_u32);

    {
        let requests = requests.clone();
        let error = error.clone();
        let needs_login = needs_login.clone();
        let loading = loading.clone();

        use_effect_with(*generation, move |_| {
            loading.set(true);
            spawn_local(async move {
                match api::my_requests().await {
                    Ok(found) => {
                        requests.set(found);
                        needs_login.set(false);
                    }
                    Err(ApiFailure::Unauthorized) => needs_login.set(true),
                    Err(failure) => error.set(Some(failure.message())),
                }
                loading.set(false);
            });
        });
    }

    let on_withdraw = {
        let generation = generation.clone();
        let error = error.clone();
        Callback::from(move |id: String| {
            let generation = generation.clone();
            let error = error.clone();
            spawn_local(async move {
                match api::withdraw_request(&id).await {
                    Ok(_) => generation.set(*generation + 1),
                    Err(failure) => error.set(Some(failure.message())),
                }
            });
        })
    };

    if session.user().is_none() && !*loading {
        return html! {
            <section class="panel">
                <h1>{ "My requests" }</h1>
                <Notice kind={NoticeKind::Info} message={"Sign in with PeeringDB to see your requests."} />
            </section>
        };
    }

    html! {
        <section class="panel">
            <h1>{ "My requests" }</h1>

            if let Some(message) = error.as_ref() {
                <Notice kind={NoticeKind::Error} message={message.clone()} />
            }
            if *needs_login {
                <Notice kind={NoticeKind::Info} message={"Sign in with PeeringDB to see your requests."} />
            } else if *loading {
                <p class="muted">{ "Loading…" }</p>
            } else if requests.is_empty() {
                <Notice kind={NoticeKind::Info} message={"Nothing yet. Find an exchange we share and send a request."} />
            } else {
                <div class="request-list">
                    { for requests.iter().map(|request| {
                        let on_withdraw = on_withdraw.clone();
                        let id = request.id.clone();
                        let withdraw = Callback::from(move |_: MouseEvent| {
                            on_withdraw.emit(id.clone());
                        });

                        html! {
                            <article class="request" key={request.id.clone()}>
                                <header>
                                    <h3>
                                        { &request.location_name }
                                        <span class="muted">{ format!(" · {}", request.kind.label()) }</span>
                                    </h3>
                                    <StatusBadge status={request.status} />
                                </header>

                                { request_details(request) }

                                if request.status.is_open() {
                                    <div class="form-actions">
                                        <button class="btn btn-small btn-quiet" onclick={withdraw}>
                                            { "Withdraw" }
                                        </button>
                                    </div>
                                }
                            </article>
                        }
                    }) }
                </div>
            }
        </section>
    }
}

/// Shared between this page and the admin queue.
pub fn request_details(request: &PeeringRequest) -> Html {
    html! {
        <>
            <dl class="inline-dl">
                <dt>{ "Between" }</dt>
                <dd>
                    { format!("{} ({})", request.peer_asn, request.peer_name) }
                    { " ↔ " }
                    { request.local_asn.to_string() }
                </dd>

                if let Some(ip) = &request.peer_ipaddr4 {
                    <dt>{ "Their IPv4" }</dt>
                    <dd class="mono">{ ip }</dd>
                }
                if let Some(ip) = &request.peer_ipaddr6 {
                    <dt>{ "Their IPv6" }</dt>
                    <dd class="mono">{ ip }</dd>
                }
                if let Some(max) = request.max_prefixes4 {
                    <dt>{ "Max v4 prefixes" }</dt>
                    <dd>{ max }</dd>
                }
                if let Some(max) = request.max_prefixes6 {
                    <dt>{ "Max v6 prefixes" }</dt>
                    <dd>{ max }</dd>
                }

                <dt>{ "Filed" }</dt>
                <dd>{ request.created_at.format("%Y-%m-%d %H:%M UTC").to_string() }</dd>
            </dl>

            if let Some(notes) = &request.notes {
                <p class="quote">{ notes }</p>
            }
            if let Some(note) = &request.decision_note {
                <p class="quote quote-decision">
                    <strong>{ "Our note: " }</strong>{ note }
                </p>
            }
        </>
    }
}
