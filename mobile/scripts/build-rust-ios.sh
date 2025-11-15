#!/usr/bin/env bash
set -euo pipefail
CRATE_PATH="$(cd "$(dirname "$0")/../.." && pwd)/rust-prover"
cd "$CRATE_PATH"

# Build for device + simulator
rustup target add aarch64-apple-ios x86_64-apple-ios aarch64-apple-ios-sim || true

cargo build --target aarch64-apple-ios --release
cargo build --target x86_64-apple-ios --release

# Create xcframework (simplified; use cargo-lipo/cbindgen for production)
mkdir -p ../ios/rust-prover.xcframework/ios-arm64
cp target/aarch64-apple-ios/release/libprover.a ../ios/rust-prover.xcframework/ios-arm64/libprover.a || true
mkdir -p ../ios/rust-prover.xcframework/ios-x86_64-simulator
cp target/x86_64-apple-ios/release/libprover.a ../ios/rust-prover.xcframework/ios-x86_64-simulator/libprover.a || true

echo "Created ios/rust-prover.xcframework (manual step may be required to make a proper xcframework)"
