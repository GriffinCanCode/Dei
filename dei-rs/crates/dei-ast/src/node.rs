//! AST node definitions with zero-copy strings

use dei_core::{metrics::*, models::*};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::sync::Arc;

/// Node identifier using generational indexing for safety
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct NodeId(pub usize);

/// AST node representing file system or code structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Node {
    pub id: NodeId,
    pub kind: NodeKind,
    pub path: Arc<str>,
    pub name: Arc<str>,
    pub depth: usize,
    pub parent: Option<NodeId>,
    pub children: Arc<[NodeId]>,
    
    // Analysis results (populated during traversal)
    pub file_metrics: Option<FileMetrics>,
    pub analysis_results: Arc<[AnalysisResult]>,
    pub god_file_result: Option<GodFileResult>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum NodeKind {
    Directory,
    File,
}

impl Node {
    pub fn new_directory(id: NodeId, path: PathBuf, depth: usize) -> Self {
        let name: Arc<str> = path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("")
            .into();
        
        Self {
            id,
            kind: NodeKind::Directory,
            path: path.to_string_lossy().to_string().into(),
            name,
            depth,
            parent: None,
            children: Arc::new([]),
            file_metrics: None,
            analysis_results: Arc::new([]),
            god_file_result: None,
        }
    }

    pub fn new_file(id: NodeId, path: PathBuf, depth: usize) -> Self {
        let name: Arc<str> = path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("")
            .into();
        
        Self {
            id,
            kind: NodeKind::File,
            path: path.to_string_lossy().to_string().into(),
            name,
            depth,
            parent: None,
            children: Arc::new([]),
            file_metrics: None,
            analysis_results: Arc::new([]),
            god_file_result: None,
        }
    }

    pub fn is_file(&self) -> bool {
        matches!(self.kind, NodeKind::File)
    }

    pub fn is_directory(&self) -> bool {
        matches!(self.kind, NodeKind::Directory)
    }

    pub fn language(&self) -> Option<Language> {
        if !self.is_file() {
            return None;
        }
        
        let ext = std::path::Path::new(self.path.as_ref())
            .extension()?
            .to_str()?;
        
        Language::from_extension(ext)
    }

    pub fn has_issues(&self) -> bool {
        self.analysis_results.iter().any(|r| r.has_issues())
            || self.god_file_result.as_ref().is_some()
    }

    pub fn with_children(mut self, children: Arc<[NodeId]>) -> Self {
        self.children = children;
        self
    }

    pub fn with_parent(mut self, parent: NodeId) -> Self {
        self.parent = Some(parent);
        self
    }

    pub fn with_file_metrics(mut self, metrics: FileMetrics) -> Self {
        self.file_metrics = Some(metrics);
        self
    }

    pub fn with_analysis_results(mut self, results: Arc<[AnalysisResult]>) -> Self {
        self.analysis_results = results;
        self
    }

    pub fn with_god_file_result(mut self, result: GodFileResult) -> Self {
        self.god_file_result = Some(result);
        self
    }
}

