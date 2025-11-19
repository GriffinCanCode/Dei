//! Advanced metrics beyond basic complexity
//! 
//! Includes graph-based analysis for coupling detection

pub mod coupling;
pub mod graph;

pub use coupling::CouplingAnalyzer;
pub use graph::DependencyGraph;

