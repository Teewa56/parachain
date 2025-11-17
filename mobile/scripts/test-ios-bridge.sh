#!/usr/bin/env bash
set -euo pipefail

echo "🧪 Testing iOS Bridge for ZK Prover"
echo "===================================="
echo ""

# Build Rust library for iOS
echo "1️⃣ Building Rust library for iOS..."
cd rust-prover
cargo build --target aarch64-apple-ios --release
cargo build --target x86_64-apple-ios --release
cd ..

echo "✅ Rust libraries built"
echo ""

# Create test xcframework
echo "2️⃣ Creating xcframework..."
./scripts/build-rust-ios.sh

echo "✅ xcframework created"
echo ""

# Run iOS unit tests
echo "3️⃣ Running iOS unit tests..."
cd ios
xcodebuild test -workspace mobile.xcworkspace -scheme mobile -destination 'platform=iOS Simulator,name=iPhone 15 Pro'
cd ..

echo "✅ Unit tests passed"
echo ""

echo "✨ iOS Bridge test complete!"