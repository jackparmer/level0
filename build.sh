#!/bin/bash

# Exit immediately if a command exits with a non-zero status.
set -e

echo "Building Bevy 3D Fog Scene for WASM..."

# Install wasm-pack if not already installed
if ! command -v wasm-pack &> /dev/null; then
    echo "Installing wasm-pack..."
    curl https://rustwasm.github.io/wasm-pack/installer/init.sh -sSf | sh
fi

# Build the project with proper getrandom configuration
echo "Compiling to WASM..."
RUSTFLAGS="--cfg getrandom_backend=\"wasm_js\"" wasm-pack build --target web --out-dir pkg --dev -- --features wasm_js

if [ $? -eq 0 ]; then
    echo "Build successful!"
    echo "To view the scene, serve the files with a web server:"
    echo "  python3 -m http.server 8000"
    echo "  Then open http://localhost:8000/index.html"
else
    echo "Build failed!"
fi