# Peering Portal Backend

This is a Cloudflare Worker-based backend for the BGP Peering Portal.

## Features

- PeeringDB authentication integration
- ASN location matching
- RESTful API endpoints

## API Endpoints

### POST /api/auth
Authenticate with PeeringDB credentials.

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

This implementation uses the PeeringDB public API. In production, you should:
- Implement proper OAuth2 authentication
- Use API keys for PeeringDB access
- Add rate limiting and caching
- Implement proper error handling
