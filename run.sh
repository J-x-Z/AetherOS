#!/bin/bash
# AetherOS Build and Run Script

set -e

echo "=== Building Guest (aarch64) ==="
cd apps/hello_world
RUSTFLAGS="-C link-arg=-T$(pwd)/layout.ld -C link-arg=-n" cargo build --release --target aarch64-unknown-none
cd ../..
rust-objcopy -O binary target/aarch64-unknown-none/release/hello_world apps/hello_world/guest-aarch64.bin

echo "=== Building Guest (x86_64) ==="
cd apps/hello_world
# x86_64 shares the same layout.ld for flat binary
RUSTFLAGS="-C link-arg=-T$(pwd)/layout.ld -C link-arg=-n -C no-redzone=yes" cargo build --release --target x86_64-unknown-none
cd ../..
rust-objcopy -O binary target/x86_64-unknown-none/release/hello_world apps/hello_world/guest-x86_64.bin

ls -lh apps/hello_world/guest-*.bin

echo "=== Building WASM Test App ==="
cd apps/wasm_simple
cargo build --release --target wasm32-unknown-unknown
cd ../..
wasm_path="target/wasm32-unknown-unknown/release/wasm_simple.wasm"

echo "=== Creating Disk Image (with WASM) ==="
cargo run --release -p mkext2 -- "$wasm_path"

echo "=== Building Kernel ==="
# We default to aarch64 guest on aarch64 host for now in macos.rs
cargo build --release -p aether

# 3. Sign & Run (Platform Specific)
OS="$(uname -s)"
if [ "$OS" = "Darwin" ]; then
    echo "=== Signing Kernel (macOS) ==="
    codesign --entitlements kernel/entitlements.plist --force -s - target/release/aether
    
    echo "=== Running AetherOS (macOS) ==="
    ./target/release/aether

elif [ "$OS" = "Linux" ]; then
    echo "=== Running AetherOS (Linux) ==="
    if [ -w /dev/kvm ]; then
        ./target/release/aether
    else
        echo "Warning: /dev/kvm is not writable. Trying sudo..."
        sudo ./target/release/aether
    fi
    
elif [[ "$OS" == CYGWIN* || "$OS" == MINGW* || "$OS" == MSYS* ]]; then
    echo "=== Running AetherOS (Windows) ==="
    ./target/release/aether.exe
    
else
    echo "Unknown OS: $OS. Attempting to run..."
    ./target/release/aether
fi
