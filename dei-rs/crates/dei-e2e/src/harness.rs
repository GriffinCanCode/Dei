//! Test harness for running full E2E analysis

use anyhow::Result;
use dei_ast::{AstBuilder, ParallelTraverser};
use dei_core::{
    models::AnalysisResult,
    thresholds::{Complexity, Lines, MethodCount, Thresholds},
};
use dei_languages::MultiLanguageParser;
use std::path::Path;

/// Simplified test harness for running analysis
pub struct TestHarness {
    thresholds: Thresholds,
}

impl TestHarness {
    pub fn new() -> Result<Self> {
        Ok(Self {
            thresholds: Thresholds::default(),
        })
    }

    pub fn with_thresholds(mut self, thresholds: Thresholds) -> Self {
        self.thresholds = thresholds;
        self
    }

    /// Run full analysis on a directory
    pub fn analyze_path(&self, path: impl AsRef<Path>) -> Result<Vec<AnalysisResult>> {
        let builder = AstBuilder::new();
        let root_id = builder.build(path.as_ref())?;
        
        let parser = MultiLanguageParser::new()?;
        let traverser = ParallelTraverser::new(parser, builder.arena().clone());
        traverser.traverse_and_analyze(root_id, &self.thresholds)?;
        
        Ok(traverser.all_results())
    }

    /// Quick check if path has any god classes
    pub fn has_god_classes(&self, path: impl AsRef<Path>) -> Result<bool> {
        let results = self.analyze_path(path)?;
        Ok(results.iter().any(|r| r.is_god_class))
    }

    /// Quick check if path has any god methods
    pub fn has_god_methods(&self, path: impl AsRef<Path>) -> Result<bool> {
        let results = self.analyze_path(path)?;
        Ok(results.iter().any(|r| !r.god_methods.is_empty()))
    }

    /// Get count of issues found
    pub fn issue_count(&self, path: impl AsRef<Path>) -> Result<usize> {
        let results = self.analyze_path(path)?;
        Ok(results.iter().filter(|r| r.has_issues()).count())
    }
}

impl Default for TestHarness {
    fn default() -> Self {
        Self::new().expect("Failed to create test harness")
    }
}

/// Builder for custom thresholds in tests
pub struct ThresholdBuilder {
    thresholds: Thresholds,
}

impl ThresholdBuilder {
    pub fn new() -> Self {
        Self {
            thresholds: Thresholds::default(),
        }
    }

    pub fn max_class_lines(mut self, lines: usize) -> Self {
        self.thresholds.max_class_lines = Lines(lines);
        self
    }

    pub fn max_methods(mut self, count: usize) -> Self {
        self.thresholds.max_methods = MethodCount(count);
        self
    }

    pub fn max_class_complexity(mut self, complexity: usize) -> Self {
        self.thresholds.max_class_complexity = Complexity(complexity);
        self
    }

    pub fn max_method_lines(mut self, lines: usize) -> Self {
        self.thresholds.max_method_lines = Lines(lines);
        self
    }

    pub fn max_method_complexity(mut self, complexity: usize) -> Self {
        self.thresholds.max_method_complexity = Complexity(complexity);
        self
    }

    pub fn build(self) -> Thresholds {
        self.thresholds
    }
}

impl Default for ThresholdBuilder {
    fn default() -> Self {
        Self::new()
    }
}

