#!/bin/bash
# Script to prepare a new release for Homebrew

set -e

VERSION=$1

if [ -z "$VERSION" ]; then
    echo "Usage: ./prepare-release.sh <version>"
    echo "Example: ./prepare-release.sh 1.0.0"
    exit 1
fi

echo "🚀 Preparing release v${VERSION}"

# Ensure we're in the project root
cd "$(dirname "$0")/.."

# Build and test
echo "📦 Building project..."
cd backend
dotnet build --configuration Release
dotnet test

echo "✅ Tests passed"

# Create tag
echo "🏷️  Creating git tag v${VERSION}..."
git tag -a "v${VERSION}" -m "Release v${VERSION}"

echo ""
echo "✅ Release prepared!"
echo ""
echo "Next steps:"
echo "1. Push the tag: git push origin v${VERSION}"
echo "2. Create GitHub release at: https://github.com/GriffinCanCode/Dei/releases/new"
echo "3. Download tarball and calculate SHA256:"
echo "   curl -L https://github.com/GriffinCanCode/Dei/archive/refs/tags/v${VERSION}.tar.gz -o dei-${VERSION}.tar.gz"
echo "   shasum -a 256 dei-${VERSION}.tar.gz"
echo "4. Update homebrew-dei/Formula/dei-cs.rb with new version and SHA256"
echo "5. Test formula: brew install --build-from-source ./homebrew-dei/Formula/dei-cs.rb"
echo "6. Push formula: cd homebrew-dei && git add . && git commit -m 'Release v${VERSION}' && git push"

