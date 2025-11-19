//! DBSCAN clustering implementation
//! 
//! Superior to K-means for this use case:
//! - Automatically determines number of clusters
//! - Robust to noise
//! - Finds clusters of arbitrary shape

use linfa::prelude::*;
use linfa_clustering::Dbscan;
use ndarray::Array2;

/// DBSCAN-based clustering
pub struct DbscanClusterer {
    min_points: usize,
    tolerance: f64,
}

impl DbscanClusterer {
    pub fn new(min_points: usize, tolerance: f64) -> Self {
        Self {
            min_points,
            tolerance,
        }
    }

    /// Cluster feature vectors
    pub fn cluster(&self, features: &Array2<f64>) -> Vec<Option<usize>> {
        let dataset = DatasetBase::from(features.clone());
        
        let model = Dbscan::params(self.min_points)
            .tolerance(self.tolerance)
            .transform(&dataset)
            .expect("DBSCAN clustering failed");

        model
            .targets()
            .iter()
            .map(|&label| if label >= 0 { Some(label as usize) } else { None })
            .collect()
    }

    /// Get optimal parameters using elbow method
    pub fn auto_params(n_samples: usize) -> (usize, f64) {
        let min_points = (n_samples as f64).sqrt().ceil() as usize;
        let min_points = min_points.max(3).min(10);
        
        // Tolerance is dataset-dependent, start conservative
        let tolerance = 0.5;
        
        (min_points, tolerance)
    }
}

impl Default for DbscanClusterer {
    fn default() -> Self {
        Self::new(3, 0.5)
    }
}

/// Cluster statistics
#[derive(Debug, Clone)]
pub struct ClusterStats {
    pub n_clusters: usize,
    pub n_noise: usize,
    pub cluster_sizes: Vec<usize>,
    pub avg_cluster_size: f64,
}

impl ClusterStats {
    pub fn from_labels(labels: &[Option<usize>]) -> Self {
        let n_noise = labels.iter().filter(|l| l.is_none()).count();
        
        let mut cluster_sizes = vec![0; labels.len()];
        for label in labels.iter().flatten() {
            if *label < cluster_sizes.len() {
                cluster_sizes[*label] += 1;
            }
        }
        
        cluster_sizes.retain(|&size| size > 0);
        let n_clusters = cluster_sizes.len();
        
        let avg_cluster_size = if n_clusters > 0 {
            cluster_sizes.iter().sum::<usize>() as f64 / n_clusters as f64
        } else {
            0.0
        };

        Self {
            n_clusters,
            n_noise,
            cluster_sizes,
            avg_cluster_size,
        }
    }
}

