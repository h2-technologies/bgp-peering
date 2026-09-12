//! Records as they are stored in SurrealDB.
//!
//! These are deliberately separate from the `shared` wire types: SurrealDB 3
//! serialises through its own `SurrealValue` trait rather than serde, and the
//! `shared` crate has to stay compilable for `wasm32`. Keeping two structs and
//! an explicit conversion also means a storage change cannot silently alter
//! the public API.

use chrono::{DateTime, Utc};
use shared::{
    Affiliation, Asn, CurrentUser, PeeringKind, PeeringRequest, RequestStatus,
};
use surrealdb::types::{Datetime, SurrealValue};

#[derive(Debug, Clone, SurrealValue)]
pub struct AffiliationRecord {
    pub asn: u32,
    pub name: String,
    pub perms: u32,
}

impl From<AffiliationRecord> for Affiliation {
    fn from(record: AffiliationRecord) -> Self {
        Self {
            asn: Asn(record.asn),
            name: record.name,
            perms: record.perms,
        }
    }
}

impl From<Affiliation> for AffiliationRecord {
    fn from(affiliation: Affiliation) -> Self {
        Self {
            asn: affiliation.asn.0,
            name: affiliation.name,
            perms: affiliation.perms,
        }
    }
}

#[derive(Debug, Clone, SurrealValue)]
pub struct SessionRecord {
    /// Opaque random token, also the value held in the session cookie.
    pub token: String,
    pub subject: String,
    pub username: String,
    pub display_name: String,
    pub email: Option<String>,
    pub verified: bool,
    pub affiliations: Vec<AffiliationRecord>,
    pub is_admin: bool,
    pub created_at: Datetime,
    pub expires_at: Datetime,
}

impl SessionRecord {
    pub fn is_expired(&self, now: DateTime<Utc>) -> bool {
        DateTime::<Utc>::from(self.expires_at.clone()) <= now
    }

    pub fn to_current_user(&self) -> CurrentUser {
        CurrentUser {
            subject: self.subject.clone(),
            username: self.username.clone(),
            display_name: self.display_name.clone(),
            email: self.email.clone(),
            verified: self.verified,
            affiliations: self.affiliations.iter().cloned().map(Into::into).collect(),
            is_admin: self.is_admin,
        }
    }
}

#[derive(Debug, Clone, SurrealValue)]
pub struct RequestRecord {
    /// Our own opaque id. SurrealDB record ids are awkward to round-trip
    /// through JSON, so requests carry an explicit id field.
    pub request_id: String,
    pub peer_asn: u32,
    pub peer_name: String,
    pub local_asn: u32,
    /// Serialised `PeeringKind`; stored as text so the data stays readable
    /// from the SurrealQL shell.
    pub kind: String,
    pub location_id: u64,
    pub location_name: String,
    pub peer_ipaddr4: Option<String>,
    pub peer_ipaddr6: Option<String>,
    pub max_prefixes4: Option<u32>,
    pub max_prefixes6: Option<u32>,
    pub notes: Option<String>,
    /// Serialised `RequestStatus`.
    pub status: String,
    pub decision_note: Option<String>,
    pub requested_by: String,
    pub requested_by_name: String,
    pub created_at: Datetime,
    pub updated_at: Datetime,
}

impl From<RequestRecord> for PeeringRequest {
    fn from(record: RequestRecord) -> Self {
        Self {
            id: record.request_id,
            peer_asn: Asn(record.peer_asn),
            peer_name: record.peer_name,
            local_asn: Asn(record.local_asn),
            kind: kind_from_str(&record.kind),
            location_id: record.location_id,
            location_name: record.location_name,
            peer_ipaddr4: record.peer_ipaddr4,
            peer_ipaddr6: record.peer_ipaddr6,
            max_prefixes4: record.max_prefixes4,
            max_prefixes6: record.max_prefixes6,
            notes: record.notes,
            status: status_from_str(&record.status),
            decision_note: record.decision_note,
            requested_by: record.requested_by,
            requested_by_name: record.requested_by_name,
            created_at: record.created_at.into(),
            updated_at: record.updated_at.into(),
        }
    }
}

#[derive(Debug, Clone, SurrealValue)]
pub struct CacheRecord {
    pub cache_key: String,
    /// The upstream JSON body, verbatim.
    pub payload: String,
    pub fetched_at: Datetime,
}

pub fn kind_to_str(kind: PeeringKind) -> &'static str {
    match kind {
        PeeringKind::PublicExchange => "public_exchange",
        PeeringKind::PrivateInterconnect => "private_interconnect",
    }
}

/// Unknown values fall back to public peering rather than panicking; the only
/// way to store one is a hand-edit of the datastore.
pub fn kind_from_str(value: &str) -> PeeringKind {
    match value {
        "private_interconnect" => PeeringKind::PrivateInterconnect,
        _ => PeeringKind::PublicExchange,
    }
}

pub fn status_to_str(status: RequestStatus) -> &'static str {
    match status {
        RequestStatus::Pending => "pending",
        RequestStatus::Approved => "approved",
        RequestStatus::Declined => "declined",
        RequestStatus::Provisioned => "provisioned",
        RequestStatus::Withdrawn => "withdrawn",
    }
}

pub fn status_from_str(value: &str) -> RequestStatus {
    match value {
        "approved" => RequestStatus::Approved,
        "declined" => RequestStatus::Declined,
        "provisioned" => RequestStatus::Provisioned,
        "withdrawn" => RequestStatus::Withdrawn,
        _ => RequestStatus::Pending,
    }
}
