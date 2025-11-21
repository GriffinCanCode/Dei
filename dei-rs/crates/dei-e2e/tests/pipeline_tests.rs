//! End-to-end pipeline tests
//!
//! These tests exercise the full analysis pipeline from filesystem
//! traversal through parsing, analysis, and result generation.

use anyhow::Result;
use dei_core::thresholds::{Lines, MethodCount};
use dei_e2e::{FixtureManager, TestHarness, ThresholdBuilder};

#[tokio::test]
async fn test_healthy_rust_code_passes() -> Result<()> {
    let fixture = FixtureManager::new()?;
    let path = fixture.copy_fixture("rust")?;
    
    let harness = TestHarness::new()?;
    let results = harness.analyze_path(path.join("healthy.rs"))?;
    
    // Should find the UserRepository and User classes
    assert!(results.len() >= 2, "Should find at least 2 structs");
    
    // None should be god classes
    let god_classes = results.iter().filter(|r| r.is_god_class).count();
    assert_eq!(god_classes, 0, "Healthy code should have no god classes");
    
    // None should have god methods
    let god_methods: usize = results.iter().map(|r| r.god_methods.len()).sum();
    assert_eq!(god_methods, 0, "Healthy code should have no god methods");
    
    Ok(())
}

#[tokio::test]
async fn test_god_class_detection_rust() -> Result<()> {
    let fixture = FixtureManager::new()?;
    let path = fixture.copy_fixture("rust")?;
    
    // Use strict thresholds to ensure god class is detected
    let thresholds = ThresholdBuilder::new()
        .max_class_lines(200)
        .max_methods(15)
        .max_class_complexity(40)
        .build();
    
    let harness = TestHarness::new()?.with_thresholds(thresholds);
    let results = harness.analyze_path(path.join("god_class.rs"))?;
    
    // Should detect the MegaUserManager as a god class
    let god_classes: Vec<_> = results.iter()
        .filter(|r| r.is_god_class)
        .collect();
    
    assert!(!god_classes.is_empty(), "Should detect at least one god class");
    
    // Verify the god class has expected characteristics
    let mega_manager = god_classes.iter()
        .find(|r| r.class_metrics.name.contains("MegaUserManager"));
    
    assert!(mega_manager.is_some(), "Should detect MegaUserManager as god class");
    
    if let Some(manager) = mega_manager {
        assert!(
            manager.class_metrics.lines > Lines(200),
            "MegaUserManager should exceed line threshold"
        );
        assert!(
            manager.class_metrics.method_count > MethodCount(15),
            "MegaUserManager should exceed method count threshold"
        );
    }
    
    Ok(())
}

#[tokio::test]
async fn test_god_method_detection_rust() -> Result<()> {
    let fixture = FixtureManager::new()?;
    let path = fixture.copy_fixture("rust")?;
    
    // Use strict thresholds for method detection
    let thresholds = ThresholdBuilder::new()
        .max_method_lines(30)
        .max_method_complexity(10)
        .build();
    
    let harness = TestHarness::new()?.with_thresholds(thresholds);
    let results = harness.analyze_path(path.join("god_method.rs"))?;
    
    // Should detect god methods in PaymentProcessor
    let classes_with_god_methods: Vec<_> = results.iter()
        .filter(|r| !r.god_methods.is_empty())
        .collect();
    
    assert!(!classes_with_god_methods.is_empty(), "Should detect god methods");
    
    // Verify the process_complex_payment method is flagged
    let payment_processor = classes_with_god_methods.iter()
        .find(|r| r.class_metrics.name.contains("PaymentProcessor"));
    
    assert!(payment_processor.is_some(), "Should find PaymentProcessor with god methods");
    
    if let Some(processor) = payment_processor {
        let complex_payment_method = processor.god_methods.iter()
            .find(|m| m.method_name.contains("process_complex_payment"));
        
        assert!(
            complex_payment_method.is_some(),
            "Should detect process_complex_payment as god method"
        );
    }
    
    Ok(())
}

#[tokio::test]
async fn test_healthy_csharp_code_passes() -> Result<()> {
    let fixture = FixtureManager::new()?;
    let path = fixture.copy_fixture("csharp")?;
    
    let harness = TestHarness::new()?;
    let results = harness.analyze_path(path.join("Healthy.cs"))?;
    
    // Should find ProductRepository and Product classes
    assert!(results.len() >= 2, "Should find at least 2 classes");
    
    // None should be god classes
    let god_classes = results.iter().filter(|r| r.is_god_class).count();
    assert_eq!(god_classes, 0, "Healthy C# code should have no god classes");
    
    Ok(())
}

#[tokio::test]
async fn test_god_class_detection_csharp() -> Result<()> {
    let fixture = FixtureManager::new()?;
    let path = fixture.copy_fixture("csharp")?;
    
    // Strict thresholds
    let thresholds = ThresholdBuilder::new()
        .max_class_lines(200)
        .max_methods(20)
        .max_class_complexity(50)
        .build();
    
    let harness = TestHarness::new()?.with_thresholds(thresholds);
    let results = harness.analyze_path(path.join("GodClass.cs"))?;
    
    // Should detect the MegaOrderManager as a god class
    let god_classes: Vec<_> = results.iter()
        .filter(|r| r.is_god_class)
        .collect();
    
    assert!(!god_classes.is_empty(), "Should detect god class in C#");
    
    let mega_order_manager = god_classes.iter()
        .find(|r| r.class_metrics.name.contains("MegaOrderManager"));
    
    assert!(mega_order_manager.is_some(), "Should detect MegaOrderManager");
    
    Ok(())
}

#[tokio::test]
async fn test_multi_language_directory() -> Result<()> {
    let fixture = FixtureManager::new()?;
    
    // Create a mixed directory with both Rust and C# files
    fixture.create_file("mixed/main.rs", include_str!("../fixtures/rust/healthy.rs"))?;
    fixture.create_file("mixed/App.cs", include_str!("../fixtures/csharp/Healthy.cs"))?;
    
    let harness = TestHarness::new()?;
    let results = harness.analyze_path(fixture.path().join("mixed"))?;
    
    // Should analyze both languages
    assert!(results.len() >= 4, "Should find classes from both languages");
    
    Ok(())
}

#[tokio::test]
async fn test_nested_directory_traversal() -> Result<()> {
    let fixture = FixtureManager::new()?;
    
    // Create nested structure
    fixture.create_file("project/src/models/user.rs", include_str!("../fixtures/rust/healthy.rs"))?;
    fixture.create_file("project/src/services/payment.rs", include_str!("../fixtures/rust/god_method.rs"))?;
    fixture.create_file("project/tests/integration.rs", "pub fn test() {}")?;
    
    let harness = TestHarness::new()?;
    let results = harness.analyze_path(fixture.path().join("project"))?;
    
    // Should traverse all nested directories
    assert!(!results.is_empty(), "Should find classes in nested directories");
    
    Ok(())
}

#[tokio::test]
async fn test_threshold_boundaries() -> Result<()> {
    let fixture = FixtureManager::new()?;
    
    // Create a class right at the threshold
    let code = r#"
pub struct BoundaryClass {
    field1: u32,
    field2: String,
}

impl BoundaryClass {
    pub fn method1(&self) -> u32 { self.field1 }
    pub fn method2(&self) -> &str { &self.field2 }
    pub fn method3(&mut self, x: u32) { self.field1 = x; }
    pub fn method4(&mut self, s: String) { self.field2 = s; }
    pub fn method5(&self) -> bool { self.field1 > 0 }
}
"#;
    
    fixture.create_file("boundary.rs", code)?;
    
    // Test with threshold exactly at the boundary
    let thresholds = ThresholdBuilder::new()
        .max_methods(5)
        .build();
    
    let harness = TestHarness::new()?.with_thresholds(thresholds);
    let results = harness.analyze_path(fixture.path().join("boundary.rs"))?;
    
    // Should NOT be flagged (at boundary, not over)
    let god_classes = results.iter().filter(|r| r.is_god_class).count();
    assert_eq!(god_classes, 0, "Class at threshold should not be flagged");
    
    // Now test with threshold just below
    let strict_thresholds = ThresholdBuilder::new()
        .max_methods(4)
        .build();
    
    let strict_harness = TestHarness::new()?.with_thresholds(strict_thresholds);
    let strict_results = strict_harness.analyze_path(fixture.path().join("boundary.rs"))?;
    
    // Should now be flagged
    let strict_god_classes = strict_results.iter().filter(|r| r.is_god_class).count();
    assert!(strict_god_classes > 0, "Class over threshold should be flagged");
    
    Ok(())
}

#[tokio::test]
async fn test_empty_directory() -> Result<()> {
    let fixture = FixtureManager::new()?;
    
    std::fs::create_dir_all(fixture.path().join("empty"))?;
    
    let harness = TestHarness::new()?;
    let results = harness.analyze_path(fixture.path().join("empty"))?;
    
    assert_eq!(results.len(), 0, "Empty directory should produce no results");
    
    Ok(())
}

#[tokio::test]
async fn test_complexity_scoring() -> Result<()> {
    let fixture = FixtureManager::new()?;
    
    // Create a method with high complexity
    let code = r#"
pub struct ComplexClass;

impl ComplexClass {
    pub fn complex_method(&self, x: i32) -> i32 {
        if x > 0 {
            if x > 10 {
                if x > 100 {
                    return 1;
                } else if x > 50 {
                    return 2;
                } else {
                    return 3;
                }
            } else if x > 5 {
                return 4;
            } else {
                return 5;
            }
        } else if x < 0 {
            if x < -10 {
                return 6;
            } else {
                return 7;
            }
        } else {
            return 0;
        }
    }
}
"#;
    
    fixture.create_file("complex.rs", code)?;
    
    let thresholds = ThresholdBuilder::new()
        .max_method_complexity(5)
        .build();
    
    let harness = TestHarness::new()?.with_thresholds(thresholds);
    let results = harness.analyze_path(fixture.path().join("complex.rs"))?;
    
    // Should detect high complexity
    let has_god_methods = results.iter().any(|r| !r.god_methods.is_empty());
    assert!(has_god_methods, "Should detect high complexity method");
    
    Ok(())
}

#[tokio::test]
async fn test_issue_count_helper() -> Result<()> {
    let fixture = FixtureManager::new()?;
    let path = fixture.copy_fixture("rust")?;
    
    let thresholds = ThresholdBuilder::new()
        .max_class_lines(100)
        .max_methods(10)
        .build();
    
    let harness = TestHarness::new()?.with_thresholds(thresholds);
    
    // Test healthy code
    let healthy_count = harness.issue_count(path.join("healthy.rs"))?;
    assert_eq!(healthy_count, 0, "Healthy code should have 0 issues");
    
    // Test god class
    let god_class_count = harness.issue_count(path.join("god_class.rs"))?;
    assert!(god_class_count > 0, "God class should have issues");
    
    Ok(())
}

#[tokio::test]
async fn test_has_god_classes_helper() -> Result<()> {
    let fixture = FixtureManager::new()?;
    let path = fixture.copy_fixture("rust")?;
    
    let harness = TestHarness::new()?;
    
    // Healthy code should not have god classes
    assert!(!harness.has_god_classes(path.join("healthy.rs"))?);
    
    // God class fixture should have god classes (with strict thresholds)
    let strict_thresholds = ThresholdBuilder::new()
        .max_class_lines(100)
        .build();
    let strict_harness = TestHarness::new()?.with_thresholds(strict_thresholds);
    assert!(strict_harness.has_god_classes(path.join("god_class.rs"))?);
    
    Ok(())
}

