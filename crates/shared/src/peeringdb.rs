//! Trimmed-down mirrors of the PeeringDB API objects we actually use.
//!
//! PeeringDB returns a great many fields per object; we deserialise only what
//! the site needs so an upstream schema addition can never break us.

use serde::{Deserialize, Serialize};

use crate::Asn;

/// A PeeringDB `net` object.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Network {
    pub id: u64,
    pub asn: Asn,
    pub name: String,
    #[serde(default)]
    pub aka: Option<String>,
    #[serde(default)]
    pub website: Option<String>,
    #[serde(default)]
    pub irr_as_set: Option<String>,
    #[serde(default)]
    pub policy_general: Option<String>,
    #[serde(default)]
    pub policy_url: Option<String>,
    #[serde(default)]
    pub info_type: Option<String>,
    #[serde(default)]
    pub info_traffic: Option<String>,
    #[serde(default)]
    pub info_ratio: Option<String>,
    #[serde(default)]
    pub info_scope: Option<String>,
    #[serde(default)]
    pub info_prefixes4: Option<u32>,
    #[serde(default)]
    pub info_prefixes6: Option<u32>,
    #[serde(default)]
    pub ix_count: Option<u32>,
    #[serde(default)]
    pub fac_count: Option<u32>,
}

/// A PeeringDB `netixlan` object: one network's port on one IXP LAN.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct NetIxLan {
    pub id: u64,
    pub asn: Asn,
    pub ix_id: u64,
    pub ixlan_id: u64,
    /// The IXP's name as PeeringDB denormalises it onto the netixlan.
    #[serde(default)]
    pub name: String,
    #[serde(default)]
    pub ipaddr4: Option<String>,
    #[serde(default)]
    pub ipaddr6: Option<String>,
    #[serde(default)]
    pub speed: u64,
    #[serde(default)]
    pub is_rs_peer: bool,
    #[serde(default)]
    pub operational: bool,
    #[serde(default)]
    pub status: String,
}

impl NetIxLan {
    /// PeeringDB soft-deletes objects; only `ok` rows are real.
    pub fn is_live(&self) -> bool {
        self.status.is_empty() || self.status == "ok"
    }
}

/// A PeeringDB `netfac` object: one network's presence in one facility.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct NetFac {
    pub id: u64,
    pub local_asn: Asn,
    pub fac_id: u64,
    #[serde(default)]
    pub name: String,
    #[serde(default)]
    pub city: Option<String>,
    #[serde(default)]
    pub country: Option<String>,
    #[serde(default)]
    pub status: String,
}

impl NetFac {
    pub fn is_live(&self) -> bool {
        self.status.is_empty() || self.status == "ok"
    }
}

/// A PeeringDB `ix` object.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct InternetExchange {
    pub id: u64,
    pub name: String,
    #[serde(default)]
    pub name_long: Option<String>,
    #[serde(default)]
    pub city: Option<String>,
    #[serde(default)]
    pub country: Option<String>,
    #[serde(default)]
    pub website: Option<String>,
}

/// A PeeringDB `fac` object.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Facility {
    pub id: u64,
    pub name: String,
    #[serde(default)]
    pub city: Option<String>,
    #[serde(default)]
    pub country: Option<String>,
    #[serde(default)]
    pub website: Option<String>,
}

/// Every PeeringDB list response is `{"data": [...], "meta": {...}}`.
#[derive(Debug, Clone, Deserialize)]
pub struct ListResponse<T> {
    #[serde(default = "Vec::new")]
    pub data: Vec<T>,
}
