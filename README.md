# Bevy 3D Fog Scene with WASM

This project creates a Bevy 0.17 3D scene with fog and compiles it to WebAssembly for browser display.

## Features

- 3D scene with multiple cubes and a ground plane
- Animated fog with linear falloff
- Rotating camera
- Directional lighting with shadows
- Ambient lighting

## Current Status

The project is set up with:
- ✅ Bevy 0.17 with 3D scene and fog
- ✅ WASM compilation configuration
- ✅ HTML template for browser display
- ❌ WASM compilation (blocked by getrandom dependency issue)

## The Issue

The current blocker is that Bevy 0.17 depends on `getrandom` 0.3.3, which doesn't support `wasm32-unknown-unknown` by default. The `getrandom` crate requires the `wasm_js` configuration flag to be set, but this is proving difficult to configure properly.

## Files Created

- `Cargo.toml` - Project configuration with Bevy 0.17
- `src/lib.rs` - Main application code with 3D scene and fog
- `src/main.rs` - Alternative main.rs (not used in WASM build)
- `index.html` - HTML template for browser display
- `build.sh` - Build script using wasm-pack
- `build_trunk.sh` - Alternative build script using trunk
- `.cargo/config.toml` - Cargo configuration for WASM

## Scene Description

The 3D scene includes:
- A central cube (2x2x2)
- A ground plane (20x20)
- Multiple smaller cubes arranged in a grid pattern
- Directional light with shadows
- Ambient lighting
- Camera with animated fog settings
- Rotating camera that orbits the scene

## Fog Configuration

The fog uses linear falloff with:
- Color: Light blue-gray (0.7, 0.8, 0.9)
- Start distance: 5.0 units
- End distance: 20.0 units
- Animated density that varies over time

## Next Steps

To resolve the WASM compilation issue, consider:
1. Using an older version of Bevy that has better WASM support
2. Using a different random number generator
3. Manually patching the getrandom dependency
4. Using a different approach like Bevy's web examples

## Running Locally

For now, you can run the project locally (not in WASM) with:
```bash
cargo run
```

This will show the 3D scene with fog in a native window.
