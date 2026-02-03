# Peering Portal Backend

This is a Cloudflare Worker-based backend for the BGP Peering Portal.

## Features

- PeeringDB OAuth2 and OIDC authentication integration
- ASN location matching
- RESTful API endpoints
- HMAC-SHA256 ID token verification

## API Endpoints

### GET /api/oauth/authorize-url
Get the OAuth2 authorization URL to redirect users to PeeringDB login.

Response:
```json
{
  "authorization_url": "https://auth.peeringdb.com/oauth2/authorize/?client_id=...&redirect_uri=...&response_type=code&scope=openid%20profile%20email"
}
```

### GET /api/oauth/callback
OAuth2 callback endpoint that receives the authorization code from PeeringDB.

Query Parameters:
- `code`: Authorization code from PeeringDB
- `error`: Error message (if authorization failed)

This endpoint:
1. Exchanges the authorization code for an access token
2. Verifies the OIDC ID token using HMAC-SHA256 (if present)
3. Redirects back to the frontend with the token

### POST /api/auth
Legacy authentication endpoint (kept for backwards compatibility).

Request:
```json
{
  "username": "your_username",
  "password": "your_password"
}
```

Response:
```json
{
  "success": true,
  "token": "pdb_token_...",
  "message": "Authentication successful"
}
```

### POST /api/match-locations
Check for common locations between two ASNs.

Request:
```json
{
  "our_asn": 64512,
  "requester_asn": 64513
}
```

Response:
```json
{
  "matches": [
    {
      "facility_id": "123",
      "name": "Example DC",
      "city": "New York",
      "country": "US"
    }
  ],
  "our_locations": ["New York, US", "London, GB"],
  "requester_locations": ["New York, US", "Paris, FR"]
}
```

## Development

Install dependencies:
```bash
cargo build
```

Build for production:
```bash
worker-build --release
```

Deploy to Cloudflare:
```bash
wrangler deploy
```

## Note

This implementation uses PeeringDB OAuth2 and OpenID Connect. Key features:

### OAuth2 Flow
- Standard authorization code flow
- Secure token exchange on the backend
- Access tokens returned to frontend

### OIDC Support
- Requests `openid profile email` scopes
- Verifies ID tokens using HMAC-SHA256
- Validates token claims (audience, expiration)
- Uses client secret as HMAC signing key

### Security Best Practices
In production, you should:
- Store client secrets in Cloudflare Workers secrets (not hardcoded)
- Implement proper OAuth state parameter for CSRF protection
- Add rate limiting and request validation
- Implement token refresh flow
- Use secure cookie storage instead of URL parameters for tokens
- Implement proper error handling and logging

### Configuration
Set up OAuth credentials in wrangler.toml secrets:
```bash
wrangler secret put OAUTH_CLIENT_ID
wrangler secret put OAUTH_CLIENT_SECRET
```

See [OAUTH_SETUP.md](../OAUTH_SETUP.md) for complete configuration details.
