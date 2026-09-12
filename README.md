# bgp-peering

A peering request portal. Operators sign in with their **PeeringDB** account,
see every internet exchange and facility where their network and ours are both
present, and file a peering request against a specific location. We work the
resulting queue.

PeeringDB is the single source of truth for both halves of the problem:

- **Identity and authorisation** — login is OpenID Connect against
  `auth.peeringdb.com`. The `networks` scope tells us which ASNs the user is
  affiliated with, so nobody has to be provisioned here and nobody can file a
  request for a network they do not represent.
- **Presence** — overlap is computed from `netixlan` and `netfac` records. If
  your PeeringDB entries are current, this site is current.

## Stack

| Layer      | Choice                                                            |
| ---------- | ----------------------------------------------------------------- |
| Backend    | [Rocket](https://rocket.rs) 0.5                                    |
| Frontend   | [Yew](https://yew.rs) 0.23 (CSR) + `yew-router`, built with Trunk  |
| Database   | [SurrealDB](https://surrealdb.com) 3, embedded on the SurrealKV engine |
| Auth       | PeeringDB OIDC (authorization code + PKCE) via `oauth2` 5          |

There is no separate database server and no separate frontend host: SurrealKV
is embedded in the process, and Rocket serves the compiled WASM bundle
alongside the API, so a deployment is one binary plus one `dist/` directory.

## Layout

```
Cargo.toml            workspace; `cargo build` builds the server only
Rocket.toml           server + application configuration
Trunk.toml            frontend build and dev-proxy configuration
crates/shared/        wire types used by both sides (serde, wasm-safe)
crates/backend/       Rocket server, SurrealDB access, PeeringDB client, OIDC
crates/frontend/      Yew SPA
```

`crates/frontend` is in the workspace but excluded from `default-members`,
because it only compiles for `wasm32-unknown-unknown`. `cargo build` at the
root therefore builds the server; Trunk builds the frontend.

## Prerequisites

```sh
rustup target add wasm32-unknown-unknown
cargo install trunk
```

## Register the OAuth application

At <https://www.peeringdb.com/oauth2/applications/>, create an application
with:

| Field         | Value                              |
| ------------- | ---------------------------------- |
| Client type   | Confidential                       |
| Grant type    | Authorization code                 |
| Redirect URI  | `<public_url>/auth/callback`       |

The redirect URI must match `public_url` exactly, including scheme and port.
For local development that is `http://localhost:8000/auth/callback`.

## Configure

Defaults live in `Rocket.toml`; anything secret should come from the
environment. Every key can be overridden with a `ROCKET_`-prefixed variable.
Copy `.env.example` to `.env` for local development — the server loads it at
startup.

| Setting                | Environment variable          | Notes                                                    |
| ---------------------- | ----------------------------- | -------------------------------------------------------- |
| `local_asns`           | `ROCKET_LOCAL_ASNS`           | **Required.** TOML/JSON array, e.g. `[64500, 64501]`      |
| `oidc_client_id`       | `ROCKET_OIDC_CLIENT_ID`       | From the PeeringDB application                            |
| `oidc_client_secret`   | `ROCKET_OIDC_CLIENT_SECRET`   | From the PeeringDB application                            |
| `public_url`           | `ROCKET_PUBLIC_URL`           | Origin browsers use; drives the redirect URI              |
| `peeringdb_api_key`    | `ROCKET_PEERINGDB_API_KEY`    | Optional; raises the API rate limit                       |
| `database_path`        | `ROCKET_DATABASE_PATH`        | SurrealKV directory, default `data/surrealkv`             |
| `cache_ttl_seconds`    | `ROCKET_CACHE_TTL_SECONDS`    | PeeringDB cache lifetime, default 3600                    |
| `session_ttl_hours`    | `ROCKET_SESSION_TTL_HOURS`    | Login lifetime, default 168                               |
| `static_dir`           | `ROCKET_STATIC_DIR`           | Trunk output, default `dist`                              |
| —                      | `ROCKET_SECRET_KEY`           | **Required in release.** Encrypts session cookies         |

Generate the secret key with `openssl rand -base64 32`. Without it a release
build refuses to start, because sessions are encrypted private cookies.

The server starts even with no ASNs and no OIDC credentials — it logs a warning
for each and disables the affected feature — so a misconfiguration surfaces as
a clear message rather than a crash loop.

## Run it

Two terminals during development, so the frontend hot-reloads:

```sh
cargo run -p backend          # API on :8000
trunk serve                   # SPA on :8080, proxying /api and /auth to :8000
```

Open <http://localhost:8080>. `Trunk.toml` proxies `/api` and `/auth` to
Rocket, which keeps everything same-origin so the session cookie works.

For a production-shaped run, build the bundle and let Rocket serve it:

```sh
trunk build --release
ROCKET_PROFILE=release \
ROCKET_SECRET_KEY="$(openssl rand -base64 32)" \
ROCKET_LOCAL_ASNS='[64500]' \
ROCKET_PUBLIC_URL='https://peering.example.net' \
  cargo run --release -p backend
```

Then the whole site is on :8000.

## API

Everything is JSON, and every error shares one shape:
`{"error": "<code>", "message": "<human text>"}`.

| Method  | Path                          | Access             |
| ------- | ----------------------------- | ------------------ |
| `GET`   | `/api/site`                   | public             |
| `GET`   | `/api/me`                     | public (`null` when signed out) |
| `GET`   | `/api/overlap/<asn>`          | signed in          |
| `GET`   | `/api/requests`               | signed in (own)    |
| `POST`  | `/api/requests`               | affiliated with the peer ASN |
| `POST`  | `/api/requests/<id>/withdraw` | requester only     |
| `GET`   | `/api/admin/requests`         | admin              |
| `PATCH` | `/api/admin/requests/<id>`    | admin              |
| `GET`   | `/auth/login`                 | starts the OIDC flow |
| `GET`   | `/auth/callback`              | OIDC redirect target |
| `POST`  | `/auth/logout`                | signed in          |

**Admin** means "affiliated in PeeringDB with one of `local_asns`". There is no
separate role table; revoking someone's affiliation in PeeringDB revokes their
access here.

Request submission is validated server-side against a freshly computed overlap,
so a caller cannot file against a location where the two networks do not
actually meet, or on behalf of an ASN they are not affiliated with.

## Storage

Three schemaless tables in the embedded datastore, each with the indexes its
lookups need:

- `session` — opaque token, PeeringDB subject, cached affiliations, expiry.
  The token lives in an encrypted `HttpOnly` cookie; expired rows are pruned at
  startup.
- `peering_request` — the queue.
- `pdb_cache` — raw PeeringDB response bodies keyed by resource and query,
  with a TTL. Public data only, so it is shared across users.

## Licence

MIT — see [LICENSE](LICENSE).
