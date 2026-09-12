//! Approval queue for users affiliated with one of our own ASNs.

use shared::{PeeringRequest, RequestStatus};
use wasm_bindgen_futures::spawn_local;
use web_sys::HtmlInputElement;
use yew::prelude::*;

use crate::api::{self, ApiFailure};
use crate::components::status::{Notice, NoticeKind, StatusBadge};
use crate::pages::requests::request_details;
use crate::SessionContext;

#[function_component(Admin)]
pub fn admin() -> Html {
    let session = use_context::<SessionContext>().expect("session context");

    let requests = use_state(Vec::<PeeringRequest>::new);
    let error = use_state(|| None::<String>);
    let denied = use_state(|| false);
    let loading = use_state(|| true);
    let generation = use_state(|| 0_u32);
    let show_closed = use_state(|| false);

    {
        let requests = requests.clone();
        let error = error.clone();
        let denied = denied.clone();
        let loading = loading.clone();

        use_effect_with(*generation, move |_| {
            loading.set(true);
            spawn_local(async move {
                match api::all_requests().await {
                    Ok(found) => {
                        requests.set(found);
                        denied.set(false);
                    }
                    Err(ApiFailure::Unauthorized) => denied.set(true),
                    Err(failure) => error.set(Some(failure.message())),
                }
                loading.set(false);
            });
        });
    }

    let on_decide = {
        let generation = generation.clone();
        let error = error.clone();
        Callback::from(move |(id, status, note): (String, RequestStatus, Option<String>)| {
            let generation = generation.clone();
            let error = error.clone();
            spawn_local(async move {
                match api::decide_request(&id, status, note).await {
                    Ok(_) => generation.set(*generation + 1),
                    Err(failure) => error.set(Some(failure.message())),
                }
            });
        })
    };

    if !session.is_admin() && !*loading {
        return html! {
            <section class="panel">
                <h1>{ "Queue" }</h1>
                <Notice kind={NoticeKind::Info} message={"This page is for people affiliated with our ASNs in PeeringDB."} />
            </section>
        };
    }

    let visible: Vec<&PeeringRequest> = requests
        .iter()
        .filter(|request| *show_closed || request.status.is_open())
        .collect();

    let toggle_closed = {
        let show_closed = show_closed.clone();
        Callback::from(move |_: MouseEvent| show_closed.set(!*show_closed))
    };

    html! {
        <section class="panel">
            <div class="panel-head">
                <h1>{ "Queue" }</h1>
                <button class="btn btn-small btn-quiet" onclick={toggle_closed}>
                    { if *show_closed { "Hide closed" } else { "Show closed" } }
                </button>
            </div>

            if *denied {
                <Notice kind={NoticeKind::Error} message={"Your PeeringDB affiliations do not include one of our ASNs."} />
            }
            if let Some(message) = error.as_ref() {
                <Notice kind={NoticeKind::Error} message={message.clone()} />
            }

            if *loading {
                <p class="muted">{ "Loading…" }</p>
            } else if visible.is_empty() {
                <Notice kind={NoticeKind::Info} message={if *show_closed { "No requests at all yet." } else { "Nothing waiting." }.to_owned()} />
            } else {
                <div class="request-list">
                    { for visible.iter().map(|request| html! {
                        <AdminRow
                            key={request.id.clone()}
                            request={(*request).clone()}
                            on_decide={on_decide.clone()}
                        />
                    }) }
                </div>
            }
        </section>
    }
}

#[derive(Properties, PartialEq)]
struct AdminRowProps {
    request: PeeringRequest,
    on_decide: Callback<(String, RequestStatus, Option<String>)>,
}

#[function_component(AdminRow)]
fn admin_row(props: &AdminRowProps) -> Html {
    let request = &props.request;
    let note = use_state(|| request.decision_note.clone().unwrap_or_default());

    let decide = {
        let on_decide = props.on_decide.clone();
        let id = request.id.clone();
        let note = note.clone();

        move |status: RequestStatus| {
            let on_decide = on_decide.clone();
            let id = id.clone();
            let note = note.clone();
            Callback::from(move |_: MouseEvent| {
                let trimmed = note.trim().to_owned();
                on_decide.emit((
                    id.clone(),
                    status,
                    (!trimmed.is_empty()).then_some(trimmed),
                ));
            })
        }
    };

    let on_note = {
        let note = note.clone();
        Callback::from(move |event: InputEvent| {
            note.set(event.target_unchecked_into::<HtmlInputElement>().value());
        })
    };

    html! {
        <article class="request">
            <header>
                <h3>
                    { &request.location_name }
                    <span class="muted">{ format!(" · {}", request.kind.label()) }</span>
                </h3>
                <StatusBadge status={request.status} />
            </header>

            { request_details(request) }

            <p class="muted">
                { format!("Filed by {} ({})", request.requested_by_name, request.requested_by) }
            </p>

            if request.status.is_open() {
                <div class="decision">
                    <input
                        type="text"
                        placeholder="Note to the requester (optional)"
                        value={(*note).clone()}
                        oninput={on_note}
                    />
                    <div class="form-actions">
                        if matches!(request.status, RequestStatus::Pending) {
                            <button class="btn btn-small btn-primary"
                                    onclick={decide(RequestStatus::Approved)}>
                                { "Approve" }
                            </button>
                            <button class="btn btn-small btn-danger"
                                    onclick={decide(RequestStatus::Declined)}>
                                { "Decline" }
                            </button>
                        }
                        if matches!(request.status, RequestStatus::Approved) {
                            <button class="btn btn-small btn-primary"
                                    onclick={decide(RequestStatus::Provisioned)}>
                                { "Mark provisioned" }
                            </button>
                            <button class="btn btn-small btn-danger"
                                    onclick={decide(RequestStatus::Declined)}>
                                { "Decline" }
                            </button>
                        }
                    </div>
                </div>
            }
        </article>
    }
}
