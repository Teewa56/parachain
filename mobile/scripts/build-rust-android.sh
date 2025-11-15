#!/usr/bin/env bash
set -euo pipefail
# Requires cargo-ndk: cargo install cargo-ndk
CRATE_PATH="$(cd "$(dirname "$0")/../.." && pwd)/rust-prover"
cd "$CRATE_PATH"

# Build for arm64 and armeabi-v7a and x86_64
cargo ndk -t arm64-v8a -t armeabi-v7a -t x86_64 --release -- build

# Copy .so to android jniLibs (adjust destination)
DEST="$(cd "$(dirname "$0")/../android/app/src/main/jniLibs" && pwd)"
mkdir -p "$DEST/arm64-v8a" "$DEST/armeabi-v7a" "$DEST/x86_64"
cp target/aarch64-linux-android/release/libprover.so "$DEST/arm64-v8a/libprover.so" || true
cp target/armv7-linux-androideabi/release/libprover.so "$DEST/armeabi-v7a/libprover.so" || true
cp target/x86_64-linux-android/release/libprover.so "$DEST/x86_64/libprover.so" || true

echo "Android .so files copied to $DEST"
