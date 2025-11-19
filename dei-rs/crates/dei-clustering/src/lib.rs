//! Advanced clustering using DBSCAN/HDBSCAN
//! 
//! Significant improvement over K-means:
//! - Finds optimal number of clusters automatically
//! - Handles noise/outliers better
//! - More robust for varying cluster densities

pub mod analyzer;
pub mod embeddings;
pub mod hdbscan;

pub use analyzer::ClusteringAnalyzer;

