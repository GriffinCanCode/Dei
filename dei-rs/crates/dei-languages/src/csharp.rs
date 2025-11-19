//! C# language parser using tree-sitter
//! 
//! Compatible with the original C# version but using Rust for performance

use dei_core::{error::Result, metrics::*, models::Language, Error};
use once_cell::sync::Lazy;
use std::path::Path;
use std::sync::Arc;
use tree_sitter::Parser;

use crate::complexity::ComplexityCalculator;

static CSHARP_LANGUAGE: Lazy<tree_sitter::Language> = Lazy::new(tree_sitter_c_sharp::language);

/// C#-specific parser
pub struct CSharpParser {
    parser: Parser,
}

impl CSharpParser {
    pub fn new() -> Result<Self> {
        let mut parser = Parser::new();
        parser
            .set_language(*CSHARP_LANGUAGE)
            .map_err(|e| Error::Analysis(format!("Failed to set C# language: {}", e)))?;
        
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
                message: "Failed to parse C# file".into(),
            })?;

        let root = tree.root_node();
        let mut classes = Vec::new();

        let mut cursor = root.walk();
        for node in root.children(&mut cursor) {
            if node.kind() == "class_declaration" {
                if let Some(class_metrics) = self.parse_class(&node, source_bytes, path) {
                    classes.push(class_metrics);
                }
            }
        }

        let lines = ComplexityCalculator::count_lines(&source);

        Ok(FileMetrics {
            path: path.to_string_lossy().to_string().into(),
            lines,
            classes: classes.into(),
        })
    }

    fn parse_class(
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

        let mut methods = Vec::new();
        let mut property_count = 0;
        let mut field_count = 0;

        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            match child.kind() {
                "method_declaration" => {
                    if let Some(method) = self.parse_method(&child, source) {
                        methods.push(method);
                    }
                }
                "property_declaration" => property_count += 1,
                "field_declaration" => field_count += 1,
                _ => {}
            }
        }

        let total_complexity = methods
            .iter()
            .map(|m| m.complexity.0)
            .sum::<usize>();

        Some(ClassMetrics {
            name: name.into(),
            fully_qualified_name: name.into(),
            file_path: path.to_string_lossy().to_string().into(),
            lines,
            method_count: MethodCount(methods.len()),
            property_count,
            field_count,
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
            .child_by_field_name("type")
            .and_then(|n| n.utf8_text(source).ok())
            .unwrap_or("void");

        let is_public = node
            .children(&mut node.walk())
            .any(|c| c.kind() == "public");

        let is_static = node
            .children(&mut node.walk())
            .any(|c| c.kind() == "static");

        let is_async = node
            .children(&mut node.walk())
            .any(|c| c.kind() == "async");

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
            is_static,
            is_async,
            tokens: tokens.into_iter().map(|s| s.into()).collect(),
        })
    }
}

impl Default for CSharpParser {
    fn default() -> Self {
        Self::new().expect("Failed to create C# parser")
    }
}

