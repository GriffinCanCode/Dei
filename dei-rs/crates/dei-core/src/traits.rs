//! Core traits for extensibility

use crate::{
    error::Result,
    metrics::*,
    models::*,
    thresholds::{Complexity, Lines, Thresholds},
};
use std::path::Path;

/// Trait for parsing source files into metrics
pub trait Parser: Send + Sync {
    /// Parse a single file
    fn parse_file(&self, path: &Path) -> Result<FileMetrics>;
    
    /// Get supported languages
    fn supported_languages(&self) -> &[Language];
}

/// Trait for calculating code complexity
pub trait ComplexityCalculator: Send + Sync {
    /// Calculate cyclomatic complexity
    fn calculate_complexity(&self, source: &str) -> Complexity;
    
    /// Count lines of code (excluding comments/whitespace)
    fn count_lines(&self, source: &str) -> Lines;
}

/// Trait for semantic analysis and clustering
pub trait ClusterAnalyzer: Send + Sync {
    /// Analyze methods and cluster by responsibility
    fn analyze(
        &self,
        class: &ClassMetrics,
        thresholds: &Thresholds,
    ) -> Result<Vec<ResponsibilityCluster>>;
}

/// Trait for generating reports
pub trait Reporter: Send + Sync {
    /// Generate a report from analysis results
    fn generate_report(&self, results: &[AnalysisResult]) -> Result<String>;
}

