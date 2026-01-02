#!/bin/bash
# Publish script for DEI crates
# Must be run from the repository root

set -e

echo "🚀 Publishing DEI to crates.io..."
echo ""

# Check if logged in to crates.io
if ! cargo login --help > /dev/null 2>&1; then
    echo "❌ cargo not found. Please install Rust."
    exit 1
fi

echo "📦 Publishing crates in dependency order..."
echo ""

# Publish in order of dependencies
echo "1️⃣  Publishing dei-core..."
cd crates/dei-core
cargo publish
cd ../..
echo "✅ dei-core published"
echo ""

echo "⏳ Waiting 10 seconds for crates.io to process..."
sleep 10
echo ""

echo "2️⃣  Publishing dei-ast..."
cd crates/dei-ast
cargo publish
cd ../..
echo "✅ dei-ast published"
echo ""

echo "⏳ Waiting 10 seconds for crates.io to process..."
sleep 10
echo ""

echo "3️⃣  Publishing dei-metrics..."
cd crates/dei-metrics
cargo publish
cd ../..
echo "✅ dei-metrics published"
echo ""

echo "⏳ Waiting 10 seconds for crates.io to process..."
sleep 10
echo ""

echo "4️⃣  Publishing dei-clustering..."
cd crates/dei-clustering
cargo publish
cd ../..
echo "✅ dei-clustering published"
echo ""

echo "⏳ Waiting 10 seconds for crates.io to process..."
sleep 10
echo ""

echo "5️⃣  Publishing dei-languages..."
cd crates/dei-languages
cargo publish
cd ../..
echo "✅ dei-languages published"
echo ""

echo "⏳ Waiting 10 seconds for crates.io to process..."
sleep 10
echo ""

echo "6️⃣  Publishing dei (main CLI)..."
cd crates/dei-cli
cargo publish
cd ../..
echo "✅ dei (CLI) published"
echo ""

echo "🎉 All crates published successfully!"
echo ""
echo "📝 Next steps:"
echo "  1. Create a git tag: git tag -a v0.1.1 -m 'Release v0.1.1'"
echo "  2. Push the tag: git push origin v0.1.1"
echo "  3. Create a GitHub release from the tag"


