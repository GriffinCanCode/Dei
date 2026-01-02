//! Java parser using tree-sitter

use dei_core::{error::Result, metrics::*, thresholds::*, Error};
use once_cell::sync::Lazy;
use std::path::Path;
use std::sync::Arc;
use tree_sitter::Parser;

use crate::complexity::ComplexityCalculator;

static JAVA_LANGUAGE: Lazy<tree_sitter::Language> = Lazy::new(|| tree_sitter_java::LANGUAGE.into());

/// Java parser
pub struct JavaParser {
    parser: Parser,
}

impl JavaParser {
    pub fn new() -> Result<Self> {
        let mut parser = Parser::new();
        parser
            .set_language(&*JAVA_LANGUAGE)
            .map_err(|e| Error::Analysis(format!("Failed to set Java language: {e}")))?;
        Ok(Self { parser })
    }

    pub fn parse_file(&mut self, path: &Path) -> Result<FileMetrics> {
        let source = std::fs::read_to_string(path)?;
        let source_bytes = source.as_bytes();

        let tree = self.parser.parse(&source, None).ok_or_else(|| Error::Parse {
            path: path.to_path_buf(),
            message: "Failed to parse Java file".into(),
        })?;

        let root = tree.root_node();
        let mut classes: Vec<ClassMetrics> = Vec::new();

        self.collect_classes(&root, source_bytes, path, &mut classes);

        Ok(FileMetrics {
            path: path.to_string_lossy().to_string().into(),
            lines: ComplexityCalculator::count_lines(&source),
            classes: classes.into(),
        })
    }

    fn collect_classes(
        &self,
        node: &tree_sitter::Node,
        source: &[u8],
        path: &Path,
        classes: &mut Vec<ClassMetrics>,
    ) {
        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            match child.kind() {
                "class_declaration" | "interface_declaration" | "enum_declaration" => {
                    if let Some(c) = self.parse_class(&child, source, path) {
                        classes.push(c);
                    }
                }
                _ => self.collect_classes(&child, source, path, classes),
            }
        }
    }

    fn parse_class(&self, node: &tree_sitter::Node, source: &[u8], path: &Path) -> Option<ClassMetrics> {
        let name = node.child_by_field_name("name")?.utf8_text(source).ok()?;
        let text = node.utf8_text(source).ok()?;
        let lines = ComplexityCalculator::count_lines(text);

        let mut methods = Vec::new();
        let mut field_count = 0;

        if let Some(body) = node.child_by_field_name("body") {
            let mut cursor = body.walk();
            for child in body.children(&mut cursor) {
                match child.kind() {
                    "method_declaration" | "constructor_declaration" => {
                        if let Some(m) = self.parse_method(&child, source) {
                            methods.push(m);
                        }
                    }
                    "field_declaration" => field_count += 1,
                    "class_declaration" | "interface_declaration" | "enum_declaration" => {
                        // Nested class - parse recursively
                        if let Some(c) = self.parse_class(&child, source, path) {
                            // Could add nested classes here if needed
                            let _ = c;
                        }
                    }
                    _ => {}
                }
            }
        }

        let total_complexity: usize = methods.iter().map(|m| m.complexity.0).sum();

        Some(ClassMetrics {
            name: name.into(),
            fully_qualified_name: name.into(),
            file_path: path.to_string_lossy().to_string().into(),
            lines,
            method_count: MethodCount(methods.len()),
            property_count: 0,
            field_count,
            complexity: Complexity(total_complexity.max(1)),
            methods: methods.into(),
            dependencies: Arc::new([]),
        })
    }

    fn parse_method(&self, node: &tree_sitter::Node, source: &[u8]) -> Option<MethodMetrics> {
        let name = node.child_by_field_name("name")?.utf8_text(source).ok()?;
        let text = node.utf8_text(source).ok()?;
        let lines = ComplexityCalculator::count_lines(text);
        let complexity = ComplexityCalculator::calculate_from_tree(node, source);
        let parameters = ComplexityCalculator::count_parameters(node, source);
        let tokens = ComplexityCalculator::extract_tokens(node, source);

        let return_type = node
            .child_by_field_name("type")
            .and_then(|t| t.utf8_text(source).ok())
            .unwrap_or("void");

        let modifiers = self.get_modifiers(node, source);
        let is_public = modifiers.contains(&"public");
        let is_static = modifiers.contains(&"static");

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
            is_async: false, // Java doesn't have async keyword
            tokens: tokens.into_iter().map(|s| s.into()).collect(),
        })
    }

    fn get_modifiers<'a>(&self, node: &tree_sitter::Node<'a>, source: &'a [u8]) -> Vec<&'a str> {
        let mut modifiers = Vec::new();
        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            if child.kind() == "modifiers" {
                let mut mod_cursor = child.walk();
                for m in child.children(&mut mod_cursor) {
                    if let Ok(text) = m.utf8_text(source) {
                        modifiers.push(text);
                    }
                }
            }
        }
        modifiers
    }
}

impl Default for JavaParser {
    fn default() -> Self {
        Self::new().expect("Failed to create Java parser")
    }
}
