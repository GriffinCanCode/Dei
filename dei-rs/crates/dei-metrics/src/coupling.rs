//! Coupling analysis for classes
//! 
//! New capability not in C# version - analyzes inter-class dependencies

use dei_core::metrics::ClassMetrics;
use std::collections::HashMap;
use std::sync::Arc;

use crate::graph::{DependencyGraph, EdgeKind};

/// Analyzes coupling between classes
pub struct CouplingAnalyzer {
    graph: DependencyGraph,
}

impl CouplingAnalyzer {
    pub fn new() -> Self {
        Self {
            graph: DependencyGraph::new(),
        }
    }

    /// Build dependency graph from class metrics
    pub fn build_graph(&mut self, classes: &[ClassMetrics]) {
        for class in classes {
            let class_name = class.name.clone();
            self.graph.add_node(class_name.clone());

            // Add dependencies
            for dep in class.dependencies.iter() {
                self.graph.add_edge(class_name.clone(), dep.clone(), EdgeKind::Uses);
            }

            // Add method calls as edges
            for method in class.methods.iter() {
                for called in method.called_methods.iter() {
                    // Heuristic: if called method looks like external class
                    if called.contains('.') || called.chars().next().map_or(false, |c| c.is_uppercase()) {
                        self.graph.add_edge(class_name.clone(), called.clone(), EdgeKind::Calls);
                    }
                }
            }
        }
    }

    /// Get coupling metrics for a class
    pub fn get_coupling(&self, class_name: &Arc<str>) -> Option<crate::graph::CouplingMetrics> {
        self.graph.coupling_metrics(class_name)
    }

    /// Find tightly coupled classes (circular dependencies)
    pub fn find_tight_coupling(&self) -> Vec<Vec<Arc<str>>> {
        self.graph.find_cycles()
    }

    /// Calculate overall architecture quality metric
    pub fn architecture_quality(&self) -> ArchitectureMetrics {
        let density = self.graph.density();
        let cycles = self.find_tight_coupling();
        let cyclomatic_quality = if cycles.is_empty() { 1.0 } else { 1.0 / (1.0 + cycles.len() as f64) };

        ArchitectureMetrics {
            density,
            n_cycles: cycles.len(),
            cyclomatic_quality,
            maintainability_index: (1.0 - density) * cyclomatic_quality,
        }
    }
}

impl Default for CouplingAnalyzer {
    fn default() -> Self {
        Self::new()
    }
}

/// High-level architecture metrics
#[derive(Debug, Clone)]
pub struct ArchitectureMetrics {
    pub density: f64,
    pub n_cycles: usize,
    pub cyclomatic_quality: f64,
    pub maintainability_index: f64,
}

