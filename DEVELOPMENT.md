# Development Guide

## Local Development Setup

### Prerequisites

1. Install Rust (1.93+):
   ```bash
   curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
   ```

2. Add wasm32 target:
   ```bash
   rustup target add wasm32-unknown-unknown
   ```

3. Install development tools:
   ```bash
   # Trunk for frontend development
   cargo install trunk
   
   # wrangler for backend development
   npm install -g wrangler
   ```

### Running Backend Locally

```bash
cd backend
wrangler dev
```

The backend will be available at `http://localhost:8787`

### Running Frontend Locally

```bash
cd frontend
trunk serve
```

The frontend will be available at `http://localhost:8080`

**Note**: For local development, you may need to update `frontend/src/api.rs` to point to your local backend:

```rust
const API_BASE: &str = "http://localhost:8787/api";
```

## Project Structure

```
bgp-peering/
├── backend/              # Cloudflare Worker
│   ├── src/
│   │   ├── lib.rs       # Main router and API endpoints
│   │   ├── peeringdb.rs # PeeringDB API integration
│   │   └── location_matcher.rs # Location matching logic
│   ├── Cargo.toml
│   └── wrangler.toml
├── frontend/             # Yew SPA
│   ├── src/
│   │   ├── main.rs      # App entry point and routing
│   │   ├── api.rs       # API client
│   │   ├── pages/       # Page components
│   │   └── components/  # Reusable components
│   ├── Cargo.toml
│   ├── index.html
│   └── styles.css
└── README.md
```

## Development Workflow

### Adding New API Endpoints

1. Add the endpoint handler in `backend/src/lib.rs`
2. Add corresponding client function in `frontend/src/api.rs`
3. Use the API function in your Yew components

Example:

```rust
// backend/src/lib.rs
router.post_async("/api/my-endpoint", |mut req, _ctx| async move {
    // Handler logic
})

// frontend/src/api.rs
pub async fn my_api_call() -> Result<Response, String> {
    // Client logic
}
```

### Adding New Pages

1. Create a new file in `frontend/src/pages/`
2. Add the page component
3. Update `frontend/src/pages/mod.rs` to export it
4. Add a route in `frontend/src/main.rs`

Example:

```rust
// frontend/src/pages/my_page.rs
use yew::prelude::*;

#[function_component(MyPage)]
pub fn my_page() -> Html {
    html! {
        <div>{ "My Page" }</div>
    }
}

// frontend/src/pages/mod.rs
pub use my_page::MyPage;

// frontend/src/main.rs
#[derive(Clone, Routable, PartialEq)]
enum Route {
    #[at("/my-page")]
    MyPage,
    // ...
}

fn switch(routes: Route) -> Html {
    match routes {
        Route::MyPage => html! { <MyPage /> },
        // ...
    }
}
```

## Testing

### Backend Tests

```bash
cd backend
cargo test
```

### Frontend Tests

```bash
cd frontend
cargo test --target wasm32-unknown-unknown
```

### Integration Testing

For integration testing, you can use tools like:
- Playwright for end-to-end testing
- wasm-pack for WASM testing
- HTTP clients for API testing

## Code Style

This project follows standard Rust conventions:

```bash
# Format code
cargo fmt

# Run clippy
cargo clippy

# Check for issues
cargo check
```

## Common Development Tasks

### Hot Reload

Both `wrangler dev` and `trunk serve` support hot reload:
- Backend: Changes trigger automatic rebuilds
- Frontend: Changes trigger automatic rebuilds and browser refresh

### Debugging

**Backend:**
- Use `console_log!()` macro from worker crate
- Check logs with `wrangler tail`

**Frontend:**
- Use `web_sys::console::log_1()` for browser console
- Use browser DevTools for debugging
- Check Network tab for API calls

### Adding Dependencies

**Backend:**
```bash
cd backend
cargo add <dependency>
```

**Frontend:**
```bash
cd frontend
cargo add <dependency>
```

## Environment Setup

### VS Code Extensions

Recommended extensions:
- rust-analyzer
- CodeLLDB (for debugging)
- Even Better TOML
- Trunk (for Yew projects)

### .vscode/settings.json

```json
{
  "rust-analyzer.check.command": "clippy",
  "rust-analyzer.cargo.features": "all"
}
```

## Troubleshooting

### "Cannot find module" errors

Clear cargo cache and rebuild:
```bash
cargo clean
cargo build
```

### WASM errors

Ensure wasm32 target is installed:
```bash
rustup target add wasm32-unknown-unknown
```

### Port already in use

Change the port in development:
```bash
# Backend
wrangler dev --port 8788

# Frontend
trunk serve --port 8081
```

## Resources

- [Cloudflare Workers Docs](https://developers.cloudflare.com/workers/)
- [Yew Documentation](https://yew.rs/)
- [PeeringDB API Docs](https://www.peeringdb.com/apidocs/)
- [Rust Book](https://doc.rust-lang.org/book/)
- [WebAssembly Docs](https://webassembly.org/)
