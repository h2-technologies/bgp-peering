//! Embedded SurrealDB (SurrealKV) datastore.

use std::path::Path;

use surrealdb::Surreal;
use surrealdb::engine::local::{Db, SurrealKv};

use crate::error::Result;

pub type Database = Surreal<Db>;

const NAMESPACE: &str = "bgp_peering";
const DATABASE: &str = "main";

/// Open (creating if needed) the SurrealKV store and apply the schema.
pub async fn connect(path: &Path) -> Result<Database> {
    std::fs::create_dir_all(path)?;

    let db = Surreal::new::<SurrealKv>(path.display().to_string()).await?;
    db.use_ns(NAMESPACE).use_db(DATABASE).await?;
    apply_schema(&db).await?;

    rocket::info!("SurrealKV datastore ready at {}", path.display());
    Ok(db)
}

/// Tables are schemaless — the Rust records are the schema — but the indexes
/// matter: every lookup below goes through one of them.
async fn apply_schema(db: &Database) -> Result<()> {
    db.query(
        r#"
        DEFINE TABLE IF NOT EXISTS session SCHEMALESS;
        DEFINE INDEX IF NOT EXISTS session_token_idx
            ON TABLE session COLUMNS token UNIQUE;
        DEFINE INDEX IF NOT EXISTS session_subject_idx
            ON TABLE session COLUMNS subject;

        DEFINE TABLE IF NOT EXISTS peering_request SCHEMALESS;
        DEFINE INDEX IF NOT EXISTS request_id_idx
            ON TABLE peering_request COLUMNS request_id UNIQUE;
        DEFINE INDEX IF NOT EXISTS request_requester_idx
            ON TABLE peering_request COLUMNS requested_by;
        DEFINE INDEX IF NOT EXISTS request_peer_asn_idx
            ON TABLE peering_request COLUMNS peer_asn;

        DEFINE TABLE IF NOT EXISTS pdb_cache SCHEMALESS;
        DEFINE INDEX IF NOT EXISTS cache_key_idx
            ON TABLE pdb_cache COLUMNS cache_key UNIQUE;
        "#,
    )
    .await?
    .check()?;

    Ok(())
}

/// Drop sessions that have already expired. Called at startup so an
/// abandoned deployment does not accumulate them forever.
pub async fn prune_expired_sessions(db: &Database) -> Result<()> {
    db.query("DELETE FROM session WHERE expires_at < time::now()")
        .await?
        .check()?;
    Ok(())
}
