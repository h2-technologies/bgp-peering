//! PeeringDB API client with a SurrealDB-backed response cache.
//!
//! Every read is public data, so the cache is keyed purely on the request and
//! shared between users. Anonymous PeeringDB access is rate limited fairly
//! aggressively, which is what the cache is really protecting us from.

use std::collections::BTreeMap;
use std::time::Duration;

use chrono::Utc;
use serde::Serialize;
use serde::de::DeserializeOwned;
use shared::peeringdb::{ListResponse, NetFac, NetIxLan};
use shared::{
    Asn, FacilityOverlap, IxOverlap, IxPresence, LocalNetwork, Network, OverlapReport,
};
use surrealdb::types::Datetime;

use crate::config::AppConfig;
use crate::db::Database;
use crate::error::{Error, Result};
use crate::models::CacheRecord;

pub struct PeeringDb {
    http: reqwest::Client,
    db: Database,
    api_base: String,
    api_key: Option<String>,
    ttl: Duration,
}

impl PeeringDb {
    pub fn new(config: &AppConfig, db: Database) -> Result<Self> {
        let http = reqwest::Client::builder()
            .user_agent(concat!(
                "bgp-peering/",
                env!("CARGO_PKG_VERSION"),
                " (+https://github.com/h2-technologies/bgp-peering)"
            ))
            .timeout(Duration::from_secs(20))
            .build()?;

        Ok(Self {
            http,
            db,
            api_base: config.peeringdb_api.trim_end_matches('/').to_owned(),
            api_key: config.peeringdb_api_key.clone(),
            ttl: Duration::from_secs(config.cache_ttl_seconds),
        })
    }

    /// The `net` object for an ASN, or `None` if PeeringDB does not know it.
    pub async fn network(&self, asn: Asn) -> Result<Option<Network>> {
        let nets: Vec<Network> = self.list("net", &[("asn", asn.0.to_string())]).await?;
        Ok(nets.into_iter().next())
    }

    /// Every IXP port this ASN has registered.
    pub async fn ix_presence(&self, asn: Asn) -> Result<Vec<NetIxLan>> {
        let rows: Vec<NetIxLan> = self
            .list("netixlan", &[("asn", asn.0.to_string())])
            .await?;
        Ok(rows.into_iter().filter(NetIxLan::is_live).collect())
    }

    /// Every facility this ASN has registered.
    pub async fn facility_presence(&self, asn: Asn) -> Result<Vec<NetFac>> {
        let rows: Vec<NetFac> = self
            .list("netfac", &[("local_asn", asn.0.to_string())])
            .await?;
        Ok(rows.into_iter().filter(NetFac::is_live).collect())
    }

    /// Our own networks, for the "who we are" panel.
    pub async fn local_networks(&self, asns: &[Asn]) -> Result<Vec<LocalNetwork>> {
        let mut networks = Vec::new();
        for asn in asns {
            let Some(net) = self.network(*asn).await? else {
                rocket::warn!("local ASN {asn} has no PeeringDB net object");
                continue;
            };
            networks.push(LocalNetwork {
                asn: net.asn,
                name: net.name,
                policy_general: net.policy_general.unwrap_or_else(|| "Not set".to_owned()),
                policy_url: net.policy_url.filter(|url| !url.is_empty()),
                irr_as_set: net.irr_as_set.filter(|set| !set.is_empty()),
                info_prefixes4: net.info_prefixes4,
                info_prefixes6: net.info_prefixes6,
            });
        }
        Ok(networks)
    }

    /// Work out every IXP and facility where `peer_asn` and one of `local_asns`
    /// are both present.
    pub async fn overlap(&self, local_asns: &[Asn], peer_asn: Asn) -> Result<OverlapReport> {
        let peer = self
            .network(peer_asn)
            .await?
            .ok_or_else(|| Error::NotFound(format!("{peer_asn} is not registered in PeeringDB")))?;

        let their_ix = self.ix_presence(peer_asn).await?;
        let their_fac = self.facility_presence(peer_asn).await?;

        // ix_id -> (name, our presences, their presences)
        let mut exchanges: BTreeMap<u64, IxOverlap> = BTreeMap::new();
        for row in &their_ix {
            exchanges
                .entry(row.ix_id)
                .or_insert_with(|| IxOverlap {
                    ix_id: row.ix_id,
                    ix_name: row.name.clone(),
                    ours: Vec::new(),
                    theirs: Vec::new(),
                })
                .theirs
                .push(presence(row));
        }

        let mut facilities: BTreeMap<u64, FacilityOverlap> = BTreeMap::new();
        for row in &their_fac {
            facilities.entry(row.fac_id).or_insert_with(|| FacilityOverlap {
                fac_id: row.fac_id,
                fac_name: row.name.clone(),
                city: row.city.clone(),
                country: row.country.clone(),
                our_asns: Vec::new(),
            });
        }

        for local in local_asns {
            // Peering with yourself is not a thing; skip so an operator whose
            // own ASN is looked up does not get a nonsense report.
            if *local == peer_asn {
                continue;
            }

            for row in self.ix_presence(*local).await? {
                if let Some(entry) = exchanges.get_mut(&row.ix_id) {
                    if entry.ix_name.is_empty() {
                        entry.ix_name = row.name.clone();
                    }
                    entry.ours.push(presence(&row));
                }
            }

            for row in self.facility_presence(*local).await? {
                if let Some(entry) = facilities.get_mut(&row.fac_id) {
                    entry.our_asns.push(*local);
                }
            }
        }

        // Anywhere only one side is present is not an overlap.
        let mut exchanges: Vec<IxOverlap> = exchanges
            .into_values()
            .filter(|entry| !entry.ours.is_empty())
            .collect();
        exchanges.sort_by(|a, b| a.ix_name.cmp(&b.ix_name));

        let mut facilities: Vec<FacilityOverlap> = facilities
            .into_values()
            .filter(|entry| !entry.our_asns.is_empty())
            .collect();
        facilities.sort_by(|a, b| a.fac_name.cmp(&b.fac_name));

        Ok(OverlapReport {
            peer,
            exchanges,
            facilities,
            generated_at: Utc::now(),
        })
    }

    /// GET `{api_base}/{resource}?{params}`, served from cache when fresh.
    async fn list<T>(&self, resource: &str, params: &[(&str, String)]) -> Result<Vec<T>>
    where
        T: DeserializeOwned + Serialize,
    {
        let key = cache_key(resource, params);

        if let Some(body) = self.cached(&key).await? {
            match serde_json::from_str::<ListResponse<T>>(&body) {
                Ok(parsed) => return Ok(parsed.data),
                // A cached body we can no longer parse (upstream schema drift,
                // or a partial write) should not be fatal — refetch instead.
                Err(error) => rocket::warn!("discarding unparsable cache entry {key}: {error}"),
            }
        }

        let body = self.get(resource, params).await?;
        let parsed: ListResponse<T> = serde_json::from_str(&body).map_err(|error| {
            Error::PeeringDb(format!("could not parse the {resource} response: {error}"))
        })?;

        self.store(&key, &body).await?;
        Ok(parsed.data)
    }

    async fn get(&self, resource: &str, params: &[(&str, String)]) -> Result<String> {
        let url = format!("{}/{resource}", self.api_base);
        let mut request = self.http.get(&url).query(params);

        if let Some(key) = &self.api_key {
            request = request.header("Authorization", format!("Api-Key {key}"));
        }

        let response = request.send().await?;
        let status = response.status();

        if status.as_u16() == 429 {
            return Err(Error::PeeringDb(
                "PeeringDB rate limit reached; try again shortly.".to_owned(),
            ));
        }
        if !status.is_success() {
            return Err(Error::PeeringDb(format!("{url} returned HTTP {status}")));
        }

        Ok(response.text().await?)
    }

    /// The cached body for `key`, if one exists and is still within the TTL.
    async fn cached(&self, key: &str) -> Result<Option<String>> {
        let mut response = self
            .db
            .query("SELECT * FROM pdb_cache WHERE cache_key = $key LIMIT 1")
            .bind(("key", key.to_owned()))
            .await?
            .check()?;

        let rows: Vec<CacheRecord> = response.take(0)?;
        let Some(row) = rows.into_iter().next() else {
            return Ok(None);
        };

        let age = Utc::now().signed_duration_since(chrono::DateTime::from(row.fetched_at));
        let expired = age
            .to_std()
            .map(|age| age > self.ttl)
            // A negative age means the clock moved backwards; treat as fresh.
            .unwrap_or(false);

        Ok((!expired).then_some(row.payload))
    }

    async fn store(&self, key: &str, body: &str) -> Result<()> {
        let record = CacheRecord {
            cache_key: key.to_owned(),
            payload: body.to_owned(),
            fetched_at: Datetime::from(Utc::now()),
        };

        // Keyed by the cache key itself, so two requests racing for the same
        // resource upsert one record rather than tripping the unique index.
        self.db
            .query("UPSERT type::record('pdb_cache', $key) CONTENT $record")
            .bind(("key", key.to_owned()))
            .bind(("record", record))
            .await?
            .check()?;

        Ok(())
    }
}

fn presence(row: &NetIxLan) -> IxPresence {
    IxPresence {
        asn: row.asn,
        ipaddr4: row.ipaddr4.clone().filter(|ip| !ip.is_empty()),
        ipaddr6: row.ipaddr6.clone().filter(|ip| !ip.is_empty()),
        speed: row.speed,
        is_rs_peer: row.is_rs_peer,
        operational: row.operational,
    }
}

fn cache_key(resource: &str, params: &[(&str, String)]) -> String {
    let query: Vec<String> = params
        .iter()
        .map(|(name, value)| format!("{name}={value}"))
        .collect();
    format!("{resource}?{}", query.join("&"))
}
