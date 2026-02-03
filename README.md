# bgp-peering

BGP Peering Portal - A web application for requesting BGP peering on networks with common presence.

## Overview

This application uses PeeringDB to authenticate users and checks locations where ASNs are present to identify common peering opportunities.

## Project Structure

The project is split into two main components:

### Backend (`/backend`)
- Cloudflare Worker implementation
- PeeringDB API integration
- Location matching logic
- RESTful API endpoints

### Frontend (`/frontend`)
- Yew-based SPA (Single Page Application)
- Modern Rust WebAssembly frontend
- Responsive UI with routing

## Features

- 🔐 **PeeringDB Authentication**: Login with PeeringDB credentials
- 🌍 **Location Matching**: Find common facilities between ASNs
- 🤝 **Easy Peering Requests**: Submit peering requests where networks have common presence
- ⚡ **Fast & Secure**: Built with Rust for both frontend and backend

## Getting Started

### Prerequisites

- Rust 1.93+ and Cargo
- wasm32-unknown-unknown target: `rustup target add wasm32-unknown-unknown`
- Trunk (for frontend): `cargo install trunk`
- wrangler CLI (for backend deployment): `npm install -g wrangler`

### Backend Development

```bash
cd backend
cargo build
wrangler dev  # Start local development server
```

### Frontend Development

```bash
cd frontend
trunk serve  # Start development server at http://localhost:8080
```

### Deployment

**Backend:**
```bash
cd backend
wrangler deploy
```

**Frontend:**
```bash
cd frontend
trunk build --release
# Deploy the dist/ directory to Cloudflare Pages or any static host
```

## API Documentation

See [backend/README.md](backend/README.md) for API endpoint documentation.

## Technologies

- **Backend**: Rust, Cloudflare Workers, worker-rs
- **Frontend**: Rust, Yew, WebAssembly
- **APIs**: PeeringDB API

## License

See LICENSE file for details.

