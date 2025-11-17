#!/usr/bin/env bash
set -euo pipefail

cd "$(dirname "$0")/.."

echo "🔑 Building key generation tool..."
cargo build --release --bin generate_keys

echo ""
echo "🚀 Running key generation..."
cargo run --release --bin generate_keys

echo ""
echo "📦 Copying keys to mobile app assets..."
cp -r assets/proving-keys ../mobile/assets/

echo ""
echo "✅ Key generation complete!"
echo "📁 Keys location: mobile/assets/proving-keys/"
ls -lh ../mobile/assets/proving-keys/