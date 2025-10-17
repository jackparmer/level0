# Level 0 - Bevy 3D WASM Scene

A 3D scene built with Bevy Engine that compiles to WebAssembly (WASM) for web deployment.

## Features

- **3D Scene**: Interactive 3D environment with a controllable cube
- **WASM Support**: Compiles to WebAssembly for browser deployment
- **Collision Detection**: Invisible collision meshes for line-of-sight blocking
- **Dynamic Objects**: Spawning spheres that chase the player
- **Visual Effects**: Smoke particles, emissive materials, and dynamic lighting
- **Camera Controls**: WASD movement with camera following

## Project Structure

```
src/
├── lib.rs          # Main WASM entry point
├── main.rs         # Local development entry point
├── simple_main.rs  # Simplified local version
└── simple_wasm.rs  # Simplified WASM version
```

## Building and Running

### Local Development
```bash
cargo run
```

### WASM Build
```bash
./build.sh
```

### Web Server
```bash
python3 -m http.server 8000
```

Then open `http://localhost:8000/index.html` in your browser.

## Controls

- **WASD**: Move the red cube
- **Shift**: Run faster
- **Arrow Keys**: Adjust camera angle

## Technical Details

- **Engine**: Bevy 0.17
- **Target**: WebAssembly (wasm32-unknown-unknown)
- **Build Tool**: wasm-pack
- **Web Server**: Python HTTP server

## Dependencies

- `bevy = "0.17"` - Game engine
- `wasm-bindgen` - WASM bindings
- `getrandom` - Random number generation

## Notes

- The scene includes transparent collision meshes for accurate line-of-sight detection
- Assets are not included in this repository due to size constraints
- The project is optimized for web deployment with minimal dependencies