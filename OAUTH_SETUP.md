# OAuth Configuration

## OAuth Redirect URLs

The application has been configured to use PeeringDB OAuth2 authentication with the following settings:

### Configuration Details

- **Client ID**: `EzgvjRew4k8Yx8YBW6FkuZAaG4ws9lq4cfRT9kh5`
- **Client Secret**: `Y4DAw45XI16cQpKo97rMdWwiN7WRrCraDgIv6SXmlYrmhRSt9VO0FI6TVmQa3iXk43tjt3NlsIWzq9E3zlMwUMjYgq02uEyaSvlDgmiYVgC0YfpRMFriIYgTppg1iCL8`

### Redirect URLs to Configure in PeeringDB

When setting up the OAuth application in PeeringDB, use the following redirect URI:

```
https://api.peering.austinh.dev/api/oauth/callback
```

### Application URLs

- **API Base URL**: `https://api.peering.austinh.dev`
- **Frontend URL**: `https://peering.austinh.dev`

## OAuth Flow

1. User clicks "Login with PeeringDB OAuth" button on the frontend
2. Frontend requests authorization URL from backend (`/api/oauth/authorize-url`)
3. User is redirected to PeeringDB's authorization page: `https://auth.peeringdb.com/oauth2/authorize/`
4. After user authorizes, PeeringDB redirects back to: `https://api.peering.austinh.dev/api/oauth/callback?code=...`
5. Backend exchanges the authorization code for an access token
6. Backend verifies OIDC ID token using HMAC-SHA256 (if present)
7. Backend redirects user to frontend with access token: `https://peering.austinh.dev/#/login?token=...`
8. Frontend stores the token in local storage

## OIDC Support

The application supports OpenID Connect (OIDC) with HMAC-SHA256 signature verification:

- **Requested Scopes**: `openid profile email`
- **ID Token Verification**: HMAC-SHA256 using the client secret as the key
- **Token Claims Validated**:
  - `aud` (audience) must match the client ID
  - `exp` (expiration) must be in the future
  - Signature must be valid

## Endpoints

### Backend API Endpoints

- `GET /api/oauth/authorize-url` - Returns the OAuth authorization URL
- `GET /api/oauth/callback` - OAuth callback endpoint (receives authorization code)
- `POST /api/auth` - Legacy authentication endpoint (kept for backwards compatibility)
- `POST /api/match-locations` - Location matching endpoint

### Frontend Routes

- `/` - Home page
- `/login` - Login page (OAuth flow starts here)
- `/match` - Location matching page

## Security Features

1. **OAuth2 Authorization Code Flow**: Industry-standard secure authentication
2. **OIDC ID Token Verification**: Additional security layer with HMAC-SHA256
3. **Token Expiration Validation**: Ensures tokens are not expired
4. **Audience Validation**: Prevents token misuse across different applications
5. **Secure Token Storage**: Access tokens stored in browser local storage

## Development vs Production

**Note**: The client secret is currently hardcoded in the source code. In a production environment, you should:

1. Store the client secret in environment variables or secrets management
2. Use Cloudflare Workers secrets for the backend
3. Never commit secrets to source control
4. Rotate secrets regularly

For Cloudflare Workers, you can set secrets using:
```bash
wrangler secret put OAUTH_CLIENT_SECRET
```

And access them in your code with:
```rust
let secret = env.secret("OAUTH_CLIENT_SECRET")?.to_string();
```
