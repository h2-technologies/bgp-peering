# Peering Portal Frontend

This is a Yew-based frontend for the BGP Peering Portal.

## Features

- Modern SPA built with Yew (Rust WebAssembly framework)
- PeeringDB authentication
- ASN location comparison
- Responsive design

## Pages

- **Home**: Landing page with features overview
- **Login**: PeeringDB authentication
- **Location Match**: Compare locations between ASNs

## Development

Install trunk (Yew build tool):
```bash
cargo install trunk
```

Install wasm target:
```bash
rustup target add wasm32-unknown-unknown
```

Run development server:
```bash
trunk serve
```

Build for production:
```bash
trunk build --release
```

The built files will be in the `dist/` directory.

## Deployment

The frontend can be deployed to:
- Cloudflare Pages
- GitHub Pages
- Netlify
- Any static hosting service

For Cloudflare Pages integration with the backend worker:
1. Deploy the backend worker first
2. Update the API_BASE URL in `src/api.rs` if needed
3. Build the frontend with `trunk build --release`
4. Deploy the `dist/` directory to Cloudflare Pages

## Technologies

- **Yew**: Rust framework for building web applications
- **yew-router**: Client-side routing
- **gloo-net**: HTTP client for making API requests
- **wasm-bindgen**: Rust/WASM bindings
