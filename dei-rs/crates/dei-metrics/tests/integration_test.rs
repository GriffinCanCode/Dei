use dei_metrics::{CouplingAnalyzer, DependencyGraph, graph::EdgeKind};
use std::sync::Arc;

#[test]
fn test_dependency_graph_creation() {
    let mut graph = DependencyGraph::new();
    
    let node_a: Arc<str> = "ModuleA".into();
    let node_b: Arc<str> = "ModuleB".into();
    let node_c: Arc<str> = "ModuleC".into();
    
    graph.add_edge(node_a.clone(), node_b.clone(), EdgeKind::Uses);
    graph.add_edge(node_b.clone(), node_c.clone(), EdgeKind::Uses);
    
    // Verify density is calculated (implicit node/edge count check)
    let density = graph.density();
    assert!(density > 0.0);
}

#[test]
fn test_cyclic_dependency_detection() {
    let mut graph = DependencyGraph::new();
    
    let a: Arc<str> = "A".into();
    let b: Arc<str> = "B".into();
    let c: Arc<str> = "C".into();
    
    graph.add_edge(a.clone(), b.clone(), EdgeKind::Uses);
    graph.add_edge(b.clone(), c.clone(), EdgeKind::Uses);
    graph.add_edge(c.clone(), a.clone(), EdgeKind::Uses); // Creates cycle
    
    let cycles = graph.find_cycles();
    assert!(!cycles.is_empty(), "Should detect circular dependency");
}

#[test]
fn test_acyclic_graph() {
    let mut graph = DependencyGraph::new();
    
    let a: Arc<str> = "A".into();
    let b: Arc<str> = "B".into();
    let c: Arc<str> = "C".into();
    
    graph.add_edge(a.clone(), b.clone(), EdgeKind::Uses);
    graph.add_edge(b.clone(), c.clone(), EdgeKind::Uses);
    
    let cycles = graph.find_cycles();
    assert!(cycles.is_empty(), "Should not detect cycles in acyclic graph");
}

#[test]
fn test_coupling_analyzer_basic() {
    let mut analyzer = CouplingAnalyzer::new();
    
    // Test that analyzer initializes correctly and can build from empty data
    analyzer.build_graph(&[]);
    let quality = analyzer.architecture_quality();
    assert_eq!(quality.n_cycles, 0);
}

#[test]
fn test_empty_graph() {
    let graph = DependencyGraph::new();
    let density = graph.density();
    assert_eq!(density, 0.0);
    
    let cycles = graph.find_cycles();
    assert!(cycles.is_empty());
}

#[test]
fn test_single_node_graph() {
    let mut graph = DependencyGraph::new();
    let node: Arc<str> = "SingleNode".into();
    
    graph.add_node(node.clone());
    
    let density = graph.density();
    assert_eq!(density, 0.0, "Single node should have zero density");
}

#[test]
fn test_coupling_metrics() {
    let mut graph = DependencyGraph::new();
    
    let a: Arc<str> = "A".into();
    let b: Arc<str> = "B".into();
    let c: Arc<str> = "C".into();
    
    graph.add_edge(a.clone(), b.clone(), EdgeKind::Uses);
    graph.add_edge(c.clone(), a.clone(), EdgeKind::Uses);
    
    let metrics = graph.coupling_metrics(&a);
    assert!(metrics.is_some());
    
    let metrics = metrics.unwrap();
    assert_eq!(metrics.afferent, 1); // One incoming from C
    assert_eq!(metrics.efferent, 1); // One outgoing to B
}

#[test]
fn test_complex_dependency_structure() {
    let mut graph = DependencyGraph::new();
    
    let core: Arc<str> = "Core".into();
    let utils: Arc<str> = "Utils".into();
    let api: Arc<str> = "Api".into();
    let db: Arc<str> = "Database".into();
    let auth: Arc<str> = "Auth".into();
    
    // Core dependencies
    graph.add_edge(api.clone(), core.clone(), EdgeKind::Uses);
    graph.add_edge(api.clone(), utils.clone(), EdgeKind::Uses);
    graph.add_edge(api.clone(), auth.clone(), EdgeKind::Uses);
    
    // Database dependencies
    graph.add_edge(db.clone(), core.clone(), EdgeKind::Uses);
    graph.add_edge(api.clone(), db.clone(), EdgeKind::Uses);
    
    // Auth dependencies
    graph.add_edge(auth.clone(), core.clone(), EdgeKind::Uses);
    graph.add_edge(auth.clone(), db.clone(), EdgeKind::Uses);
    
    let density = graph.density();
    assert!(density > 0.0, "Complex graph should have positive density");
    
    // Core should have high afferent coupling (many things depend on it)
    let core_metrics = graph.coupling_metrics(&core).unwrap();
    assert!(core_metrics.afferent >= 3);
}

