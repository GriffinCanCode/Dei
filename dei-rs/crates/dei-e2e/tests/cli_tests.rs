//! CLI integration tests
//!
//! These tests verify the CLI behaves correctly in real-world scenarios.

use anyhow::Result;
use assert_cmd::Command;
use dei_e2e::FixtureManager;
use predicates::prelude::*;

#[test]
fn test_cli_check_healthy_code() -> Result<()> {
    let fixture = FixtureManager::new()?;
    let path = fixture.copy_fixture("rust")?;
    
    let mut cmd = Command::cargo_bin("dei")?;
    cmd.arg("check")
        .arg(path.join("healthy.rs"))
        .arg("--format")
        .arg("text");
    
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("DEI - CODE ANALYSIS"));
    
    Ok(())
}

#[test]
fn test_cli_check_god_class_fails() -> Result<()> {
    let fixture = FixtureManager::new()?;
    let path = fixture.copy_fixture("rust")?;
    
    let mut cmd = Command::cargo_bin("dei")?;
    cmd.arg("check")
        .arg(path.join("god_class.rs"))
        .arg("--max-lines")
        .arg("100")
        .arg("--max-methods")
        .arg("10");
    
    // Should exit with code 1 when issues found
    cmd.assert()
        .failure()
        .stdout(predicate::str::contains("GOD CLASS"));
    
    Ok(())
}

#[test]
fn test_cli_check_json_output() -> Result<()> {
    let fixture = FixtureManager::new()?;
    let path = fixture.copy_fixture("rust")?;
    
    let mut cmd = Command::cargo_bin("dei")?;
    cmd.arg("check")
        .arg(path.join("healthy.rs"))
        .arg("--format")
        .arg("json");
    
    let output = cmd.assert().success();
    
    // Output should be valid JSON
    let stdout = String::from_utf8(output.get_output().stdout.clone())?;
    
    // Try to parse as JSON to verify format
    let _parsed: serde_json::Value = serde_json::from_str(&stdout)?;
    
    Ok(())
}

#[test]
fn test_cli_check_custom_thresholds() -> Result<()> {
    let fixture = FixtureManager::new()?;
    let path = fixture.copy_fixture("rust")?;
    
    // Very strict thresholds
    let mut cmd = Command::cargo_bin("dei")?;
    cmd.arg("check")
        .arg(path.join("god_class.rs"))
        .arg("--max-lines")
        .arg("50")
        .arg("--max-methods")
        .arg("5")
        .arg("--max-complexity")
        .arg("10");
    
    cmd.assert()
        .failure(); // Should fail with strict thresholds
    
    // Very lenient thresholds
    let mut cmd2 = Command::cargo_bin("dei")?;
    cmd2.arg("check")
        .arg(path.join("god_class.rs"))
        .arg("--max-lines")
        .arg("10000")
        .arg("--max-methods")
        .arg("1000")
        .arg("--max-complexity")
        .arg("1000");
    
    cmd2.assert()
        .success(); // Should pass with lenient thresholds
    
    Ok(())
}

#[test]
fn test_cli_check_verbose_flag() -> Result<()> {
    let fixture = FixtureManager::new()?;
    let path = fixture.copy_fixture("rust")?;
    
    let mut cmd = Command::cargo_bin("dei")?;
    cmd.arg("check")
        .arg(path.join("god_class.rs"))
        .arg("--max-lines")
        .arg("100")
        .arg("--verbose");
    
    cmd.assert()
        .failure()
        .stdout(predicate::str::contains("SUMMARY"));
    
    Ok(())
}

#[test]
fn test_cli_check_nonexistent_path() -> Result<()> {
    let mut cmd = Command::cargo_bin("dei")?;
    cmd.arg("check")
        .arg("/nonexistent/path/that/does/not/exist");
    
    cmd.assert()
        .failure();
    
    Ok(())
}

#[test]
fn test_cli_check_directory() -> Result<()> {
    let fixture = FixtureManager::new()?;
    let path = fixture.copy_fixture("rust")?;
    
    // Check entire directory
    let mut cmd = Command::cargo_bin("dei")?;
    cmd.arg("check")
        .arg(&path);
    
    let output = cmd.assert().success();
    
    // Should analyze multiple files
    let stdout = String::from_utf8(output.get_output().stdout.clone())?;
    assert!(stdout.contains("Analyzing"), "Should show analyzing message");
    
    Ok(())
}

#[test]
fn test_cli_arch_command() -> Result<()> {
    let fixture = FixtureManager::new()?;
    let path = fixture.copy_fixture("rust")?;
    
    let mut cmd = Command::cargo_bin("dei")?;
    cmd.arg("arch")
        .arg(&path);
    
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("DEI"));
    
    Ok(())
}

#[test]
fn test_cli_help_message() -> Result<()> {
    let mut cmd = Command::cargo_bin("dei")?;
    cmd.arg("--help");
    
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("check"))
        .stdout(predicate::str::contains("arch"));
    
    Ok(())
}

#[test]
fn test_cli_version() -> Result<()> {
    let mut cmd = Command::cargo_bin("dei")?;
    cmd.arg("--version");
    
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("dei"));
    
    Ok(())
}

#[test]
fn test_cli_check_csharp_files() -> Result<()> {
    let fixture = FixtureManager::new()?;
    let path = fixture.copy_fixture("csharp")?;
    
    let mut cmd = Command::cargo_bin("dei")?;
    cmd.arg("check")
        .arg(path.join("Healthy.cs"));
    
    cmd.assert()
        .success();
    
    Ok(())
}

#[test]
fn test_cli_mixed_language_directory() -> Result<()> {
    let fixture = FixtureManager::new()?;
    
    // Create mixed directory
    fixture.create_file("mixed/file1.rs", include_str!("../fixtures/rust/healthy.rs"))?;
    fixture.create_file("mixed/file2.cs", include_str!("../fixtures/csharp/Healthy.cs"))?;
    
    let mut cmd = Command::cargo_bin("dei")?;
    cmd.arg("check")
        .arg(fixture.path().join("mixed"));
    
    cmd.assert()
        .success();
    
    Ok(())
}

#[test]
fn test_cli_respects_gitignore() -> Result<()> {
    let fixture = FixtureManager::new()?;
    
    // Create files
    fixture.create_file("project/src/main.rs", include_str!("../fixtures/rust/healthy.rs"))?;
    fixture.create_file("project/target/debug/build.rs", include_str!("../fixtures/rust/god_class.rs"))?;
    
    // Create .gitignore
    fixture.create_file("project/.gitignore", "target/")?;
    
    let mut cmd = Command::cargo_bin("dei")?;
    cmd.arg("check")
        .arg(fixture.path().join("project"))
        .arg("--max-lines")
        .arg("100");
    
    // Should not fail because target/ should be ignored
    cmd.assert()
        .success();
    
    Ok(())
}

#[test]
fn test_cli_progress_indicators() -> Result<()> {
    let fixture = FixtureManager::new()?;
    let path = fixture.copy_fixture("rust")?;
    
    let mut cmd = Command::cargo_bin("dei")?;
    cmd.arg("check")
        .arg(&path);
    
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("✓")); // Should show progress checkmarks
    
    Ok(())
}

#[test]
fn test_cli_handles_empty_directory() -> Result<()> {
    let fixture = FixtureManager::new()?;
    std::fs::create_dir_all(fixture.path().join("empty"))?;
    
    let mut cmd = Command::cargo_bin("dei")?;
    cmd.arg("check")
        .arg(fixture.path().join("empty"));
    
    cmd.assert()
        .success(); // Should not crash on empty directory
    
    Ok(())
}

#[test]
fn test_cli_output_coloring() -> Result<()> {
    let fixture = FixtureManager::new()?;
    let path = fixture.copy_fixture("rust")?;
    
    let mut cmd = Command::cargo_bin("dei")?;
    cmd.arg("check")
        .arg(path.join("god_class.rs"))
        .arg("--max-lines")
        .arg("100");
    
    let output = cmd.assert().failure();
    
    // Should contain color formatting indicators
    let stdout = String::from_utf8(output.get_output().stdout.clone())?;
    assert!(!stdout.is_empty(), "Should produce output");
    
    Ok(())
}

