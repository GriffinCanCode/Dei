//! Parallel AST traverser using Rayon's work-stealing scheduler
//! 
//! Significantly more efficient than C#'s Parallel.ForEach due to:
//! - Work stealing (better load balancing)
//! - Cache-friendly memory layout (arena allocation)
//! - Lock-free data structures (DashMap)

use dashmap::DashMap;
use dei_core::{
    error::Result,
    metrics::*,
    models::*,
    thresholds::Thresholds,
    traits::Parser,
    Error,
};
use rayon::prelude::*;
use std::sync::Arc;

use crate::{arena::SharedArena, node::{Node, NodeId}};

/// Parallel AST traverser with intelligent work distribution
pub struct ParallelTraverser<P>
where
    P: Parser,
{
    parser: Arc<P>,
    arena: SharedArena,
    results: Arc<DashMap<NodeId, Vec<AnalysisResult>>>,
}

impl<P> ParallelTraverser<P>
where
    P: Parser,
{
    pub fn new(parser: P, arena: SharedArena) -> Self {
        Self {
            parser: Arc::new(parser),
            arena,
            results: Arc::new(DashMap::new()),
        }
    }

    /// Traverse and analyze AST in parallel using Rayon
    pub fn traverse_and_analyze(
        &self,
        root_id: NodeId,
        thresholds: &Thresholds,
    ) -> Result<()> {
        self.traverse_node(root_id, thresholds)
    }

    fn traverse_node(&self, node_id: NodeId, thresholds: &Thresholds) -> Result<()> {
        let node = self.arena.get(node_id).ok_or_else(|| {
            Error::Analysis(format!("Node {:?} not found", node_id))
        })?;

        if node.is_file() {
            self.analyze_file_node(&node, thresholds)?;
        } else if node.is_directory() {
            self.traverse_directory(&node, thresholds)?;
        }

        Ok(())
    }

    fn traverse_directory(&self, node: &Node, thresholds: &Thresholds) -> Result<()> {
        // Rayon's par_iter uses work-stealing for optimal load balancing
        node.children
            .par_iter()
            .try_for_each(|&child_id| self.traverse_node(child_id, thresholds))?;

        Ok(())
    }

    fn analyze_file_node(&self, node: &Node, thresholds: &Thresholds) -> Result<()> {
        // Only analyze files in supported languages
        if node.language().is_none() {
            return Ok(());
        }

        // Parse file to get metrics
        let path = std::path::Path::new(node.path.as_ref());
        let file_metrics = self.parser.parse_file(path)?;

        // Update node with file metrics
        if let Some(mut updated_node) = self.arena.get(node.id) {
            updated_node = updated_node.with_file_metrics(file_metrics.clone());

            // Check for god file
            if file_metrics.is_god_file(thresholds) {
                let god_file = self.create_god_file_result(&file_metrics, thresholds);
                updated_node = updated_node.with_god_file_result(god_file);
            }

            // Analyze each class
            let mut analysis_results = Vec::new();
            for class in file_metrics.classes.iter() {
                let result = self.analyze_class(class, thresholds);
                analysis_results.push(result);
            }

            // Store results
            self.results.insert(node.id, analysis_results.clone());
            
            updated_node = updated_node.with_analysis_results(analysis_results.into());
            self.arena.update(node.id, updated_node);
        }

        Ok(())
    }

    fn analyze_class(&self, class: &ClassMetrics, thresholds: &Thresholds) -> AnalysisResult {
        if !class.is_god_class(thresholds) && class.god_method_count(thresholds) == 0 {
            return AnalysisResult::healthy(class.clone());
        }

        // Detect god methods
        let god_methods: Arc<[GodMethodResult]> = class
            .methods
            .iter()
            .filter(|m| m.is_god_method(thresholds))
            .map(|m| self.create_god_method_result(m, class, thresholds))
            .collect();

        let summary = if class.is_god_class(thresholds) {
            format!(
                "God class detected: {} (lines: {}, methods: {}, complexity: {})",
                class.name, class.lines.0, class.method_count.0, class.complexity.0
            )
        } else {
            format!("Class '{}' has {} god method(s)", class.name, god_methods.len())
        };

        AnalysisResult {
            class_metrics: class.clone(),
            is_god_class: class.is_god_class(thresholds),
            suggested_extractions: Arc::new([]), // Will be filled by clustering analyzer
            god_methods,
            analyzed_at: std::time::SystemTime::now(),
            summary: summary.into(),
        }
    }

    fn create_god_method_result(
        &self,
        method: &MethodMetrics,
        class: &ClassMetrics,
        thresholds: &Thresholds,
    ) -> GodMethodResult {
        let mut violations = Vec::new();

        if method.lines > thresholds.max_method_lines {
            violations.push(Violation {
                kind: ViolationKind::Lines,
                actual: method.lines.0,
                threshold: thresholds.max_method_lines.0,
            });
        }

        if method.complexity > thresholds.max_method_complexity {
            violations.push(Violation {
                kind: ViolationKind::Complexity,
                actual: method.complexity.0,
                threshold: thresholds.max_method_complexity.0,
            });
        }

        if method.parameters > thresholds.max_parameters {
            violations.push(Violation {
                kind: ViolationKind::ParameterCount,
                actual: method.parameters.0,
                threshold: thresholds.max_parameters.0,
            });
        }

        GodMethodResult {
            method_name: method.name.clone(),
            class_name: class.name.clone(),
            file_path: class.file_path.clone(),
            metrics: method.clone(),
            violations: violations.into(),
            violation_score: method.violation_score(thresholds),
        }
    }

    fn create_god_file_result(
        &self,
        file_metrics: &FileMetrics,
        thresholds: &Thresholds,
    ) -> GodFileResult {
        let mut violations = Vec::new();

        if file_metrics.classes.len() > thresholds.max_classes_per_file {
            violations.push(Violation {
                kind: ViolationKind::ClassesPerFile,
                actual: file_metrics.classes.len(),
                threshold: thresholds.max_classes_per_file,
            });
        }

        if file_metrics.lines > thresholds.max_file_lines {
            violations.push(Violation {
                kind: ViolationKind::Lines,
                actual: file_metrics.lines.0,
                threshold: thresholds.max_file_lines.0,
            });
        }

        GodFileResult {
            file_path: file_metrics.path.clone(),
            class_count: file_metrics.classes.len(),
            total_lines: file_metrics.lines.0,
            class_names: file_metrics
                .classes
                .iter()
                .map(|c| c.name.clone())
                .collect(),
            violations: violations.into(),
        }
    }

    pub fn get_results(&self, node_id: NodeId) -> Option<Vec<AnalysisResult>> {
        self.results.get(&node_id).map(|r| r.clone())
    }

    pub fn all_results(&self) -> Vec<AnalysisResult> {
        self.results
            .iter()
            .flat_map(|entry| entry.value().clone())
            .collect()
    }
}

