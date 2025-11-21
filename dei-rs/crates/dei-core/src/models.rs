//! Analysis result models

use serde::{Deserialize, Serialize};
use std::sync::Arc;
use std::time::SystemTime;

use crate::metrics::*;

/// Represents a cluster of methods with shared responsibility
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResponsibilityCluster {
    pub suggested_name: Arc<str>,
    pub methods: Arc<[Arc<str>]>, // Method names
    pub cohesion_score: f64,
    pub shared_dependencies: Arc<[Arc<str>]>,
    pub justification: Arc<str>,
}

/// Analysis result for a god method
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GodMethodResult {
    pub method_name: Arc<str>,
    pub class_name: Arc<str>,
    pub file_path: Arc<str>,
    pub metrics: MethodMetrics,
    pub violations: Arc<[Violation]>,
    pub violation_score: f64,
}

/// Analysis result for a god file
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GodFileResult {
    pub file_path: Arc<str>,
    pub class_count: usize,
    pub total_lines: usize,
    pub class_names: Arc<[Arc<str>]>,
    pub violations: Arc<[Violation]>,
}

/// Specific threshold violation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Violation {
    pub kind: ViolationKind,
    pub actual: usize,
    pub threshold: usize,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ViolationKind {
    Lines,
    Complexity,
    MethodCount,
    ParameterCount,
    ClassesPerFile,
}

/// Complete analysis result for a class
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnalysisResult {
    pub class_metrics: ClassMetrics,
    pub is_god_class: bool,
    pub suggested_extractions: Arc<[ResponsibilityCluster]>,
    pub god_methods: Arc<[GodMethodResult]>,
    #[serde(skip_serializing, default = "default_systemtime")]
    pub analyzed_at: SystemTime,
    pub summary: Arc<str>,
}

fn default_systemtime() -> SystemTime {
    SystemTime::now()
}

impl AnalysisResult {
    pub fn healthy(metrics: ClassMetrics) -> Self {
        Self {
            summary: format!("Class '{}' is within acceptable thresholds", metrics.name).into(),
            class_metrics: metrics,
            is_god_class: false,
            suggested_extractions: Arc::new([]),
            god_methods: Arc::new([]),
            analyzed_at: SystemTime::now(),
        }
    }

    pub fn has_issues(&self) -> bool {
        self.is_god_class || !self.god_methods.is_empty()
    }
}

/// Language being analyzed
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Language {
    Rust,
    CSharp,
    Python,
    JavaScript,
    TypeScript,
    Go,
    Java,
}

impl Language {
    pub fn from_extension(ext: &str) -> Option<Self> {
        match ext {
            "rs" => Some(Language::Rust),
            "cs" => Some(Language::CSharp),
            "py" => Some(Language::Python),
            "js" => Some(Language::JavaScript),
            "ts" => Some(Language::TypeScript),
            "go" => Some(Language::Go),
            "java" => Some(Language::Java),
            _ => None,
        }
    }

    pub fn extensions(&self) -> &[&str] {
        match self {
            Language::Rust => &["rs"],
            Language::CSharp => &["cs"],
            Language::Python => &["py"],
            Language::JavaScript => &["js"],
            Language::TypeScript => &["ts"],
            Language::Go => &["go"],
            Language::Java => &["java"],
        }
    }
}

