//! Code metrics with strong typing

use serde::{Deserialize, Serialize};
use std::sync::Arc;

use crate::thresholds::*;

/// Method-level metrics with zero-copy strings where possible
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MethodMetrics {
    pub name: Arc<str>,
    pub lines: Lines,
    pub complexity: Complexity,
    pub parameters: ParamCount,
    pub called_methods: Arc<[Arc<str>]>,
    pub accessed_fields: Arc<[Arc<str>]>,
    pub return_type: Arc<str>,
    pub is_public: bool,
    pub is_static: bool,
    pub is_async: bool,
    pub tokens: Arc<[Arc<str>]>, // For semantic analysis
}

impl MethodMetrics {
    /// Check if method exceeds any threshold
    pub fn is_god_method(&self, thresholds: &Thresholds) -> bool {
        self.lines > thresholds.max_method_lines
            || self.complexity > thresholds.max_method_complexity
            || self.parameters > thresholds.max_parameters
    }

    /// Calculate violation score (higher = worse)
    pub fn violation_score(&self, thresholds: &Thresholds) -> f64 {
        let line_ratio = self.lines.0 as f64 / thresholds.max_method_lines.0 as f64;
        let complexity_ratio = self.complexity.0 as f64 / thresholds.max_method_complexity.0 as f64;
        let param_ratio = self.parameters.0 as f64 / thresholds.max_parameters.0 as f64;
        
        (line_ratio + complexity_ratio + param_ratio) / 3.0
    }
}

/// Class-level metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClassMetrics {
    pub name: Arc<str>,
    pub fully_qualified_name: Arc<str>,
    pub file_path: Arc<str>,
    pub lines: Lines,
    pub method_count: MethodCount,
    pub property_count: usize,
    pub field_count: usize,
    pub complexity: Complexity,
    pub methods: Arc<[MethodMetrics]>,
    pub dependencies: Arc<[Arc<str>]>,
}

impl ClassMetrics {
    /// Check if class exceeds any threshold
    pub fn is_god_class(&self, thresholds: &Thresholds) -> bool {
        self.lines > thresholds.max_class_lines
            || self.method_count > thresholds.max_methods
            || self.complexity > thresholds.max_class_complexity
    }

    /// Count god methods in this class
    pub fn god_method_count(&self, thresholds: &Thresholds) -> usize {
        self.methods.iter().filter(|m| m.is_god_method(thresholds)).count()
    }
}

/// File-level metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileMetrics {
    pub path: Arc<str>,
    pub lines: Lines,
    pub classes: Arc<[ClassMetrics]>,
}

impl FileMetrics {
    /// Check if file has too many classes or is too long
    pub fn is_god_file(&self, thresholds: &Thresholds) -> bool {
        self.classes.len() > thresholds.max_classes_per_file
            || self.lines > thresholds.max_file_lines
    }
}

