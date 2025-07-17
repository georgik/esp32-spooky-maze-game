#!/bin/bash

echo "Building Spooky Maze WASM..."

# Check if wasm-pack is installed
if ! command -v wasm-pack &> /dev/null; then
    echo "wasm-pack not found. Installing..."
    curl https://rustwasm.github.io/wasm-pack/installer/init.sh -sSf | sh
fi

# Build the WASM package
wasm-pack build --target web --out-dir pkg --dev

echo "Build complete!"
echo "You can now serve the files with a local server:"
echo "  python3 -m http.server 8000"
echo "  or"
echo "  npx serve ."
echo ""
echo "Then open http://localhost:8000 in your browser."
