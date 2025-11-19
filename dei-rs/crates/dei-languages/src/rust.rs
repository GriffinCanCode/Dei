//! Rust language parser using tree-sitter and syn
//! 
//! Dual approach: tree-sitter for speed, syn for deep analysis

use dei_core::{error::Result, metrics::*, models::Language, Error};
use once_cell::sync::Lazy;
use std::path::Path;
use std::sync::Arc;
use tree_sitter::{Parser, Query, QueryCursor};

use crate::complexity::ComplexityCalculator;

static RUST_LANGUAGE: Lazy<tree_sitter::Language> = Lazy::new(tree_sitter_rust::language);

/// Rust-specific parser
pub struct RustParser {
    parser: Parser,
}

impl RustParser {
    pub fn new() -> Result<Self> {
        let mut parser = Parser::new();
        parser
            .set_language(*RUST_LANGUAGE)
            .map_err(|e| Error::Analysis(format!("Failed to set Rust language: {}", e)))?;
        
        Ok(Self { parser })
    }

    pub fn parse_file(&mut self, path: &Path) -> Result<FileMetrics> {
        let source = std::fs::read_to_string(path)?;
        let source_bytes = source.as_bytes();

        let tree = self
            .parser
            .parse(&source, None)
            .ok_or_else(|| Error::Parse {
                path: path.to_path_buf(),
                message: "Failed to parse Rust file".into(),
            })?;

        let root = tree.root_node();
        let mut classes = Vec::new();

        // Find all structs, enums, and impls
        let mut cursor = root.walk();
        for node in root.children(&mut cursor) {
            match node.kind() {
                "struct_item" | "enum_item" => {
                    if let Some(class_metrics) = self.parse_type(&node, source_bytes, path) {
                        classes.push(class_metrics);
                    }
                }
                "impl_item" => {
                    // Parse implementation blocks
                    if let Some(class_metrics) = self.parse_impl(&node, source_bytes, path) {
                        classes.push(class_metrics);
                    }
                }
                _ => {}
            }
        }

        let lines = ComplexityCalculator::count_lines(&source);

        Ok(FileMetrics {
            path: path.to_string_lossy().to_string().into(),
            lines,
            classes: classes.into(),
        })
    }

    fn parse_type(
        &self,
        node: &tree_sitter::Node,
        source: &[u8],
        path: &Path,
    ) -> Option<ClassMetrics> {
        let name = node
            .child_by_field_name("name")?
            .utf8_text(source)
            .ok()?;

        let text = node.utf8_text(source).ok()?;
        let lines = ComplexityCalculator::count_lines(text);
        
        // For structs/enums, methods are in separate impl blocks
        // This is a simplified version - in practice, we'd need to correlate impls
        
        Some(ClassMetrics {
            name: name.into(),
            fully_qualified_name: name.into(), // Would need full module path
            file_path: path.to_string_lossy().to_string().into(),
            lines,
            method_count: MethodCount(0),
            property_count: self.count_fields(node),
            field_count: self.count_fields(node),
            complexity: Complexity(1),
            methods: Arc::new([]),
            dependencies: Arc::new([]),
        })
    }

    fn parse_impl(
        &self,
        node: &tree_sitter::Node,
        source: &[u8],
        path: &Path,
    ) -> Option<ClassMetrics> {
        let type_name = node
            .child_by_field_name("type")?
            .utf8_text(source)
            .ok()?;

        let text = node.utf8_text(source).ok()?;
        let lines = ComplexityCalculator::count_lines(text);

        let mut methods = Vec::new();
        let mut cursor = node.walk();
        
        for child in node.children(&mut cursor) {
            if child.kind() == "function_item" {
                if let Some(method) = self.parse_method(&child, source) {
                    methods.push(method);
                }
            }
        }

        let total_complexity = methods
            .iter()
            .map(|m| m.complexity.0)
            .sum::<usize>();

        Some(ClassMetrics {
            name: type_name.into(),
            fully_qualified_name: type_name.into(),
            file_path: path.to_string_lossy().to_string().into(),
            lines,
            method_count: MethodCount(methods.len()),
            property_count: 0,
            field_count: 0,
            complexity: Complexity(total_complexity),
            methods: methods.into(),
            dependencies: Arc::new([]),
        })
    }

    fn parse_method(
        &self,
        node: &tree_sitter::Node,
        source: &[u8],
    ) -> Option<MethodMetrics> {
        let name = node
            .child_by_field_name("name")?
            .utf8_text(source)
            .ok()?;

        let text = node.utf8_text(source).ok()?;
        let lines = ComplexityCalculator::count_lines(text);
        let complexity = ComplexityCalculator::calculate_from_tree(node, source);
        let parameters = ComplexityCalculator::count_parameters(node, source);

        let return_type = node
            .child_by_field_name("return_type")
            .and_then(|n| n.utf8_text(source).ok())
            .unwrap_or("()");

        let is_public = node
            .children(&mut node.walk())
            .any(|c| c.kind() == "visibility_modifier" && c.utf8_text(source).ok() == Some("pub"));

        let tokens = ComplexityCalculator::extract_tokens(node, source);

        Some(MethodMetrics {
            name: name.into(),
            lines,
            complexity,
            parameters,
            called_methods: Arc::new([]),
            accessed_fields: Arc::new([]),
            return_type: return_type.into(),
            is_public,
            is_static: false, // Rust doesn't have static methods in the same way
            is_async: self.is_async_fn(node),
            tokens: tokens.into_iter().map(|s| s.into()).collect(),
        })
    }

    fn is_async_fn(&self, node: &tree_sitter::Node) -> bool {
        let mut cursor = node.walk();
        node.children(&mut cursor)
            .any(|c| c.kind() == "async")
    }

    fn count_fields(&self, node: &tree_sitter::Node) -> usize {
        let mut cursor = node.walk();
        node.children(&mut cursor)
            .filter(|c| matches!(c.kind(), "field_declaration" | "field_declaration_list"))
            .count()
    }
}

impl Default for RustParser {
    fn default() -> Self {
        Self::new().expect("Failed to create Rust parser")
    }
}

