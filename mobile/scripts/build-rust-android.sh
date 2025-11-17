#!/usr/bin/env bash
set -euo pipefail

cd "$(dirname "$0")/../rust-prover"

# Install targets
rustup target add aarch64-linux-android armv7-linux-androideabi x86_64-linux-android

# Build for all architectures
cargo ndk -t arm64-v8a -t armeabi-v7a -t x86_64 --release -- build --features android

# Copy to jniLibs
DEST="../android/app/src/main/jniLibs"
mkdir -p "$DEST/arm64-v8a" "$DEST/armeabi-v7a" "$DEST/x86_64"

cp target/aarch64-linux-android/release/libprover.so "$DEST/arm64-v8a/" || true
cp target/armv7-linux-androideabi/release/libprover.so "$DEST/armeabi-v7a/" || true
cp target/x86_64-linux-android/release/libprover.so "$DEST/x86_64/" || true

echo "✅ Android .so files built and copied"