//! Wire types shared by the Rocket backend and the Yew frontend.
//!
//! This crate is compiled for both native and `wasm32-unknown-unknown`, so it
//! must stay free of platform-specific dependencies.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::fmt;

pub mod peeringdb;

pub use peeringdb::{Facility, InternetExchange, Network};

/// An ASN, kept as a distinct type so it is never confused with a PeeringDB
/// object id (the two are both bare integers and are easy to mix up).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(transparent)]
pub struct Asn(pub u32);

impl fmt::Display for Asn {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "AS{}", self.0)
    }
}

impl From<u32> for Asn {
    fn from(value: u32) -> Self {
        Self(value)
    }
}

impl std::str::FromStr for Asn {
    type Err = std::num::ParseIntError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let s = s.trim();
        let digits = s.strip_prefix("AS").or_else(|| s.strip_prefix("as")).unwrap_or(s);
        digits.parse().map(Asn)
    }
}

/// A network the signed-in user is affiliated with, straight from the
/// PeeringDB `networks` OIDC scope.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Affiliation {
    pub asn: Asn,
    pub name: String,
    /// PeeringDB permission bitmask for this network (0x01 read, 0x02 write,
    /// 0x04 create, 0x08 delete). We only ever check it for non-zero.
    #[serde(default)]
    pub perms: u32,
}

/// Who the caller is. Returned by `GET /api/me`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CurrentUser {
    /// PeeringDB user id (the OIDC `sub` claim).
    pub subject: String,
    pub username: String,
    pub display_name: String,
    pub email: Option<String>,
    /// Whether PeeringDB considers this a verified user.
    pub verified: bool,
    /// Every network this user may act for.
    pub affiliations: Vec<Affiliation>,
    /// True when one of `affiliations` is one of our own ASNs, which is what
    /// grants access to the admin queue.
    pub is_admin: bool,
}

impl CurrentUser {
    /// Whether this user is allowed to file or view requests for `asn`.
    pub fn may_act_for(&self, asn: Asn) -> bool {
        self.affiliations.iter().any(|a| a.asn == asn)
    }
}

/// The site's own networks, so the frontend can render "who we are" without a
/// second round trip.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct LocalNetwork {
    pub asn: Asn,
    pub name: String,
    pub policy_general: String,
    pub policy_url: Option<String>,
    pub irr_as_set: Option<String>,
    pub info_prefixes4: Option<u32>,
    pub info_prefixes6: Option<u32>,
}

/// Everything the frontend needs to render the site chrome before (or without)
/// a login. Returned by `GET /api/site`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SiteInfo {
    /// Our ASNs, even the ones with no PeeringDB record.
    pub local_asns: Vec<Asn>,
    /// The subset of `local_asns` that PeeringDB knows about, with detail.
    pub local_networks: Vec<LocalNetwork>,
    /// False when the server has no OIDC credentials, so the frontend can
    /// explain why the login button is disabled instead of 502-ing.
    pub login_enabled: bool,
}

/// One IXP where one of our ASNs and the requester's ASN are both present.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct IxOverlap {
    pub ix_id: u64,
    pub ix_name: String,
    /// Our presence on this fabric, one entry per local ASN present.
    pub ours: Vec<IxPresence>,
    /// Their presence on this fabric.
    pub theirs: Vec<IxPresence>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct IxPresence {
    pub asn: Asn,
    pub ipaddr4: Option<String>,
    pub ipaddr6: Option<String>,
    /// Port speed in Mbit/s as PeeringDB reports it.
    pub speed: u64,
    pub is_rs_peer: bool,
    pub operational: bool,
}

/// One private facility where one of our ASNs and the requester's ASN are both
/// present — the candidate set for a PNI.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FacilityOverlap {
    pub fac_id: u64,
    pub fac_name: String,
    pub city: Option<String>,
    pub country: Option<String>,
    pub our_asns: Vec<Asn>,
}

/// The full answer to "where can we peer with AS<n>?".
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct OverlapReport {
    pub peer: Network,
    pub exchanges: Vec<IxOverlap>,
    pub facilities: Vec<FacilityOverlap>,
    /// When the underlying PeeringDB data was fetched.
    pub generated_at: DateTime<Utc>,
}

impl OverlapReport {
    pub fn is_empty(&self) -> bool {
        self.exchanges.is_empty() && self.facilities.is_empty()
    }
}

/// What kind of interconnection is being asked for.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PeeringKind {
    /// Public peering across an IXP fabric.
    PublicExchange,
    /// A private network interconnect in a shared facility.
    PrivateInterconnect,
}

impl PeeringKind {
    pub fn label(&self) -> &'static str {
        match self {
            Self::PublicExchange => "Public peering (IXP)",
            Self::PrivateInterconnect => "PNI",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RequestStatus {
    Pending,
    Approved,
    Declined,
    /// Configured and live on our side.
    Provisioned,
    /// Withdrawn by the requester.
    Withdrawn,
}

impl RequestStatus {
    pub fn label(&self) -> &'static str {
        match self {
            Self::Pending => "Pending",
            Self::Approved => "Approved",
            Self::Declined => "Declined",
            Self::Provisioned => "Provisioned",
            Self::Withdrawn => "Withdrawn",
        }
    }

    /// Whether an admin can still act on a request in this state.
    pub fn is_open(&self) -> bool {
        matches!(self, Self::Pending | Self::Approved)
    }
}

/// A submitted peering request as stored and rendered.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PeeringRequest {
    pub id: String,
    /// The ASN asking to peer.
    pub peer_asn: Asn,
    pub peer_name: String,
    /// Which of our ASNs the request is aimed at.
    pub local_asn: Asn,
    pub kind: PeeringKind,
    /// PeeringDB `ix_id` for public peering, `fac_id` for a PNI.
    pub location_id: u64,
    pub location_name: String,
    pub peer_ipaddr4: Option<String>,
    pub peer_ipaddr6: Option<String>,
    pub max_prefixes4: Option<u32>,
    pub max_prefixes6: Option<u32>,
    pub notes: Option<String>,
    pub status: RequestStatus,
    /// Set by an admin when approving, declining or provisioning.
    pub decision_note: Option<String>,
    /// PeeringDB subject of the user who filed it.
    pub requested_by: String,
    pub requested_by_name: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Payload for `POST /api/requests`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct NewPeeringRequest {
    pub peer_asn: Asn,
    pub local_asn: Asn,
    pub kind: Option<PeeringKind>,
    pub location_id: u64,
    pub peer_ipaddr4: Option<String>,
    pub peer_ipaddr6: Option<String>,
    pub max_prefixes4: Option<u32>,
    pub max_prefixes6: Option<u32>,
    pub notes: Option<String>,
}

impl Default for Asn {
    fn default() -> Self {
        Asn(0)
    }
}

/// Payload for `PATCH /api/admin/requests/<id>`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DecisionRequest {
    pub status: RequestStatus,
    pub decision_note: Option<String>,
}

/// Uniform error body for every failing API call.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ApiError {
    pub error: String,
    pub message: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn asn_parses_the_forms_people_actually_type() {
        assert_eq!("13335".parse::<Asn>().unwrap(), Asn(13335));
        assert_eq!("AS13335".parse::<Asn>().unwrap(), Asn(13335));
        assert_eq!("as13335".parse::<Asn>().unwrap(), Asn(13335));
        assert_eq!("  AS13335  ".parse::<Asn>().unwrap(), Asn(13335));
    }

    #[test]
    fn asn_rejects_nonsense() {
        assert!("".parse::<Asn>().is_err());
        assert!("AS".parse::<Asn>().is_err());
        assert!("thirteen".parse::<Asn>().is_err());
        assert!("-1".parse::<Asn>().is_err());
    }

    #[test]
    fn asn_displays_with_the_as_prefix() {
        assert_eq!(Asn(13335).to_string(), "AS13335");
    }

    #[test]
    fn only_pending_and_approved_are_actionable() {
        assert!(RequestStatus::Pending.is_open());
        assert!(RequestStatus::Approved.is_open());
        assert!(!RequestStatus::Declined.is_open());
        assert!(!RequestStatus::Provisioned.is_open());
        assert!(!RequestStatus::Withdrawn.is_open());
    }

    #[test]
    fn a_user_may_act_only_for_networks_they_are_affiliated_with() {
        let user = CurrentUser {
            subject: "1".to_owned(),
            username: "someone".to_owned(),
            display_name: "Someone".to_owned(),
            email: None,
            verified: true,
            affiliations: vec![Affiliation {
                asn: Asn(64500),
                name: "Example".to_owned(),
                perms: 15,
            }],
            is_admin: false,
        };

        assert!(user.may_act_for(Asn(64500)));
        assert!(!user.may_act_for(Asn(64501)));
    }
}
