//! The JSON API consumed by the Yew frontend.

use chrono::Utc;
use oauth2::CsrfToken;
use rocket::serde::json::Json;
use rocket::{State, get, patch, post};
use shared::{
    Asn, CurrentUser, DecisionRequest, NewPeeringRequest, OverlapReport, PeeringKind,
    PeeringRequest, RequestStatus, SiteInfo,
};
use surrealdb::types::Datetime;

use crate::auth::{AdminUser, AuthUser};
use crate::config::AppConfig;
use crate::db::Database;
use crate::error::{Error, Result};
use crate::models::{RequestRecord, kind_to_str, status_from_str, status_to_str};
use crate::peeringdb::PeeringDb;

/// Site chrome: who we are and whether login is available. Unauthenticated.
#[get("/api/site")]
pub async fn site(config: &State<AppConfig>, pdb: &State<PeeringDb>) -> Result<Json<SiteInfo>> {
    let local_asns = config.local_asns();
    // A PeeringDB outage should not blank the whole site, so fall back to the
    // bare ASN list rather than failing.
    let local_networks = match pdb.local_networks(&local_asns).await {
        Ok(networks) => networks,
        Err(error) => {
            rocket::warn!("could not load our own PeeringDB records: {error}");
            Vec::new()
        }
    };

    Ok(Json(SiteInfo {
        local_asns,
        local_networks,
        login_enabled: config.oidc_configured(),
    }))
}

/// The caller's identity, or `null` when signed out.
#[get("/api/me")]
pub fn me(user: Option<AuthUser>) -> Json<Option<CurrentUser>> {
    Json(user.map(|AuthUser(session)| session.to_current_user()))
}

/// Where we and `asn` can peer. PeeringDB data is public, so any signed-in
/// user may look up any ASN.
#[get("/api/overlap/<asn>")]
pub async fn overlap(
    asn: u32,
    _user: AuthUser,
    config: &State<AppConfig>,
    pdb: &State<PeeringDb>,
) -> Result<Json<OverlapReport>> {
    let report = pdb.overlap(&config.local_asns(), Asn(asn)).await?;
    Ok(Json(report))
}

/// The caller's own requests, newest first.
#[get("/api/requests")]
pub async fn my_requests(
    user: AuthUser,
    db: &State<Database>,
) -> Result<Json<Vec<PeeringRequest>>> {
    let mut response = db
        .query("SELECT * FROM peering_request WHERE requested_by = $subject ORDER BY created_at DESC")
        .bind(("subject", user.0.subject.clone()))
        .await?
        .check()?;

    let records: Vec<RequestRecord> = response.take(0)?;
    Ok(Json(records.into_iter().map(Into::into).collect()))
}

#[post("/api/requests", data = "<body>")]
pub async fn create_request(
    body: Json<NewPeeringRequest>,
    user: AuthUser,
    config: &State<AppConfig>,
    pdb: &State<PeeringDb>,
    db: &State<Database>,
) -> Result<Json<PeeringRequest>> {
    let body = body.into_inner();

    // PeeringDB affiliation is the authorisation check: you may only ask for
    // peering on behalf of a network you are actually attached to.
    if !user.may_act_for(body.peer_asn) {
        return Err(Error::Forbidden(format!(
            "You are not affiliated with {} in PeeringDB.",
            body.peer_asn
        )));
    }

    if !config.is_local_asn(body.local_asn) {
        return Err(Error::BadRequest(format!(
            "{} is not one of our networks.",
            body.local_asn
        )));
    }

    // Re-derive the overlap server-side rather than trusting the submitted
    // location: it both validates the request and gives us the display name.
    let report = pdb.overlap(&config.local_asns(), body.peer_asn).await?;
    let (kind, location_name) =
        resolve_location(&report, body.location_id, body.kind, body.local_asn)?;

    let now = Utc::now();
    let record = RequestRecord {
        request_id: CsrfToken::new_random_len(12).secret().clone(),
        peer_asn: body.peer_asn.0,
        peer_name: report.peer.name.clone(),
        local_asn: body.local_asn.0,
        kind: kind_to_str(kind).to_owned(),
        location_id: body.location_id,
        location_name,
        peer_ipaddr4: sanitise(body.peer_ipaddr4),
        peer_ipaddr6: sanitise(body.peer_ipaddr6),
        max_prefixes4: body.max_prefixes4,
        max_prefixes6: body.max_prefixes6,
        notes: sanitise(body.notes),
        status: status_to_str(RequestStatus::Pending).to_owned(),
        decision_note: None,
        requested_by: user.0.subject.clone(),
        requested_by_name: user.0.display_name.clone(),
        created_at: Datetime::from(now),
        updated_at: Datetime::from(now),
    };

    db.query("CREATE peering_request CONTENT $record")
        .bind(("record", record.clone()))
        .await?
        .check()?;

    rocket::info!(
        "new {} request from {} at {} (by {})",
        kind.label(),
        record.peer_asn,
        record.location_name,
        user.0.display_name
    );

    Ok(Json(record.into()))
}

/// Withdraw one of your own requests.
#[post("/api/requests/<id>/withdraw")]
pub async fn withdraw_request(
    id: &str,
    user: AuthUser,
    db: &State<Database>,
) -> Result<Json<PeeringRequest>> {
    let existing = load_request(db, id).await?;

    if existing.requested_by != user.0.subject {
        return Err(Error::Forbidden("That request is not yours.".to_owned()));
    }
    if !status_from_str(&existing.status).is_open() {
        return Err(Error::BadRequest(
            "That request is already closed.".to_owned(),
        ));
    }

    set_status(db, id, RequestStatus::Withdrawn, None).await?;
    Ok(Json(load_request(db, id).await?.into()))
}

/// Every request, for the admin queue.
#[get("/api/admin/requests")]
pub async fn all_requests(
    _admin: AdminUser,
    db: &State<Database>,
) -> Result<Json<Vec<PeeringRequest>>> {
    let mut response = db
        .query("SELECT * FROM peering_request ORDER BY created_at DESC")
        .await?
        .check()?;

    let records: Vec<RequestRecord> = response.take(0)?;
    Ok(Json(records.into_iter().map(Into::into).collect()))
}

#[patch("/api/admin/requests/<id>", data = "<body>")]
pub async fn decide_request(
    id: &str,
    body: Json<DecisionRequest>,
    admin: AdminUser,
    db: &State<Database>,
) -> Result<Json<PeeringRequest>> {
    let body = body.into_inner();

    if matches!(body.status, RequestStatus::Withdrawn) {
        return Err(Error::BadRequest(
            "Only the requester can withdraw a request.".to_owned(),
        ));
    }

    // Confirm it exists before writing, so a bad id is a 404 rather than a
    // silent no-op.
    load_request(db, id).await?;
    set_status(db, id, body.status, sanitise(body.decision_note)).await?;

    rocket::info!(
        "{} marked request {id} as {}",
        admin.0.display_name,
        body.status.label()
    );

    Ok(Json(load_request(db, id).await?.into()))
}

async fn load_request(db: &Database, id: &str) -> Result<RequestRecord> {
    let mut response = db
        .query("SELECT * FROM peering_request WHERE request_id = $id LIMIT 1")
        .bind(("id", id.to_owned()))
        .await?
        .check()?;

    let records: Vec<RequestRecord> = response.take(0)?;
    records
        .into_iter()
        .next()
        .ok_or_else(|| Error::NotFound(format!("No request with id {id}.")))
}

async fn set_status(
    db: &Database,
    id: &str,
    status: RequestStatus,
    note: Option<String>,
) -> Result<()> {
    db.query(
        "UPDATE peering_request
         SET status = $status, decision_note = $note, updated_at = $now
         WHERE request_id = $id",
    )
    .bind(("id", id.to_owned()))
    .bind(("status", status_to_str(status).to_owned()))
    .bind(("note", note))
    .bind(("now", Datetime::from(Utc::now())))
    .await?
    .check()?;

    Ok(())
}

/// Match a submitted `location_id` against the computed overlap, returning the
/// interconnection kind and the location's display name.
///
/// `local_asn` must itself be present at that location: the report as a whole
/// covers every ASN of ours, so "the location is in the report" is a weaker
/// claim than "these two networks meet there".
fn resolve_location(
    report: &OverlapReport,
    location_id: u64,
    declared: Option<PeeringKind>,
    local_asn: Asn,
) -> Result<(PeeringKind, String)> {
    let exchange = report
        .exchanges
        .iter()
        .find(|ix| ix.ix_id == location_id && ix.ours.iter().any(|p| p.asn == local_asn));
    let facility = report
        .facilities
        .iter()
        .find(|fac| fac.fac_id == location_id && fac.our_asns.contains(&local_asn));

    let not_there = || {
        Error::BadRequest(format!(
            "{local_asn} and {} do not both appear at that location.",
            report.peer.asn
        ))
    };

    match (declared, exchange, facility) {
        (Some(PeeringKind::PublicExchange), Some(ix), _) => {
            Ok((PeeringKind::PublicExchange, ix.ix_name.clone()))
        }
        (Some(PeeringKind::PrivateInterconnect), _, Some(fac)) => {
            Ok((PeeringKind::PrivateInterconnect, fac.fac_name.clone()))
        }
        // An id is an IX id or a facility id, never both, so when the caller
        // does not say which it is we can infer it unambiguously.
        (None, Some(ix), None) => Ok((PeeringKind::PublicExchange, ix.ix_name.clone())),
        (None, None, Some(fac)) => Ok((PeeringKind::PrivateInterconnect, fac.fac_name.clone())),
        (None, Some(_), Some(_)) => Err(Error::BadRequest(
            "That location is ambiguous; specify whether you want an IX or a PNI.".to_owned(),
        )),
        _ => Err(not_there()),
    }
}

/// Treat whitespace-only input as absent.
fn sanitise(value: Option<String>) -> Option<String> {
    value
        .map(|text| text.trim().to_owned())
        .filter(|text| !text.is_empty())
}

#[cfg(test)]
mod tests {
    use chrono::Utc;
    use shared::{FacilityOverlap, IxOverlap, IxPresence, Network};

    use super::*;

    const PEER: Asn = Asn(64510);
    const OURS: Asn = Asn(64500);
    /// One of ours, but not present at the exchange in the fixture.
    const OURS_ELSEWHERE: Asn = Asn(64501);

    fn presence(asn: Asn) -> IxPresence {
        IxPresence {
            asn,
            ipaddr4: Some("198.51.100.1".to_owned()),
            ipaddr6: None,
            speed: 10_000,
            is_rs_peer: false,
            operational: true,
        }
    }

    fn report() -> OverlapReport {
        OverlapReport {
            peer: Network {
                id: 1,
                asn: PEER,
                name: "Peer Networks".to_owned(),
                aka: None,
                website: None,
                irr_as_set: None,
                policy_general: Some("Open".to_owned()),
                policy_url: None,
                info_type: None,
                info_traffic: None,
                info_ratio: None,
                info_scope: None,
                info_prefixes4: None,
                info_prefixes6: None,
                ix_count: None,
                fac_count: None,
            },
            exchanges: vec![IxOverlap {
                ix_id: 100,
                ix_name: "Example IX".to_owned(),
                ours: vec![presence(OURS)],
                theirs: vec![presence(PEER)],
            }],
            facilities: vec![FacilityOverlap {
                fac_id: 200,
                fac_name: "Example DC".to_owned(),
                city: Some("Somewhere".to_owned()),
                country: Some("US".to_owned()),
                our_asns: vec![OURS],
            }],
            generated_at: Utc::now(),
        }
    }

    #[test]
    fn an_exchange_id_resolves_to_public_peering() {
        let (kind, name) = resolve_location(&report(), 100, None, OURS).unwrap();
        assert!(matches!(kind, PeeringKind::PublicExchange));
        assert_eq!(name, "Example IX");
    }

    #[test]
    fn a_facility_id_resolves_to_a_pni() {
        let (kind, name) = resolve_location(&report(), 200, None, OURS).unwrap();
        assert!(matches!(kind, PeeringKind::PrivateInterconnect));
        assert_eq!(name, "Example DC");
    }

    #[test]
    fn a_declared_kind_must_match_the_location() {
        // 100 is an exchange, so asking for a PNI there is not satisfiable.
        assert!(
            resolve_location(
                &report(),
                100,
                Some(PeeringKind::PrivateInterconnect),
                OURS
            )
            .is_err()
        );
    }

    #[test]
    fn unknown_locations_are_rejected() {
        assert!(resolve_location(&report(), 999, None, OURS).is_err());
    }

    /// The report covers every ASN of ours, so a location being present in it
    /// does not mean the *requested* ASN of ours is there.
    #[test]
    fn the_chosen_local_asn_must_be_present_at_the_location() {
        assert!(resolve_location(&report(), 100, None, OURS_ELSEWHERE).is_err());
        assert!(resolve_location(&report(), 200, None, OURS_ELSEWHERE).is_err());
    }
}
