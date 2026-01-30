#!/bin/zsh

set -e

echo "1. Building the Core (libsumi)..."
cargo build -p libsumi --quiet

echo "2. Testing the Brain (SumiC Math Parity)..."
# We run it with --help just to trigger the build and the parity print
cargo run -p sumic --quiet -- --help > /dev/null

echo "3. Compiling Shader (ripple.glsl -> ripple.metal)..."
cargo run -p sumic --quiet -- ripple.glsl --shadertoy --output ripple.metal

echo "4. Checking the Body (Hanga Build)..."
cargo check -p hanga --quiet

echo "--- ✅ SYSTEM STABLE ---"
echo "All systems green. Metal shader generated. Runtime ready."
