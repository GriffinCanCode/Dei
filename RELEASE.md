# Release Guide for Dei

This guide covers releasing both the C# version (dei-cs) and the future Rust version (dei-rs) to Homebrew.

## Prerequisites

1. GitHub account with access to the repositories
2. Git installed and configured
3. Homebrew installed (for testing)
4. .NET 9.0 SDK (for dei-cs)
5. Rust toolchain (for dei-rs, when ready)

## Repository Setup

### Main Repository (Dei)
Contains the C# implementation in the `backend/` directory.

### Homebrew Tap (homebrew-dei)
Contains the Homebrew formulas for both versions.

**To publish the tap:**
```bash
cd homebrew-dei
git init
git add .
git commit -m "Initial commit: Homebrew formulas for dei-cs and dei-rs"
git branch -M main
git remote add origin https://github.com/GriffinCanCode/homebrew-dei.git
git push -u origin main
```

## Releasing dei-cs (C# Version)

### 1. Prepare the Release

```bash
cd backend

# Ensure everything builds
dotnet build --configuration Release

# Run tests
dotnet test

# Test the CLI
cd src/GodClassDetector.Console
dotnet run check ../..
```

### 2. Create a GitHub Release

```bash
# Tag the release
git tag -a v1.0.0 -m "Release v1.0.0 - Initial C# version"
git push origin v1.0.0
```

Then on GitHub:
1. Go to Releases → Draft a new release
2. Choose tag v1.0.0
3. Title: "v1.0.0 - Initial Release"
4. Add release notes (see template below)
5. Publish release

### 3. Calculate SHA256 for Homebrew Formula

```bash
# Download the tarball
curl -L https://github.com/GriffinCanCode/Dei/archive/refs/tags/v1.0.0.tar.gz -o dei-1.0.0.tar.gz

# Calculate SHA256
shasum -a 256 dei-1.0.0.tar.gz
```

### 4. Update Homebrew Formula

Edit `homebrew-dei/Formula/dei-cs.rb`:

```ruby
url "https://github.com/GriffinCanCode/Dei/archive/refs/tags/v1.0.0.tar.gz"
sha256 "PASTE_SHA256_HERE"
```

### 5. Test the Formula Locally

```bash
# Install from local tap
brew install --build-from-source ./homebrew-dei/Formula/dei-cs.rb

# Test it
dei-cs check .

# Uninstall
brew uninstall dei-cs
```

### 6. Publish to Homebrew Tap

```bash
cd homebrew-dei
git add Formula/dei-cs.rb
git commit -m "Release dei-cs v1.0.0"
git push origin main
```

### 7. Install from Tap

```bash
brew tap GriffinCanCode/dei
brew install dei-cs
```

## Releasing dei-rs (Rust Version) [Future]

### 1. Create Rust Repository

```bash
# Create new repository
git clone https://github.com/GriffinCanCode/dei-rs.git
cd dei-rs

# Initialize Cargo project
cargo init --name dei-rs

# Add to Cargo.toml
[package]
name = "dei-rs"
version = "1.0.0"
edition = "2021"
authors = ["Your Name <email@example.com>"]
description = "High-performance god class detector written in Rust"
license = "MIT"
repository = "https://github.com/GriffinCanCode/dei-rs"
```

### 2. Build and Test

```bash
cargo build --release
cargo test
./target/release/dei-rs check .
```

### 3. Create GitHub Release

```bash
git tag -a v1.0.0 -m "Release v1.0.0"
git push origin v1.0.0
```

### 4. Update Homebrew Formula

Same process as dei-cs:
1. Get tarball URL and SHA256
2. Update `homebrew-dei/Formula/dei-rs.rb`
3. Test locally
4. Push to tap

## Release Notes Template

```markdown
## v1.0.0 - Initial Release

### Features
- AST-based parallel file system traversal
- God Class detection with configurable thresholds
- God File detection (files with too many classes)
- God Method detection (overly complex methods)
- Semantic clustering using K-means for refactoring suggestions
- Rich console output with tree visualization
- Configurable thresholds via appsettings.json

### Thresholds
- Max Lines per Class: 300
- Max Methods per Class: 20
- Max Complexity per Class: 50
- Max Method Lines: 50
- Max Method Complexity: 10
- Max Method Parameters: 5
- Max Classes per File: 3
- Max File Lines: 500

### Installation
\`\`\`bash
brew tap GriffinCanCode/dei
brew install dei-cs
\`\`\`

### Usage
\`\`\`bash
dei-cs check /path/to/your/project
\`\`\`

### Requirements
- .NET 9.0 Runtime (automatically installed via Homebrew)
```

## Updating Formulas

### Version Bump

1. Update version in the project
2. Create new GitHub release
3. Update formula with new URL and SHA256
4. Test locally
5. Push to tap

### Formula Maintenance

```bash
# Audit formula
brew audit --strict --online dei-cs

# Style check
brew style dei-cs

# Test installation
brew install --build-from-source dei-cs
brew test dei-cs
brew uninstall dei-cs
```

## Troubleshooting

### Formula fails to install

```bash
# Check logs
brew install --debug --verbose dei-cs

# Clean up
brew cleanup dei-cs
```

### SHA256 mismatch

```bash
# Recalculate
curl -L [URL] | shasum -a 256
```

### .NET runtime issues

```bash
# Check .NET installation
dotnet --info

# Reinstall .NET
brew reinstall dotnet
```

## CI/CD

The tap includes GitHub Actions workflows that:
- Validate formula syntax
- Test installation on Ubuntu and macOS
- Build bottles for distribution

Bottles (pre-built binaries) speed up installation by avoiding compilation.

## Publishing to Homebrew Core (Optional, Future)

To get dei into the main Homebrew repository:

1. Build stable userbase (100+ stars, active issues/PRs)
2. Ensure formula passes all audits
3. Submit PR to homebrew-core
4. Address reviewer feedback
5. Wait for approval and merge

Requirements:
- Stable, documented API
- Active maintenance
- Good test coverage
- Follows Homebrew guidelines

## Links

- Main Repo: https://github.com/GriffinCanCode/Dei
- Homebrew Tap: https://github.com/GriffinCanCode/homebrew-dei
- Rust Version: https://github.com/GriffinCanCode/dei-rs (coming soon)

