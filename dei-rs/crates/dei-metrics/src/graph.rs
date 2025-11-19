//! Dependency graph for analyzing coupling
//! 
//! Extension beyond the C# version - provides graph-based insights

use petgraph::graph::{DiGraph, NodeIndex};
use petgraph::visit::EdgeRef;
use std::collections::HashMap;
use std::sync::Arc;

/// Represents a dependency graph between classes/methods
#[derive(Debug, Clone)]
pub struct DependencyGraph {
    graph: DiGraph<Arc<str>, EdgeKind>,
    node_map: HashMap<Arc<str>, NodeIndex>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EdgeKind {
    Calls,
    Inherits,
    Uses,
    Implements,
}

impl DependencyGraph {
    pub fn new() -> Self {
        Self {
            graph: DiGraph::new(),
            node_map: HashMap::new(),
        }
    }

    /// Add a node (class or method)
    pub fn add_node(&mut self, name: Arc<str>) -> NodeIndex {
        if let Some(&idx) = self.node_map.get(&name) {
            return idx;
        }

        let idx = self.graph.add_node(name.clone());
        self.node_map.insert(name, idx);
        idx
    }

    /// Add an edge between nodes
    pub fn add_edge(&mut self, from: Arc<str>, to: Arc<str>, kind: EdgeKind) {
        let from_idx = self.add_node(from);
        let to_idx = self.add_node(to);
        self.graph.add_edge(from_idx, to_idx, kind);
    }

    /// Calculate coupling metrics
    pub fn coupling_metrics(&self, node: &Arc<str>) -> Option<CouplingMetrics> {
        let idx = *self.node_map.get(node)?;

        let incoming = self
            .graph
            .edges_directed(idx, petgraph::Direction::Incoming)
            .count();
        
        let outgoing = self
            .graph
            .edges_directed(idx, petgraph::Direction::Outgoing)
            .count();

        Some(CouplingMetrics {
            afferent: incoming,
            efferent: outgoing,
            instability: if incoming + outgoing > 0 {
                outgoing as f64 / (incoming + outgoing) as f64
            } else {
                0.0
            },
        })
    }

    /// Find strongly connected components (circular dependencies)
    pub fn find_cycles(&self) -> Vec<Vec<Arc<str>>> {
        let sccs = petgraph::algo::kosaraju_scc(&self.graph);
        
        sccs.into_iter()
            .filter(|scc| scc.len() > 1)
            .map(|scc| {
                scc.into_iter()
                    .map(|idx| self.graph[idx].clone())
                    .collect()
            })
            .collect()
    }

    /// Calculate graph density
    pub fn density(&self) -> f64 {
        let n = self.graph.node_count();
        let e = self.graph.edge_count();
        
        if n <= 1 {
            0.0
        } else {
            e as f64 / (n * (n - 1)) as f64
        }
    }
}

impl Default for DependencyGraph {
    fn default() -> Self {
        Self::new()
    }
}

/// Coupling metrics for a node
#[derive(Debug, Clone)]
pub struct CouplingMetrics {
    pub afferent: usize,  // Incoming dependencies
    pub efferent: usize,  // Outgoing dependencies
    pub instability: f64, // Efferent / (Afferent + Efferent)
}

