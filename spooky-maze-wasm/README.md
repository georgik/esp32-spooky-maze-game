# Spooky Maze Game - WebAssembly Version

This is a WebAssembly version of the Spooky Maze Game built with Bevy and Rust. The game runs in the browser using WebAssembly for near-native performance.

## Features

- **Procedurally Generated Mazes**: Navigate through randomly generated maze layouts
- **Collectible Coins**: Find and collect coins scattered throughout the maze
- **Enemy NPCs**: Avoid or interact with various enemies
- **Teleportation**: Use special teleport ability to navigate quickly
- **Dynamite**: Place dynamite to destroy obstacles
- **Keyboard & Mouse Controls**: Full support for both input methods

## Building

### Prerequisites

- Rust (latest stable version)
- wasm-pack (will be installed automatically by build script)

### Build Instructions

1. Run the build script:
   ```bash
   ./build.sh
   ```

2. Serve the files with a local HTTP server:
   ```bash
   python3 -m http.server 8000
   ```
   or
   ```bash
   npx serve .
   ```

3. Open your browser and navigate to `http://localhost:8000`

## Controls

### Keyboard Controls
- **Arrow Keys** or **WASD**: Move player
- **Space**: Teleport
- **Enter**: Place dynamite

### Mouse Controls
- Use the on-screen directional buttons for movement
- Click "Teleport" or "Place Dynamite" buttons for special actions

## Game Mechanics

- **Objective**: Navigate through the maze, collect coins, and avoid enemies
- **Movement**: The player moves one tile at a time
- **Collision Detection**: Walls block movement, coins are collected on contact
- **Enemy Behavior**: NPCs move autonomously and can interact with the player

## Technical Details

- **Engine**: Bevy 0.16.1 with WebAssembly support
- **Graphics**: 2D sprite-based rendering at 320x240 resolution
- **Physics**: Tile-based movement system
- **Random Generation**: Procedural maze generation using Rust's `rand` crate

## Architecture

The WASM version shares the core game logic with other platform implementations:

- `spooky-core`: Common game logic and systems
- `spooky-maze-wasm`: WebAssembly-specific implementation
- `wasm_input.rs`: Input handling for keyboard and mouse
- `lib.rs`: Main WebAssembly interface

## Troubleshooting

### Build Issues
- Ensure Rust is installed and up-to-date
- Make sure wasm-pack is properly installed
- Check that all dependencies are available

### Runtime Issues
- Use a modern browser with WebAssembly support
- Ensure the page is served over HTTP (not file://)
- Check browser console for error messages

## Development

To modify the game:

1. Edit the core game logic in `../spooky-core/`
2. Modify WASM-specific code in `src/`
3. Rebuild with `./build.sh`
4. Test in the browser

## License

MIT OR Apache-2.0
