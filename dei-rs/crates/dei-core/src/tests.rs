#[cfg(test)]
mod tests {
    use crate::{metrics::*, thresholds::*};
    use std::sync::Arc;

    #[test]
    fn test_threshold_validation() {
        let valid = Thresholds::default();
        assert!(valid.validate().is_ok());

        let invalid = Thresholds {
            max_class_lines: Lines(10),
            max_method_lines: Lines(100),
            ..Default::default()
        };
        assert!(invalid.validate().is_err());
    }

    #[test]
    fn test_god_method_detection() {
        let method = MethodMetrics {
            name: "huge_method".into(),
            lines: Lines(100),
            complexity: Complexity(15),
            parameters: ParamCount(7),
            called_methods: Arc::new([]),
            accessed_fields: Arc::new([]),
            return_type: "void".into(),
            is_public: true,
            is_static: false,
            is_async: false,
            tokens: Arc::new([]),
        };

        let thresholds = Thresholds::default();
        assert!(method.is_god_method(&thresholds));
    }

    #[test]
    fn test_god_class_detection() {
        let class = ClassMetrics {
            name: "GodClass".into(),
            fully_qualified_name: "com.example.GodClass".into(),
            file_path: "/test.rs".into(),
            lines: Lines(500),
            method_count: MethodCount(30),
            property_count: 10,
            field_count: 15,
            complexity: Complexity(80),
            methods: Arc::new([]),
            dependencies: Arc::new([]),
        };

        let thresholds = Thresholds::default();
        assert!(class.is_god_class(&thresholds));
    }

    #[test]
    fn test_violation_score() {
        let method = MethodMetrics {
            name: "test".into(),
            lines: Lines(100),
            complexity: Complexity(20),
            parameters: ParamCount(10),
            called_methods: Arc::new([]),
            accessed_fields: Arc::new([]),
            return_type: "void".into(),
            is_public: true,
            is_static: false,
            is_async: false,
            tokens: Arc::new([]),
        };

        let thresholds = Thresholds::default();
        let score = method.violation_score(&thresholds);
        assert!(score > 1.0); // Exceeds all thresholds
    }
}

