#!/bin/bash

# Exit immediately if a command exits with a non-zero status.
set -e

echo "Building Bevy 3D Fog Scene for WASM with Trunk..."

# Install trunk if not already installed
if ! command -v trunk &> /dev/null; then
    echo "Installing trunk..."
    cargo install trunk
fi

# Build the project with proper getrandom configuration
echo "Compiling to WASM with Trunk..."
RUSTFLAGS="--cfg getrandom_backend=\"wasm_js\"" trunk build --release --features wasm_js

if [ $? -eq 0 ]; then
    echo "Build successful!"
    echo "To view the scene, serve the files with a web server:"
    echo "  trunk serve --release"
else
    echo "Build failed!"
fi