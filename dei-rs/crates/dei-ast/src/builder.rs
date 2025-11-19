//! AST builder for constructing filesystem trees

use dei_core::{error::Result, Error};
use ignore::WalkBuilder;
use std::path::Path;
use std::sync::Arc;

use crate::{
    arena::SharedArena,
    node::{Node, NodeId},
};

/// Builds filesystem AST with smart filtering
pub struct AstBuilder {
    arena: SharedArena,
    ignore_patterns: Vec<String>,
}

impl AstBuilder {
    pub fn new() -> Self {
        Self {
            arena: SharedArena::new(),
            ignore_patterns: Self::default_ignore_patterns(),
        }
    }

    pub fn with_arena(arena: SharedArena) -> Self {
        Self {
            arena,
            ignore_patterns: Self::default_ignore_patterns(),
        }
    }

    /// Default patterns to ignore (build artifacts, etc.)
    fn default_ignore_patterns() -> Vec<String> {
        vec![
            "target".into(),
            "bin".into(),
            "obj".into(),
            "node_modules".into(),
            ".git".into(),
            "dist".into(),
            "build".into(),
            "__pycache__".into(),
        ]
    }

    pub fn add_ignore_pattern(&mut self, pattern: String) {
        self.ignore_patterns.push(pattern);
    }

    /// Build AST from a directory path
    pub fn build(&self, root: &Path) -> Result<NodeId> {
        if !root.exists() {
            return Err(Error::PathNotFound(root.to_path_buf()));
        }

        let root_id = if root.is_dir() {
            self.build_directory(root, 0, None)?
        } else {
            self.build_file(root, 0, None)?
        };

        Ok(root_id)
    }

    fn build_directory(&self, path: &Path, depth: usize, parent: Option<NodeId>) -> Result<NodeId> {
        let node = Node::new_directory(NodeId(0), path.to_path_buf(), depth);
        let node_id = self.arena.alloc(node);

        let mut children = Vec::new();

        // Use ignore crate for smart traversal
        for entry in WalkBuilder::new(path)
            .max_depth(Some(1))
            .hidden(false)
            .build()
            .skip(1) // Skip root itself
        {
            let entry = entry.map_err(|e| Error::Io(std::io::Error::other(e.to_string())))?;
            let entry_path = entry.path();

            // Skip ignored patterns
            if self.should_ignore(entry_path) {
                continue;
            }

            let child_id = if entry_path.is_dir() {
                self.build_directory(entry_path, depth + 1, Some(node_id))?
            } else {
                self.build_file(entry_path, depth + 1, Some(node_id))?
            };

            children.push(child_id);
        }

        // Update node with children
        if let Some(mut node) = self.arena.get(node_id) {
            node = node.with_children(children.into());
            if let Some(parent_id) = parent {
                node = node.with_parent(parent_id);
            }
            self.arena.update(node_id, node);
        }

        Ok(node_id)
    }

    fn build_file(&self, path: &Path, depth: usize, parent: Option<NodeId>) -> Result<NodeId> {
        let mut node = Node::new_file(NodeId(0), path.to_path_buf(), depth);
        
        if let Some(parent_id) = parent {
            node = node.with_parent(parent_id);
        }
        
        let node_id = self.arena.alloc(node);
        Ok(node_id)
    }

    fn should_ignore(&self, path: &Path) -> bool {
        path.components().any(|c| {
            if let Some(name) = c.as_os_str().to_str() {
                self.ignore_patterns.iter().any(|p| name.contains(p.as_str()))
            } else {
                false
            }
        })
    }

    pub fn arena(&self) -> &SharedArena {
        &self.arena
    }
}

impl Default for AstBuilder {
    fn default() -> Self {
        Self::new()
    }
}

