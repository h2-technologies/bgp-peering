# Configuration Example

This file demonstrates how to configure the peering portal for your specific ASNs.

## Backend Configuration

### Custom ASNs

To configure your own ASNs, you can either:

1. **Environment Variables** (Recommended for production)

In `backend/wrangler.toml`:
```toml
[vars]
OUR_ASNS = "64512,64513,64514"
```

2. **Hardcode in Backend**

In `backend/src/lib.rs`, add a configuration struct:

```rust
struct Config {
    our_asns: Vec<u32>,
}

impl Config {
    fn from_env(env: &Env) -> Self {
        let asns_str = env.var("OUR_ASNS").unwrap_or(Ok("64512".to_string())).unwrap();
        let our_asns = asns_str
            .split(',')
            .filter_map(|s| s.trim().parse::<u32>().ok())
            .collect();
        
        Self { our_asns }
    }
}
```

### PeeringDB API Configuration

For production, use proper API credentials:

```toml
# backend/wrangler.toml
[env.production]
name = "peering-portal-backend-prod"

[env.production.vars]
PEERINGDB_API_BASE = "https://www.peeringdb.com/api"
```

Use secrets for sensitive data:
```bash
wrangler secret put PEERINGDB_API_KEY
```

## Frontend Configuration

### API Endpoint

Update `frontend/src/api.rs` for different environments:

```rust
#[cfg(debug_assertions)]
const API_BASE: &str = "http://localhost:8787/api";

#[cfg(not(debug_assertions))]
const API_BASE: &str = "https://peering-portal-backend.your-domain.workers.dev/api";
```

### Default ASNs

You can set default ASN values in the frontend:

```rust
// frontend/src/pages/location_match.rs
#[function_component(LocationMatch)]
pub fn location_match() -> Html {
    // Set default values
    let our_asn = use_state(|| String::from("64512"));
    let requester_asn = use_state(|| String::new());
    // ...
}
```

## Example Configurations

### Single ASN Organization

```toml
# backend/wrangler.toml
[vars]
OUR_ASNS = "64512"
ORGANIZATION_NAME = "Example Networks"
```

### Multi-ASN Organization

```toml
# backend/wrangler.toml
[vars]
OUR_ASNS = "64512,64513,64514,64515"
ORGANIZATION_NAME = "Example Multi-Networks"
```

### Production Environment

```toml
# backend/wrangler.toml
[env.production]
name = "peering-portal-backend-prod"
routes = [
  { pattern = "api.peering.example.com", zone_name = "example.com" }
]

[env.production.vars]
ENVIRONMENT = "production"
PEERINGDB_API_BASE = "https://www.peeringdb.com/api"
```

### Development Environment

```toml
# backend/wrangler.toml
[env.dev]
name = "peering-portal-backend-dev"

[env.dev.vars]
ENVIRONMENT = "development"
```

## CORS Configuration

For production, update CORS settings in `backend/src/lib.rs`:

```rust
use worker::*;

async fn add_cors_headers(response: Response, origin: &str) -> Result<Response> {
    let mut headers = response.headers().clone();
    headers.set("Access-Control-Allow-Origin", origin)?;
    headers.set("Access-Control-Allow-Methods", "GET, POST, OPTIONS")?;
    headers.set("Access-Control-Allow-Headers", "Content-Type")?;
    
    Ok(response.with_headers(headers))
}
```

## Feature Flags

You can add feature flags to enable/disable features:

```toml
# backend/wrangler.toml
[vars]
ENABLE_REGISTRATION = "false"
ENABLE_PEERING_REQUESTS = "true"
REQUIRE_AUTHENTICATION = "true"
```

## Customization Examples

### Custom Branding

Update `frontend/src/main.rs`:

```rust
html! {
    <header class="header">
        <h1>{ "Your Company - BGP Peering Portal" }</h1>
        // ...
    </header>
}
```

Update `frontend/styles.css` for custom colors:

```css
.header {
    background: linear-gradient(135deg, #YOUR_COLOR_1 0%, #YOUR_COLOR_2 100%);
}
```

### Custom Email Notifications

Add email service configuration:

```toml
# backend/wrangler.toml
[vars]
NOTIFICATION_EMAIL = "peering@example.com"
SENDGRID_API_KEY = "your_key"  # Better as a secret
```

### Rate Limiting

```toml
# backend/wrangler.toml
[vars]
RATE_LIMIT_PER_MINUTE = "60"
RATE_LIMIT_PER_HOUR = "1000"
```
