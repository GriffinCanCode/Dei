# dei-e2e

End-to-end testing suite for the dei code analyzer.

## Overview

This crate provides comprehensive E2E tests that exercise the entire analysis pipeline with real-world scenarios. It includes:

- **Realistic test fixtures**: Real Rust and C# code samples representing healthy code, god classes, and god methods
- **Full pipeline tests**: Tests that run the complete analysis from filesystem traversal to result generation
- **CLI integration tests**: Tests that verify CLI behavior in real-world scenarios
- **Edge case tests**: Tests for unusual inputs, error handling, and boundary conditions
- **Performance benchmarks**: Benchmarks measuring real-world performance

## Running Tests

### Run all E2E tests
```bash
cargo test -p dei-e2e
```

### Run specific test suites
```bash
# Pipeline tests
cargo test -p dei-e2e --test pipeline_tests

# CLI tests
cargo test -p dei-e2e --test cli_tests

# Edge case tests
cargo test -p dei-e2e --test edge_cases
```

### Run with output
```bash
cargo test -p dei-e2e -- --nocapture
```

## Running Benchmarks

```bash
# Run all benchmarks
cargo bench -p dei-e2e

# Run specific benchmark
cargo bench -p dei-e2e --bench e2e_benchmarks -- single_file

# Generate HTML report
cargo bench -p dei-e2e
# Report available at target/criterion/report/index.html
```

## Test Fixtures

### Rust Fixtures
- `fixtures/rust/healthy.rs` - Well-structured code that should pass all checks
- `fixtures/rust/god_class.rs` - Example of a god class with too many responsibilities
- `fixtures/rust/god_method.rs` - Example of god methods that are too complex

### C# Fixtures
- `fixtures/csharp/Healthy.cs` - Well-structured C# code
- `fixtures/csharp/GodClass.cs` - Example of a C# god class

## Test Categories

### Pipeline Tests (`tests/pipeline_tests.rs`)
- Healthy code validation
- God class detection
- God method detection
- Multi-language support
- Nested directory traversal
- Threshold boundary testing
- Complexity scoring

### CLI Tests (`tests/cli_tests.rs`)
- Command-line argument parsing
- Exit codes
- JSON output format
- Progress indicators
- `.gitignore` respect
- Error handling

### Edge Cases (`tests/edge_cases.rs`)
- Empty files
- Comment-only files
- Unicode support
- Binary file handling
- Symlink handling
- Permission errors
- Very large files
- Extreme thresholds
- Concurrent analysis

### Benchmarks (`benches/e2e_benchmarks.rs`)
- Single file analysis
- Directory analysis (10, 50, 100 files)
- AST building
- Parsing performance
- Parallel traversal
- Large file handling
- Multi-language analysis
- Threshold variations

## Test Utilities

### `FixtureManager`
Manages temporary test fixtures:
```rust
let fixture = FixtureManager::new()?;
let path = fixture.copy_fixture("rust")?;
fixture.create_file("test.rs", "pub struct Test;")?;
```

### `TestHarness`
Simplified interface for running analysis:
```rust
let harness = TestHarness::new()?;
let results = harness.analyze_path("src/")?;
assert!(!harness.has_god_classes("src/")?);
```

### `ThresholdBuilder`
Build custom thresholds for tests:
```rust
let thresholds = ThresholdBuilder::new()
    .max_class_lines(100)
    .max_methods(10)
    .build();
```

## Continuous Integration

These tests are designed to run in CI environments. They:
- Use temporary directories (auto-cleanup)
- Are deterministic and reproducible
- Handle platform differences (Unix vs Windows)
- Run quickly (most tests < 1 second)

## Performance Expectations

Based on benchmarks, dei should:
- Analyze a single file in < 10ms
- Analyze 100 files in < 500ms
- Handle 1000+ line files efficiently
- Scale linearly with file count (thanks to parallelism)

## Adding New Tests

When adding new tests:
1. Add fixture files to `fixtures/` if needed
2. Use `FixtureManager` for temporary files
3. Test both success and failure cases
4. Include edge cases
5. Add benchmarks for performance-critical paths

## Known Platform Differences

Some tests may behave differently on different platforms:
- Symlink tests may skip on Windows
- Permission tests are Unix-specific
- Line ending handling varies by platform

These tests gracefully handle platform differences and won't cause false failures.

