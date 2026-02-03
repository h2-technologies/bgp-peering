# Deployment Guide

This guide covers deploying the BGP Peering Portal to Cloudflare.

## Prerequisites

- Cloudflare account
- wrangler CLI installed: `npm install -g wrangler`
- Rust toolchain (1.93+)
- trunk CLI: `cargo install trunk`
- wasm32-unknown-unknown target: `rustup target add wasm32-unknown-unknown`

## Quick Start

### 1. Build Both Components

```bash
./build.sh
```

Or build individually:

```bash
# Backend
cd backend
cargo build --release

# Frontend
cd frontend
cargo build --target wasm32-unknown-unknown --release
```

## Backend Deployment

The backend is a Cloudflare Worker that provides the API endpoints.

### Step 1: Login to Cloudflare

```bash
cd backend
wrangler login
```

### Step 2: Deploy the Worker

```bash
wrangler deploy
```

This will deploy the worker to your Cloudflare account. Take note of the worker URL (e.g., `https://peering-portal-backend.your-subdomain.workers.dev`).

### Step 3: Configure Custom Domain (Optional)

```bash
wrangler domains add <your-custom-domain>
```

## Frontend Deployment

The frontend is a static Yew application that can be deployed to Cloudflare Pages.

### Step 1: Update API Base URL

Before building the frontend for production, update the API base URL in `frontend/src/api.rs`:

```rust
// Change this to your deployed worker URL
const API_BASE: &str = "https://peering-portal-backend.your-subdomain.workers.dev/api";
```

### Step 2: Build the Frontend

```bash
cd frontend
trunk build --release
```

This creates a `dist/` directory with the compiled application.

### Step 3: Deploy to Cloudflare Pages

#### Option A: Using wrangler

```bash
wrangler pages deploy dist --project-name=peering-portal
```

#### Option B: Using Cloudflare Dashboard

1. Go to Cloudflare Dashboard > Pages
2. Create a new project
3. Connect your GitHub repository
4. Configure build settings:
   - Build command: `cd frontend && trunk build --release`
   - Build output directory: `frontend/dist`
   - Root directory: `/`

5. Deploy

#### Option C: Manual Upload

1. Go to Cloudflare Dashboard > Pages
2. Create a new project
3. Upload the `frontend/dist/` folder directly

### Step 4: Configure Custom Domain (Optional)

In the Cloudflare Pages dashboard, add a custom domain for your frontend.

## Environment Variables

### Backend

You can add environment variables to your worker in `backend/wrangler.toml`:

```toml
[env.production.vars]
PEERINGDB_API_KEY = "your_api_key"
```

Or using wrangler:

```bash
wrangler secret put PEERINGDB_API_KEY
```

### Frontend

For environment-specific configuration, you can create different build profiles and update the API base URL accordingly.

## Production Considerations

### Security

1. **Enable CORS**: Update the backend to properly handle CORS for your frontend domain
2. **API Authentication**: Implement proper OAuth2 flow with PeeringDB
3. **Rate Limiting**: Add rate limiting to API endpoints
4. **Input Validation**: Ensure all inputs are validated

### Performance

1. **Caching**: Add caching for PeeringDB API responses
2. **CDN**: Cloudflare Pages automatically uses the CDN
3. **Minification**: trunk automatically minifies the WASM output in release mode

### Monitoring

1. Enable observability in `backend/wrangler.toml`
2. Use Cloudflare Analytics for Pages
3. Set up Cloudflare Workers Analytics

## Testing Deployment

After deployment:

1. Visit your frontend URL
2. Test the login functionality
3. Test the location matching feature
4. Check browser console for errors
5. Verify API calls in Network tab

## Troubleshooting

### CORS Errors

Add CORS headers to the backend responses:

```rust
response
    .headers_mut()
    .set("Access-Control-Allow-Origin", "https://your-frontend-domain.com")?;
```

### API Not Found (404)

Ensure the API_BASE URL in `frontend/src/api.rs` matches your deployed worker URL.

### Build Failures

- Ensure Rust toolchain is up to date: `rustup update`
- Clear cargo cache: `cargo clean`
- Reinstall trunk: `cargo install trunk --force`

## Rollback

### Backend

```bash
cd backend
wrangler rollback
```

### Frontend (Cloudflare Pages)

Use the Cloudflare Dashboard to rollback to a previous deployment.

## CI/CD

You can automate deployments using GitHub Actions. Create `.github/workflows/deploy.yml`:

```yaml
name: Deploy

on:
  push:
    branches: [main]

jobs:
  deploy-backend:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3
      - uses: actions-rs/toolchain@v1
        with:
          toolchain: stable
      - name: Deploy Worker
        run: |
          cd backend
          npx wrangler deploy
        env:
          CLOUDFLARE_API_TOKEN: ${{ secrets.CLOUDFLARE_API_TOKEN }}

  deploy-frontend:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3
      - uses: actions-rs/toolchain@v1
        with:
          toolchain: stable
          target: wasm32-unknown-unknown
      - name: Install trunk
        run: cargo install trunk
      - name: Build
        run: |
          cd frontend
          trunk build --release
      - name: Deploy to Pages
        run: npx wrangler pages deploy frontend/dist --project-name=peering-portal
        env:
          CLOUDFLARE_API_TOKEN: ${{ secrets.CLOUDFLARE_API_TOKEN }}
```

## Support

For issues or questions:
- Check the README.md files in backend/ and frontend/
- Review Cloudflare Workers documentation
- Check Yew framework documentation
