#!/usr/bin/env bash
set -euo pipefail

cd "$(dirname "$0")/../rust-prover"

# Install targets
rustup target add aarch64-apple-ios x86_64-apple-ios aarch64-apple-ios-sim

# Build
cargo build --target aarch64-apple-ios --release
cargo build --target x86_64-apple-ios --release
cargo build --target aarch64-apple-ios-sim --release

# Create xcframework
xcodebuild -create-xcframework \
  -library target/aarch64-apple-ios/release/libprover.a \
  -library target/x86_64-apple-ios/release/libprover.a \
  -library target/aarch64-apple-ios-sim/release/libprover.a \
  -output ../ios/rust-prover.xcframework

echo "✅ iOS xcframework created"