//! Detection thresholds with strong typing and validation

use serde::{Deserialize, Serialize};

/// Newtype for lines of code to prevent mixing with other integers
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct Lines(pub usize);

/// Newtype for cyclomatic complexity
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct Complexity(pub usize);

/// Newtype for method count
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct MethodCount(pub usize);

/// Newtype for parameter count
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct ParamCount(pub usize);

/// Configurable detection thresholds with strong typing
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Thresholds {
    // Class-level
    pub max_class_lines: Lines,
    pub max_methods: MethodCount,
    pub max_class_complexity: Complexity,
    
    // Method-level
    pub max_method_lines: Lines,
    pub max_method_complexity: Complexity,
    pub max_parameters: ParamCount,
    
    // File-level
    pub max_classes_per_file: usize,
    pub max_file_lines: Lines,
    
    // Clustering
    pub min_cluster_size: usize,
    pub cluster_threshold: f64,
}

impl Default for Thresholds {
    fn default() -> Self {
        Self {
            max_class_lines: Lines(300),
            max_methods: MethodCount(20),
            max_class_complexity: Complexity(50),
            max_method_lines: Lines(50),
            max_method_complexity: Complexity(10),
            max_parameters: ParamCount(5),
            max_classes_per_file: 3,
            max_file_lines: Lines(500),
            min_cluster_size: 3,
            cluster_threshold: 0.7,
        }
    }
}

impl Thresholds {
    /// Validate thresholds are sensible
    pub fn validate(&self) -> Result<(), String> {
        if self.max_class_lines.0 < self.max_method_lines.0 {
            return Err("max_class_lines must be >= max_method_lines".into());
        }
        if self.cluster_threshold < 0.0 || self.cluster_threshold > 1.0 {
            return Err("cluster_threshold must be between 0.0 and 1.0".into());
        }
        if self.min_cluster_size < 2 {
            return Err("min_cluster_size must be >= 2".into());
        }
        Ok(())
    }
}

