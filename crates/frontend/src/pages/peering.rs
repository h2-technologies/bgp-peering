//! Find where we overlap with a given ASN, and file a request from the result.

use shared::{Asn, NewPeeringRequest, OverlapReport, PeeringKind};
use wasm_bindgen_futures::spawn_local;
use web_sys::{HtmlInputElement, HtmlSelectElement, HtmlTextAreaElement};
use yew::prelude::*;

use crate::api::{self, ApiFailure};
use crate::components::status::{Notice, NoticeKind, format_speed};
use crate::{Session, SessionContext};

/// A location the user picked, carrying everything the form needs to prefill.
#[derive(Clone, PartialEq)]
struct Selection {
    kind: PeeringKind,
    location_id: u64,
    location_name: String,
    /// Only the ASNs of ours actually present at this location.
    local_options: Vec<Asn>,
    peer_ipaddr4: Option<String>,
    peer_ipaddr6: Option<String>,
}

#[function_component(Peering)]
pub fn peering() -> Html {
    let session = use_context::<SessionContext>().expect("session context");

    let asn_input = use_state(String::new);
    let report = use_state(|| None::<OverlapReport>);
    let error = use_state(|| None::<String>);
    let needs_login = use_state(|| false);
    let loading = use_state(|| false);
    let selection = use_state(|| None::<Selection>);
    let confirmation = use_state(|| None::<String>);

    // Once we know who the user is, offer their own ASN as the default.
    {
        let asn_input = asn_input.clone();
        let default_asn = session
            .user()
            .and_then(|user| user.affiliations.first())
            .map(|affiliation| affiliation.asn.0);

        use_effect_with(default_asn, move |default_asn| {
            if let Some(asn) = default_asn {
                if asn_input.is_empty() {
                    asn_input.set(asn.to_string());
                }
            }
        });
    }

    let on_input = {
        let asn_input = asn_input.clone();
        Callback::from(move |event: InputEvent| {
            asn_input.set(event.target_unchecked_into::<HtmlInputElement>().value());
        })
    };

    let on_search = {
        let asn_input = asn_input.clone();
        let report = report.clone();
        let error = error.clone();
        let needs_login = needs_login.clone();
        let loading = loading.clone();
        let selection = selection.clone();
        let confirmation = confirmation.clone();

        Callback::from(move |event: SubmitEvent| {
            event.prevent_default();

            let Ok(asn) = asn_input.parse::<Asn>() else {
                error.set(Some(
                    "That does not look like an ASN. Try something like 13335 or AS13335."
                        .to_owned(),
                ));
                return;
            };

            let report = report.clone();
            let error = error.clone();
            let needs_login = needs_login.clone();
            let loading = loading.clone();
            selection.set(None);
            confirmation.set(None);
            error.set(None);
            needs_login.set(false);
            loading.set(true);

            spawn_local(async move {
                match api::overlap(asn.0).await {
                    Ok(found) => report.set(Some(found)),
                    Err(ApiFailure::Unauthorized) => needs_login.set(true),
                    Err(failure) => error.set(Some(failure.message())),
                }
                loading.set(false);
            });
        })
    };

    let on_pick = {
        let selection = selection.clone();
        let confirmation = confirmation.clone();
        Callback::from(move |picked: Selection| {
            confirmation.set(None);
            selection.set(Some(picked));
        })
    };

    let on_cancel = {
        let selection = selection.clone();
        Callback::from(move |()| selection.set(None))
    };

    let on_submitted = {
        let selection = selection.clone();
        let confirmation = confirmation.clone();
        Callback::from(move |message: String| {
            selection.set(None);
            confirmation.set(Some(message));
        })
    };

    html! {
        <>
            <section class="panel">
                <h1>{ "Where can we peer?" }</h1>
                <p class="muted">
                    { "Enter an ASN and we'll compare its PeeringDB presence with ours." }
                </p>

                <form class="search" onsubmit={on_search}>
                    <input
                        type="text"
                        inputmode="numeric"
                        placeholder="AS13335"
                        aria-label="ASN"
                        value={(*asn_input).clone()}
                        oninput={on_input}
                    />
                    <button class="btn btn-primary" type="submit" disabled={*loading}>
                        { if *loading { "Looking…" } else { "Find overlap" } }
                    </button>
                </form>

                if *needs_login {
                    <Notice kind={NoticeKind::Info} message={"Sign in with PeeringDB to look up overlap."} />
                }
                if let Some(message) = error.as_ref() {
                    <Notice kind={NoticeKind::Error} message={message.clone()} />
                }
                if let Some(message) = confirmation.as_ref() {
                    <Notice kind={NoticeKind::Info} message={message.clone()} />
                }
            </section>

            if let Some(found) = report.as_ref() {
                { render_report(found, &session, &on_pick) }

                if let Some(picked) = selection.as_ref() {
                    <RequestForm
                        key={picked.location_id}
                        peer_asn={found.peer.asn}
                        selection={picked.clone()}
                        default_prefixes4={found.peer.info_prefixes4}
                        default_prefixes6={found.peer.info_prefixes6}
                        on_cancel={on_cancel.clone()}
                        on_submitted={on_submitted.clone()}
                    />
                }
            }
        </>
    }
}

fn render_report(report: &OverlapReport, session: &Session, on_pick: &Callback<Selection>) -> Html {
    let peer = &report.peer;
    let can_request = session
        .user()
        .is_some_and(|user| user.is_admin || user.may_act_for(peer.asn));

    html! {
        <>
            <section class="panel">
                <h2>
                    { &peer.name }
                    <span class="asn">{ peer.asn.to_string() }</span>
                </h2>
                <dl class="inline-dl">
                    <dt>{ "Policy" }</dt>
                    <dd>{ peer.policy_general.clone().unwrap_or_else(|| "Unknown".to_owned()) }</dd>
                    <dt>{ "Traffic" }</dt>
                    <dd>{ peer.info_traffic.clone().unwrap_or_else(|| "Unknown".to_owned()) }</dd>
                    <dt>{ "Ratio" }</dt>
                    <dd>{ peer.info_ratio.clone().unwrap_or_else(|| "Unknown".to_owned()) }</dd>
                    <dt>{ "Scope" }</dt>
                    <dd>{ peer.info_scope.clone().unwrap_or_else(|| "Unknown".to_owned()) }</dd>
                </dl>

                if !can_request {
                    <Notice kind={NoticeKind::Info} message={"You are not affiliated with this ASN in PeeringDB, so you can \
                           browse the overlap but not file a request for it."} />
                }
            </section>

            if report.is_empty() {
                <section class="panel">
                    <Notice kind={NoticeKind::Info} message={"We have no exchange or facility in common according to \
                           PeeringDB. If that looks wrong, check that both networks' \
                           netixlan and netfac records are up to date."} />
                </section>
            }

            if !report.exchanges.is_empty() {
                <section class="panel">
                    <h2>{ format!("{} shared exchange(s)", report.exchanges.len()) }</h2>
                    <div class="table-scroll">
                        <table class="grid">
                            <thead>
                                <tr>
                                    <th>{ "Exchange" }</th>
                                    <th>{ "Us" }</th>
                                    <th>{ "Them" }</th>
                                    <th class="right"></th>
                                </tr>
                            </thead>
                            <tbody>
                                { for report.exchanges.iter().map(|exchange| {
                                    let theirs = exchange.theirs.first();
                                    let selection = Selection {
                                        kind: PeeringKind::PublicExchange,
                                        location_id: exchange.ix_id,
                                        location_name: exchange.ix_name.clone(),
                                        local_options: exchange.ours.iter().map(|p| p.asn).collect(),
                                        peer_ipaddr4: theirs.and_then(|p| p.ipaddr4.clone()),
                                        peer_ipaddr6: theirs.and_then(|p| p.ipaddr6.clone()),
                                    };
                                    let on_pick = on_pick.clone();
                                    let pick = Callback::from(move |_: MouseEvent| {
                                        on_pick.emit(selection.clone());
                                    });

                                    html! {
                                        <tr key={exchange.ix_id}>
                                            <td class="strong">{ &exchange.ix_name }</td>
                                            <td>{ render_presences(&exchange.ours) }</td>
                                            <td>{ render_presences(&exchange.theirs) }</td>
                                            <td class="right">
                                                if can_request {
                                                    <button class="btn btn-small" onclick={pick}>
                                                        { "Request" }
                                                    </button>
                                                }
                                            </td>
                                        </tr>
                                    }
                                }) }
                            </tbody>
                        </table>
                    </div>
                </section>
            }

            if !report.facilities.is_empty() {
                <section class="panel">
                    <h2>{ format!("{} shared facility(ies)", report.facilities.len()) }</h2>
                    <p class="muted">{ "Candidates for a private interconnect." }</p>
                    <div class="card-grid">
                        { for report.facilities.iter().map(|facility| {
                            let selection = Selection {
                                kind: PeeringKind::PrivateInterconnect,
                                location_id: facility.fac_id,
                                location_name: facility.fac_name.clone(),
                                local_options: facility.our_asns.clone(),
                                peer_ipaddr4: None,
                                peer_ipaddr6: None,
                            };
                            let on_pick = on_pick.clone();
                            let pick = Callback::from(move |_: MouseEvent| {
                                on_pick.emit(selection.clone());
                            });

                            let where_ = [facility.city.clone(), facility.country.clone()]
                                .into_iter()
                                .flatten()
                                .collect::<Vec<_>>()
                                .join(", ");

                            html! {
                                <article class="card" key={facility.fac_id}>
                                    <h3>{ &facility.fac_name }</h3>
                                    if !where_.is_empty() {
                                        <p class="muted">{ where_ }</p>
                                    }
                                    <p class="asn-list">
                                        { for facility.our_asns.iter().map(|asn| html! {
                                            <span class="pill">{ asn.to_string() }</span>
                                        }) }
                                    </p>
                                    if can_request {
                                        <button class="btn btn-small" onclick={pick}>
                                            { "Request PNI" }
                                        </button>
                                    }
                                </article>
                            }
                        }) }
                    </div>
                </section>
            }
        </>
    }
}

fn render_presences(presences: &[shared::IxPresence]) -> Html {
    html! {
        <ul class="presence">
            { for presences.iter().map(|presence| html! {
                <li key={presence.asn.0}>
                    <span class="pill">{ presence.asn.to_string() }</span>
                    <span class="mono">
                        { presence.ipaddr4.clone().unwrap_or_else(|| "—".to_owned()) }
                    </span>
                    <span class="mono">
                        { presence.ipaddr6.clone().unwrap_or_else(|| "—".to_owned()) }
                    </span>
                    <span class="muted">{ format_speed(presence.speed) }</span>
                    if presence.is_rs_peer {
                        <span class="pill pill-quiet" title="Peers with the route servers">
                            { "RS" }
                        </span>
                    }
                </li>
            }) }
        </ul>
    }
}

#[derive(Properties, PartialEq, Clone)]
struct RequestFormProps {
    peer_asn: Asn,
    selection: Selection,
    default_prefixes4: Option<u32>,
    default_prefixes6: Option<u32>,
    on_cancel: Callback<()>,
    on_submitted: Callback<String>,
}

#[function_component(RequestForm)]
fn request_form(props: &RequestFormProps) -> Html {
    let selection = &props.selection;

    let local_asn = use_state(|| {
        selection
            .local_options
            .first()
            .copied()
            .unwrap_or(Asn(0))
            .0
    });
    let ipaddr4 = use_state(|| selection.peer_ipaddr4.clone().unwrap_or_default());
    let ipaddr6 = use_state(|| selection.peer_ipaddr6.clone().unwrap_or_default());
    let prefixes4 = use_state(|| props.default_prefixes4.map(|n| n.to_string()).unwrap_or_default());
    let prefixes6 = use_state(|| props.default_prefixes6.map(|n| n.to_string()).unwrap_or_default());
    let notes = use_state(String::new);
    let error = use_state(|| None::<String>);
    let submitting = use_state(|| false);

    let on_submit = {
        let props = props.clone();
        let local_asn = local_asn.clone();
        let ipaddr4 = ipaddr4.clone();
        let ipaddr6 = ipaddr6.clone();
        let prefixes4 = prefixes4.clone();
        let prefixes6 = prefixes6.clone();
        let notes = notes.clone();
        let error = error.clone();
        let submitting = submitting.clone();

        Callback::from(move |event: SubmitEvent| {
            event.prevent_default();

            if *submitting {
                return;
            }

            let body = NewPeeringRequest {
                peer_asn: props.peer_asn,
                local_asn: Asn(*local_asn),
                kind: Some(props.selection.kind),
                location_id: props.selection.location_id,
                peer_ipaddr4: optional(&ipaddr4),
                peer_ipaddr6: optional(&ipaddr6),
                max_prefixes4: optional(&prefixes4).and_then(|v| v.parse().ok()),
                max_prefixes6: optional(&prefixes6).and_then(|v| v.parse().ok()),
                notes: optional(&notes),
            };

            let location = props.selection.location_name.clone();
            let on_submitted = props.on_submitted.clone();
            let error = error.clone();
            let submitting = submitting.clone();

            submitting.set(true);
            error.set(None);

            spawn_local(async move {
                match api::submit_request(&body).await {
                    Ok(_) => on_submitted.emit(format!(
                        "Request sent for {location}. You can track it on the \
                         My requests page."
                    )),
                    Err(failure) => error.set(Some(failure.message())),
                }
                submitting.set(false);
            });
        })
    };

    let cancel = {
        let on_cancel = props.on_cancel.clone();
        Callback::from(move |_: MouseEvent| on_cancel.emit(()))
    };

    let set_local_asn = {
        let local_asn = local_asn.clone();
        Callback::from(move |event: Event| {
            let value = event.target_unchecked_into::<HtmlSelectElement>().value();
            if let Ok(parsed) = value.parse::<u32>() {
                local_asn.set(parsed);
            }
        })
    };

    html! {
        <section class="panel form-panel">
            <h2>
                { format!("{} at {}", selection.kind.label(), selection.location_name) }
            </h2>

            <form onsubmit={on_submit}>
                <div class="field-row">
                    <label class="field">
                        <span>{ "Peer with" }</span>
                        <select onchange={set_local_asn}>
                            { for selection.local_options.iter().map(|asn| html! {
                                <option key={asn.0} value={asn.0.to_string()}
                                        selected={asn.0 == *local_asn}>
                                    { asn.to_string() }
                                </option>
                            }) }
                        </select>
                    </label>

                    <label class="field">
                        <span>{ "Your ASN" }</span>
                        <input type="text" readonly=true value={props.peer_asn.to_string()} />
                    </label>
                </div>

                if matches!(selection.kind, PeeringKind::PublicExchange) {
                    <div class="field-row">
                        { text_field("Your IPv4 on this fabric", "198.51.100.1", &ipaddr4) }
                        { text_field("Your IPv6 on this fabric", "2001:db8::1", &ipaddr6) }
                    </div>
                }

                <div class="field-row">
                    { text_field("Max IPv4 prefixes", "1000", &prefixes4) }
                    { text_field("Max IPv6 prefixes", "100", &prefixes6) }
                </div>

                <label class="field">
                    <span>{ "Notes" }</span>
                    <textarea
                        rows="3"
                        placeholder="Anything we should know — NOC contact, timing, MD5, LOA for a cross connect…"
                        value={(*notes).clone()}
                        oninput={{
                            let notes = notes.clone();
                            Callback::from(move |event: InputEvent| {
                                notes.set(
                                    event.target_unchecked_into::<HtmlTextAreaElement>().value(),
                                );
                            })
                        }}
                    />
                </label>

                if let Some(message) = error.as_ref() {
                    <Notice kind={NoticeKind::Error} message={message.clone()} />
                }

                <div class="form-actions">
                    <button class="btn btn-primary" type="submit" disabled={*submitting}>
                        { if *submitting { "Sending…" } else { "Send request" } }
                    </button>
                    <button class="btn btn-quiet" type="button" onclick={cancel}>
                        { "Cancel" }
                    </button>
                </div>
            </form>
        </section>
    }
}

fn text_field(label: &str, placeholder: &str, state: &UseStateHandle<String>) -> Html {
    let oninput = {
        let state = state.clone();
        Callback::from(move |event: InputEvent| {
            state.set(event.target_unchecked_into::<HtmlInputElement>().value());
        })
    };

    html! {
        <label class="field">
            <span>{ label }</span>
            <input
                type="text"
                placeholder={placeholder.to_owned()}
                value={(**state).clone()}
                {oninput}
            />
        </label>
    }
}

/// Treat a blank field as "not supplied".
fn optional(state: &UseStateHandle<String>) -> Option<String> {
    let trimmed = state.trim();
    (!trimmed.is_empty()).then(|| trimmed.to_owned())
}
