//! Rust language parser using tree-sitter and syn
//! 
//! Dual approach: tree-sitter for speed, syn for deep analysis

use dei_core::{error::Result, metrics::*, thresholds::*, Error};
use once_cell::sync::Lazy;
use std::path::Path;
use std::sync::Arc;
use tree_sitter::Parser;

use crate::complexity::ComplexityCalculator;

static RUST_LANGUAGE: Lazy<tree_sitter::Language> = Lazy::new(|| tree_sitter_rust::LANGUAGE.into());

/// Rust-specific parser
pub struct RustParser {
    parser: Parser,
}

impl RustParser {
    pub fn new() -> Result<Self> {
        let mut parser = Parser::new();
        parser
            .set_language(&*RUST_LANGUAGE)
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
        let mut type_defs = std::collections::HashMap::new();
        let mut impls = Vec::new();

        // First pass: collect type definitions and impl blocks
        let mut cursor = root.walk();
        for node in root.children(&mut cursor) {
            match node.kind() {
                "struct_item" | "enum_item" => {
                    if let Some(class_metrics) = self.parse_type(&node, source_bytes, path) {
                        let name = class_metrics.name.to_string();
                        type_defs.insert(name, class_metrics);
                    }
                }
                "impl_item" => {
                    if let Some(class_metrics) = self.parse_impl(&node, source_bytes, path) {
                        impls.push(class_metrics);
                    }
                }
                _ => {}
            }
        }

        // Second pass: merge impl blocks into type definitions
        for impl_metrics in impls {
            let type_name = impl_metrics.name.to_string();
            
            if let Some(type_def) = type_defs.get_mut(&type_name) {
                // Merge methods from impl into the type definition
                let mut all_methods = Vec::from(type_def.methods.as_ref());
                all_methods.extend_from_slice(&impl_metrics.methods);
                
                type_def.methods = all_methods.into();
                type_def.method_count = MethodCount(type_def.methods.len());
                type_def.lines = Lines(type_def.lines.0 + impl_metrics.lines.0);
                type_def.complexity = Complexity(type_def.complexity.0 + impl_metrics.complexity.0);
            } else {
                // Impl without a type definition in this file (e.g., impl for external type)
                type_defs.insert(type_name, impl_metrics);
            }
        }

        let lines = ComplexityCalculator::count_lines(&source);

        Ok(FileMetrics {
            path: path.to_string_lossy().to_string().into(),
            lines,
            classes: type_defs.into_values().collect::<Vec<_>>().into(),
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
        
        // Methods are inside the declaration_list
        for child in node.children(&mut cursor) {
            if child.kind() == "declaration_list" {
                let mut decl_cursor = child.walk();
                for decl_child in child.children(&mut decl_cursor) {
                    if decl_child.kind() == "function_item" {
                        if let Some(method) = self.parse_method(&decl_child, source) {
                            methods.push(method);
                        }
                    }
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
        let children: Vec<_> = node.children(&mut cursor).collect();
        children.iter().any(|c| c.kind() == "async")
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

