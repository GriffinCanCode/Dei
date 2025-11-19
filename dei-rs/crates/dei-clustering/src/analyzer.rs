//! High-level clustering analyzer
//! 
//! Orchestrates feature extraction, clustering, and cluster naming

use dei_core::{
    error::Result,
    metrics::ClassMetrics,
    models::ResponsibilityCluster,
    thresholds::Thresholds,
    traits::ClusterAnalyzer,
    Error,
};
use std::collections::HashMap;
use std::sync::Arc;

use crate::{embeddings, hdbscan::DbscanClusterer};

pub struct ClusteringAnalyzer {
    clusterer: DbscanClusterer,
}

impl ClusteringAnalyzer {
    pub fn new() -> Self {
        Self {
            clusterer: DbscanClusterer::default(),
        }
    }

    pub fn with_params(min_points: usize, tolerance: f64) -> Self {
        Self {
            clusterer: DbscanClusterer::new(min_points, tolerance),
        }
    }

    /// Generate cluster name from common tokens
    fn generate_cluster_name(
        &self,
        method_indices: &[usize],
        methods: &[dei_core::metrics::MethodMetrics],
        original_class: &str,
    ) -> String {
        let mut token_freq: HashMap<String, usize> = HashMap::new();

        for &idx in method_indices {
            if let Some(method) = methods.get(idx) {
                for token in method.tokens.iter() {
                    *token_freq.entry(token.to_string()).or_insert(0) += 1;
                }
            }
        }

        // Filter common verbs
        let common_words = [
            "get", "set", "add", "remove", "delete", "update", "create", "save",
            "load", "handle", "process", "execute", "run", "do", "is", "has", "can",
        ];

        let top_tokens: Vec<String> = token_freq
            .into_iter()
            .filter(|(token, _)| !common_words.contains(&token.as_str()) && token.len() > 2)
            .max_set_by_key(|(_, count)| *count)
            .into_iter()
            .take(2)
            .map(|(token, _)| capitalize_first(&token))
            .collect();

        if top_tokens.is_empty() {
            format!("{}Component", original_class)
        } else {
            format!("{}Service", top_tokens.join(""))
        }
    }

    /// Calculate cohesion score for a cluster
    fn calculate_cohesion(
        &self,
        method_indices: &[usize],
        methods: &[dei_core::metrics::MethodMetrics],
    ) -> f64 {
        if method_indices.len() < 2 {
            return 0.5;
        }

        // Calculate based on shared accessed fields
        let mut all_fields = Vec::new();
        for &idx in method_indices {
            if let Some(method) = methods.get(idx) {
                all_fields.extend(method.accessed_fields.iter().map(|s| s.as_ref()));
            }
        }

        let mut field_counts: HashMap<&str, usize> = HashMap::new();
        for field in &all_fields {
            *field_counts.entry(field).or_insert(0) += 1;
        }

        let shared_fields = field_counts
            .values()
            .filter(|&&count| count >= method_indices.len() / 2)
            .count();

        let avg_fields_per_method = all_fields.len() as f64 / method_indices.len() as f64;

        if avg_fields_per_method == 0.0 {
            0.3
        } else {
            (shared_fields as f64 / avg_fields_per_method).min(1.0)
        }
    }

    /// Generate justification text for a cluster
    fn generate_justification(
        &self,
        method_indices: &[usize],
        methods: &[dei_core::metrics::MethodMetrics],
    ) -> String {
        let method_names: Vec<&str> = method_indices
            .iter()
            .filter_map(|&idx| methods.get(idx).map(|m| m.name.as_ref()))
            .take(5)
            .collect();

        let names_str = method_names.join(", ");
        
        format!(
            "Cohesive group of {} method(s): {}",
            method_indices.len(),
            names_str
        )
    }
}

impl ClusterAnalyzer for ClusteringAnalyzer {
    fn analyze(
        &self,
        class: &ClassMetrics,
        thresholds: &Thresholds,
    ) -> Result<Vec<ResponsibilityCluster>> {
        let methods: Vec<_> = class.methods.iter().cloned().collect();

        if methods.len() < thresholds.min_cluster_size {
            return Ok(Vec::new());
        }

        // Build feature matrix
        let (features, _vocab) = embeddings::build_feature_matrix(&methods);

        // Perform clustering
        let labels = self.clusterer.cluster(&features);

        // Group methods by cluster
        let mut clusters: HashMap<usize, Vec<usize>> = HashMap::new();
        for (idx, label) in labels.iter().enumerate() {
            if let Some(cluster_id) = label {
                clusters.entry(*cluster_id).or_default().push(idx);
            }
        }

        // Create responsibility clusters
        let mut result = Vec::new();
        for (_, method_indices) in clusters {
            if method_indices.len() < thresholds.min_cluster_size {
                continue;
            }

            let suggested_name = self.generate_cluster_name(
                &method_indices,
                &methods,
                class.name.as_ref(),
            );

            let cohesion = self.calculate_cohesion(&method_indices, &methods);
            let justification = self.generate_justification(&method_indices, &methods);

            let method_names: Arc<[Arc<str>]> = method_indices
                .iter()
                .filter_map(|&idx| methods.get(idx).map(|m| m.name.clone()))
                .collect();

            let shared_deps: Arc<[Arc<str>]> = Arc::new([]);

            result.push(ResponsibilityCluster {
                suggested_name: suggested_name.into(),
                methods: method_names,
                cohesion_score: cohesion,
                shared_dependencies: shared_deps,
                justification: justification.into(),
            });
        }

        Ok(result)
    }
}

impl Default for ClusteringAnalyzer {
    fn default() -> Self {
        Self::new()
    }
}

fn capitalize_first(s: &str) -> String {
    let mut chars = s.chars();
    match chars.next() {
        None => String::new(),
        Some(first) => first.to_uppercase().chain(chars).collect(),
    }
}

