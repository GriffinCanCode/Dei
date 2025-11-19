#!/bin/bash
# Script to test Homebrew formula locally

set -e

FORMULA=${1:-dei-cs}

echo "🧪 Testing Homebrew formula: ${FORMULA}"

# Ensure we're in the project root
cd "$(dirname "$0")/.."

# Check if formula exists
if [ ! -f "homebrew-dei/Formula/${FORMULA}.rb" ]; then
    echo "❌ Formula not found: homebrew-dei/Formula/${FORMULA}.rb"
    exit 1
fi

echo "📋 Auditing formula..."
brew audit --strict homebrew-dei/Formula/${FORMULA}.rb

echo "🎨 Checking style..."
brew style homebrew-dei/Formula/${FORMULA}.rb

echo "🔨 Installing from source..."
brew install --build-from-source homebrew-dei/Formula/${FORMULA}.rb

echo "✅ Testing installation..."
brew test ${FORMULA}

echo "🧹 Cleaning up..."
brew uninstall ${FORMULA}

echo ""
echo "✅ All tests passed for ${FORMULA}!"

