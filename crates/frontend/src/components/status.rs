//! Small shared presentation bits.

use shared::RequestStatus;
use yew::prelude::*;

#[derive(Properties, PartialEq)]
pub struct StatusBadgeProps {
    pub status: RequestStatus,
}

#[function_component(StatusBadge)]
pub fn status_badge(props: &StatusBadgeProps) -> Html {
    let modifier = match props.status {
        RequestStatus::Pending => "pending",
        RequestStatus::Approved => "approved",
        RequestStatus::Declined => "declined",
        RequestStatus::Provisioned => "provisioned",
        RequestStatus::Withdrawn => "withdrawn",
    };

    html! {
        <span class={classes!("badge", format!("badge-{modifier}"))}>
            { props.status.label() }
        </span>
    }
}

/// A full-width message panel, used for errors and empty states.
#[derive(Properties, PartialEq)]
pub struct NoticeProps {
    pub message: AttrValue,
    #[prop_or_default]
    pub kind: NoticeKind,
}

#[derive(Clone, Copy, PartialEq, Eq, Default)]
pub enum NoticeKind {
    #[default]
    Info,
    Error,
}

#[function_component(Notice)]
pub fn notice(props: &NoticeProps) -> Html {
    let modifier = match props.kind {
        NoticeKind::Info => "notice-info",
        NoticeKind::Error => "notice-error",
    };

    html! {
        <div class={classes!("notice", modifier)}>
            { &props.message }
        </div>
    }
}

/// Render a port speed the way network engineers write it.
pub fn format_speed(mbits: u64) -> String {
    match mbits {
        0 => "unspecified".to_owned(),
        speed if speed >= 1_000_000 => format!("{}T", speed / 1_000_000),
        speed if speed >= 1_000 => format!("{}G", speed / 1_000),
        speed => format!("{speed}M"),
    }
}
