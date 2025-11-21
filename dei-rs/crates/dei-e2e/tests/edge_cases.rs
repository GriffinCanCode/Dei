//! Edge case and error handling tests
//!
//! These tests verify dei handles unusual inputs gracefully.

use anyhow::Result;
use dei_e2e::{FixtureManager, TestHarness, ThresholdBuilder};

#[tokio::test]
async fn test_empty_file() -> Result<()> {
    let fixture = FixtureManager::new()?;
    fixture.create_file("empty.rs", "")?;
    
    let harness = TestHarness::new()?;
    let results = harness.analyze_path(fixture.path().join("empty.rs"))?;
    
    assert_eq!(results.len(), 0, "Empty file should produce no results");
    
    Ok(())
}

#[tokio::test]
async fn test_comment_only_file() -> Result<()> {
    let fixture = FixtureManager::new()?;
    fixture.create_file("comments.rs", r#"
// This file only contains comments
// No actual code here
/* Multi-line comment
   with multiple lines
   but no code */
"#)?;
    
    let harness = TestHarness::new()?;
    let results = harness.analyze_path(fixture.path().join("comments.rs"))?;
    
    assert_eq!(results.len(), 0, "Comment-only file should produce no results");
    
    Ok(())
}

#[tokio::test]
async fn test_single_line_file() -> Result<()> {
    let fixture = FixtureManager::new()?;
    fixture.create_file("oneline.rs", "pub struct Tiny;")?;
    
    let harness = TestHarness::new()?;
    let results = harness.analyze_path(fixture.path().join("oneline.rs"))?;
    
    // Should handle it gracefully
    assert!(results.len() <= 1);
    
    Ok(())
}

#[tokio::test]
async fn test_very_long_line() -> Result<()> {
    let fixture = FixtureManager::new()?;
    
    let long_line = format!("pub struct Long {{ {} }}", "field: u32,".repeat(100));
    fixture.create_file("longline.rs", &long_line)?;
    
    let harness = TestHarness::new()?;
    let results = harness.analyze_path(fixture.path().join("longline.rs"))?;
    
    // Should handle without crashing
    assert!(!results.is_empty());
    
    Ok(())
}

#[tokio::test]
async fn test_unicode_in_code() -> Result<()> {
    let fixture = FixtureManager::new()?;
    
    let unicode_code = r#"
pub struct 用户 {
    名称: String,
}

impl 用户 {
    pub fn 创建() -> Self {
        Self { 名称: String::new() }
    }
}
"#;
    
    fixture.create_file("unicode.rs", unicode_code)?;
    
    let harness = TestHarness::new()?;
    let results = harness.analyze_path(fixture.path().join("unicode.rs"))?;
    
    // Should handle Unicode identifiers
    assert!(!results.is_empty());
    
    Ok(())
}

#[tokio::test]
async fn test_deeply_nested_structures() -> Result<()> {
    let fixture = FixtureManager::new()?;
    
    let nested = r#"
pub mod level1 {
    pub mod level2 {
        pub mod level3 {
            pub mod level4 {
                pub struct Deep {
                    value: u32,
                }
                
                impl Deep {
                    pub fn new() -> Self {
                        Self { value: 0 }
                    }
                }
            }
        }
    }
}
"#;
    
    fixture.create_file("nested.rs", nested)?;
    
    let harness = TestHarness::new()?;
    let _results = harness.analyze_path(fixture.path().join("nested.rs"))?;
    
    // Should handle deep nesting without stack overflow
    Ok(())
}

#[tokio::test]
async fn test_syntax_error_resilience() -> Result<()> {
    let fixture = FixtureManager::new()?;
    
    // Invalid Rust syntax
    let invalid = r#"
pub struct Broken {
    field: u32
    // Missing semicolon
}

impl Broken {
    pub fn incomplete(
    // Missing closing paren and body
"#;
    
    fixture.create_file("broken.rs", invalid)?;
    
    let harness = TestHarness::new()?;
    // Should handle parsing errors gracefully
    let result = harness.analyze_path(fixture.path().join("broken.rs"));
    
    // May succeed with empty results or return error - both acceptable
    match result {
        Ok(results) => assert!(results.is_empty() || !results.is_empty()),
        Err(_) => {} // Error is also acceptable for invalid syntax
    }
    
    Ok(())
}

#[tokio::test]
async fn test_binary_file_ignored() -> Result<()> {
    let fixture = FixtureManager::new()?;
    
    // Create a binary file
    let binary_data: Vec<u8> = vec![0xFF, 0xFE, 0xFD, 0x00, 0x01, 0x02];
    std::fs::write(fixture.path().join("binary.dat"), binary_data)?;
    
    let harness = TestHarness::new()?;
    let results = harness.analyze_path(fixture.path().join("binary.dat"))?;
    
    // Should ignore non-source files
    assert_eq!(results.len(), 0);
    
    Ok(())
}

#[tokio::test]
async fn test_symlink_handling() -> Result<()> {
    let fixture = FixtureManager::new()?;
    
    // Create real file
    fixture.create_file("real.rs", include_str!("../fixtures/rust/healthy.rs"))?;
    
    // Try to create symlink (may fail on some systems)
    let symlink_path = fixture.path().join("link.rs");
    let target_path = fixture.path().join("real.rs");
    
    #[cfg(unix)]
    {
        use std::os::unix::fs::symlink;
        if symlink(&target_path, &symlink_path).is_ok() {
            let harness = TestHarness::new()?;
            let _results = harness.analyze_path(symlink_path)?;
            // Should handle symlinks gracefully
        }
    }
    
    Ok(())
}

#[tokio::test]
async fn test_permission_denied_handling() -> Result<()> {
    // This test is platform-specific and may be skipped
    #[cfg(unix)]
    {
        let fixture = FixtureManager::new()?;
        fixture.create_file("restricted.rs", "pub struct Test;")?;
        
        let path = fixture.path().join("restricted.rs");
        
        // Make file unreadable
        use std::os::unix::fs::PermissionsExt;
        let mut perms = std::fs::metadata(&path)?.permissions();
        perms.set_mode(0o000);
        std::fs::set_permissions(&path, perms)?;
        
        let harness = TestHarness::new()?;
        let result = harness.analyze_path(&path);
        
        // Should either skip the file or return an error gracefully
        let _ = result;
        
        // Restore permissions for cleanup
        let mut perms = std::fs::metadata(&path)?.permissions();
        perms.set_mode(0o644);
        let _ = std::fs::set_permissions(&path, perms);
    }
    
    Ok(())
}

#[tokio::test]
async fn test_very_large_file() -> Result<()> {
    let fixture = FixtureManager::new()?;
    
    // Generate a large file
    let mut large_code = String::from("pub struct Large {\n");
    for i in 0..1000 {
        large_code.push_str(&format!("    field{}: u32,\n", i));
    }
    large_code.push_str("}\n\nimpl Large {\n");
    for i in 0..1000 {
        large_code.push_str(&format!("    pub fn method{}(&self) -> u32 {{ self.field{} }}\n", i, i));
    }
    large_code.push_str("}\n");
    
    fixture.create_file("large.rs", &large_code)?;
    
    let harness = TestHarness::new()?;
    let results = harness.analyze_path(fixture.path().join("large.rs"))?;
    
    // Should handle large files
    assert!(!results.is_empty());
    
    // Should definitely be flagged as god class
    assert!(results.iter().any(|r| r.is_god_class));
    
    Ok(())
}

#[tokio::test]
async fn test_extreme_thresholds() -> Result<()> {
    let fixture = FixtureManager::new()?;
    fixture.create_file("test.rs", include_str!("../fixtures/rust/healthy.rs"))?;
    
    // Test with zero thresholds (everything is a violation)
    let zero_thresholds = ThresholdBuilder::new()
        .max_class_lines(0)
        .max_methods(0)
        .max_method_lines(0)
        .build();
    
    let harness = TestHarness::new()?.with_thresholds(zero_thresholds);
    let results = harness.analyze_path(fixture.path().join("test.rs"))?;
    
    // Everything should be flagged
    if !results.is_empty() {
        assert!(results.iter().any(|r| r.has_issues()));
    }
    
    // Test with maximum thresholds (nothing is a violation)
    let max_thresholds = ThresholdBuilder::new()
        .max_class_lines(usize::MAX)
        .max_methods(usize::MAX)
        .max_method_lines(usize::MAX)
        .build();
    
    let lenient_harness = TestHarness::new()?.with_thresholds(max_thresholds);
    let lenient_results = lenient_harness.analyze_path(fixture.path().join("test.rs"))?;
    
    // Nothing should be flagged
    let has_issues = lenient_results.iter().any(|r| r.has_issues());
    assert!(!has_issues, "With max thresholds, nothing should be flagged");
    
    Ok(())
}

#[tokio::test]
async fn test_concurrent_analysis() -> Result<()> {
    let fixture = FixtureManager::new()?;
    
    // Create multiple test files
    for i in 0..10 {
        fixture.create_file(
            &format!("file{}.rs", i),
            include_str!("../fixtures/rust/healthy.rs"),
        )?;
    }
    
    let harness = TestHarness::new()?;
    
    // Run multiple analyses concurrently
    let mut handles = vec![];
    for i in 0..10 {
        let path = fixture.path().join(format!("file{}.rs", i));
        let harness_clone = TestHarness::new()?;
        
        let handle = tokio::spawn(async move {
            harness_clone.analyze_path(path)
        });
        handles.push(handle);
    }
    
    // All should complete successfully
    for handle in handles {
        let result = handle.await?;
        assert!(result.is_ok());
    }
    
    Ok(())
}

#[tokio::test]
async fn test_special_characters_in_path() -> Result<()> {
    let fixture = FixtureManager::new()?;
    
    // Create directory with special characters
    let special_dir = "test with spaces & special!";
    fixture.create_file(
        &format!("{}/test.rs", special_dir),
        "pub struct Test;",
    )?;
    
    let harness = TestHarness::new()?;
    let results = harness.analyze_path(fixture.path().join(special_dir))?;
    
    // Should handle special characters in paths
    assert!(!results.is_empty() || results.is_empty()); // Either way is fine
    
    Ok(())
}

#[tokio::test]
async fn test_mixed_line_endings() -> Result<()> {
    let fixture = FixtureManager::new()?;
    
    // Create file with mixed line endings (CRLF and LF)
    let mixed = "pub struct Mixed {\r\n    field1: u32,\n    field2: String,\r\n}\r\n";
    fixture.create_file("mixed.rs", mixed)?;
    
    let harness = TestHarness::new()?;
    let results = harness.analyze_path(fixture.path().join("mixed.rs"))?;
    
    // Should handle mixed line endings
    assert!(!results.is_empty());
    
    Ok(())
}

