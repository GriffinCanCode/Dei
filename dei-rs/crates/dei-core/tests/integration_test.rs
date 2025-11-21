use dei_core::{
    metrics::*, 
    models::*, 
    thresholds::*,
    Error, Result,
};
use std::sync::Arc;

#[test]
fn test_thresholds_creation() {
    let thresholds = Thresholds::default();
    assert!(thresholds.validate().is_ok());
    
    let custom = Thresholds {
        max_class_lines: Lines(300),
        max_method_lines: Lines(40),
        max_class_complexity: Complexity(50),
        max_method_complexity: Complexity(8),
        max_methods: MethodCount(15),
        max_parameters: ParamCount(4),
        max_classes_per_file: 3,
        max_file_lines: Lines(500),
        min_cluster_size: 3,
        cluster_threshold: 0.7,
    };
    assert!(custom.validate().is_ok());
}

#[test]
fn test_invalid_thresholds() {
    let invalid = Thresholds {
        max_class_lines: Lines(10),
        max_method_lines: Lines(100), // Invalid: method lines > class lines
        max_class_complexity: Complexity(50),
        max_method_complexity: Complexity(10),
        max_methods: MethodCount(20),
        max_parameters: ParamCount(5),
        max_classes_per_file: 3,
        max_file_lines: Lines(500),
        min_cluster_size: 3,
        cluster_threshold: 0.7,
    };
    assert!(invalid.validate().is_err(), "Should fail validation when method lines > class lines");
}

#[test]
fn test_method_metrics_god_detection() {
    let god_method = MethodMetrics {
        name: "do_everything".into(),
        lines: Lines(150),
        complexity: Complexity(25),
        parameters: ParamCount(8),
        called_methods: Arc::new([]),
        accessed_fields: Arc::new([]),
        return_type: "Result<(), Error>".into(),
        is_public: true,
        is_static: false,
        is_async: false,
        tokens: Arc::new([]),
    };
    
    let thresholds = Thresholds::default();
    assert!(god_method.is_god_method(&thresholds), "Expected method to be flagged as god method");
}

#[test]
fn test_class_metrics_god_detection() {
    let god_class = ClassMetrics {
        name: "MegaController".into(),
        fully_qualified_name: "api::controllers::MegaController".into(),
        file_path: "/src/controllers/mega.rs".into(),
        lines: Lines(800),
        method_count: MethodCount(45),
        property_count: 20,
        field_count: 30,
        complexity: Complexity(120),
        methods: Arc::new([]),
        dependencies: Arc::new([]),
    };
    
    let thresholds = Thresholds::default();
    assert!(god_class.is_god_class(&thresholds), "Expected class to be flagged as god class");
}

#[test]
fn test_normal_class_metrics() {
    let normal_class = ClassMetrics {
        name: "User".into(),
        fully_qualified_name: "models::User".into(),
        file_path: "/src/models/user.rs".into(),
        lines: Lines(80),
        method_count: MethodCount(8),
        property_count: 5,
        field_count: 5,
        complexity: Complexity(15),
        methods: Arc::new([]),
        dependencies: Arc::new([]),
    };
    
    let thresholds = Thresholds::default();
    assert!(!normal_class.is_god_class(&thresholds), "Normal class should not be flagged as god class");
}

#[test]
fn test_language_detection() {
    assert_eq!(Language::from_extension("rs"), Some(Language::Rust));
    assert_eq!(Language::from_extension("cs"), Some(Language::CSharp));
    assert_eq!(Language::from_extension("txt"), None);
}

#[test]
fn test_metrics_display() {
    let lines = Lines(100);
    let complexity = Complexity(10);
    let methods = MethodCount(5);
    let params = ParamCount(3);
    
    assert_eq!(lines.0, 100);
    assert_eq!(complexity.0, 10);
    assert_eq!(methods.0, 5);
    assert_eq!(params.0, 3);
}

