#!/bin/bash
# Build script for the BGP Peering Portal

set -e

echo "Building BGP Peering Portal..."
echo

echo "1. Building Backend..."
cd backend
cargo build --release
cd ..
echo "✓ Backend built successfully"
echo

echo "2. Building Frontend..."
cd frontend
cargo build --target wasm32-unknown-unknown --release
echo "✓ Frontend built successfully"
cd ..

echo
echo "Build completed successfully!"
echo
echo "Next steps:"
echo "  - Backend: cd backend && wrangler deploy"
echo "  - Frontend: cd frontend && trunk build --release"
echo "             (or deploy the dist/ folder to Cloudflare Pages)"
