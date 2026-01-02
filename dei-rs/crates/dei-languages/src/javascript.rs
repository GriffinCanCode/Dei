//! JavaScript/TypeScript parser using tree-sitter

use dei_core::{error::Result, metrics::*, thresholds::*, Error};
use once_cell::sync::Lazy;
use std::path::Path;
use std::sync::Arc;
use tree_sitter::Parser;

use crate::complexity::ComplexityCalculator;

static JS_LANGUAGE: Lazy<tree_sitter::Language> = Lazy::new(|| tree_sitter_javascript::LANGUAGE.into());
static TS_LANGUAGE: Lazy<tree_sitter::Language> = Lazy::new(|| tree_sitter_typescript::LANGUAGE_TYPESCRIPT.into());
static TSX_LANGUAGE: Lazy<tree_sitter::Language> = Lazy::new(|| tree_sitter_typescript::LANGUAGE_TSX.into());

/// JavaScript/TypeScript parser
pub struct JsParser {
    js_parser: Parser,
    ts_parser: Parser,
    tsx_parser: Parser,
}

impl JsParser {
    pub fn new() -> Result<Self> {
        let mut js_parser = Parser::new();
        js_parser
            .set_language(&*JS_LANGUAGE)
            .map_err(|e| Error::Analysis(format!("Failed to set JS language: {e}")))?;

        let mut ts_parser = Parser::new();
        ts_parser
            .set_language(&*TS_LANGUAGE)
            .map_err(|e| Error::Analysis(format!("Failed to set TS language: {e}")))?;

        let mut tsx_parser = Parser::new();
        tsx_parser
            .set_language(&*TSX_LANGUAGE)
            .map_err(|e| Error::Analysis(format!("Failed to set TSX language: {e}")))?;

        Ok(Self { js_parser, ts_parser, tsx_parser })
    }

    pub fn parse_file(&mut self, path: &Path) -> Result<FileMetrics> {
        let source = std::fs::read_to_string(path)?;
        let source_bytes = source.as_bytes();

        let ext = path.extension().and_then(|e| e.to_str()).unwrap_or("");
        let parser = match ext {
            "ts" => &mut self.ts_parser,
            "tsx" => &mut self.tsx_parser,
            "jsx" => &mut self.js_parser,
            _ => &mut self.js_parser,
        };

        let tree = parser.parse(&source, None).ok_or_else(|| Error::Parse {
            path: path.to_path_buf(),
            message: "Failed to parse JS/TS file".into(),
        })?;

        let root = tree.root_node();
        let mut classes: Vec<ClassMetrics> = Vec::new();
        let mut loose_functions: Vec<MethodMetrics> = Vec::new();

        self.collect_definitions(&root, source_bytes, path, &mut classes, &mut loose_functions);

        // Group loose functions into a synthetic "module" class if present
        if !loose_functions.is_empty() {
            let module_name = path
                .file_stem()
                .and_then(|s| s.to_str())
                .unwrap_or("module");

            let total_complexity: usize = loose_functions.iter().map(|m| m.complexity.0).sum();
            let total_lines: usize = loose_functions.iter().map(|m| m.lines.0).sum();

            classes.push(ClassMetrics {
                name: module_name.into(),
                fully_qualified_name: module_name.into(),
                file_path: path.to_string_lossy().to_string().into(),
                lines: Lines(total_lines),
                method_count: MethodCount(loose_functions.len()),
                property_count: 0,
                field_count: 0,
                complexity: Complexity(total_complexity),
                methods: loose_functions.into(),
                dependencies: Arc::new([]),
            });
        }

        Ok(FileMetrics {
            path: path.to_string_lossy().to_string().into(),
            lines: ComplexityCalculator::count_lines(&source),
            classes: classes.into(),
        })
    }

    fn collect_definitions(
        &self,
        node: &tree_sitter::Node,
        source: &[u8],
        path: &Path,
        classes: &mut Vec<ClassMetrics>,
        loose_functions: &mut Vec<MethodMetrics>,
    ) {
        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            match child.kind() {
                "class_declaration" | "class" => {
                    if let Some(c) = self.parse_class(&child, source, path) {
                        classes.push(c);
                    }
                }
                "function_declaration" | "generator_function_declaration" => {
                    if let Some(m) = self.parse_function(&child, source) {
                        loose_functions.push(m);
                    }
                }
                "lexical_declaration" | "variable_declaration" => {
                    // Arrow functions / const fn = () => {}
                    self.extract_arrow_functions(&child, source, loose_functions);
                }
                "export_statement" => {
                    // Recurse into exports
                    self.collect_definitions(&child, source, path, classes, loose_functions);
                }
                _ => {
                    // Recurse for nested structures
                    self.collect_definitions(&child, source, path, classes, loose_functions);
                }
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
                    "method_definition" => {
                        if let Some(m) = self.parse_method(&child, source) {
                            methods.push(m);
                        }
                    }
                    "public_field_definition" | "field_definition" => field_count += 1,
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
            property_count: field_count,
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

        let is_async = node.children(&mut node.walk()).any(|c| c.kind() == "async");
        let is_static = node.children(&mut node.walk()).any(|c| c.kind() == "static");

        Some(MethodMetrics {
            name: name.into(),
            lines,
            complexity,
            parameters,
            called_methods: Arc::new([]),
            accessed_fields: Arc::new([]),
            return_type: "unknown".into(),
            is_public: true,
            is_static,
            is_async,
            tokens: tokens.into_iter().map(|s| s.into()).collect(),
        })
    }

    fn parse_function(&self, node: &tree_sitter::Node, source: &[u8]) -> Option<MethodMetrics> {
        let name = node.child_by_field_name("name")?.utf8_text(source).ok()?;
        let text = node.utf8_text(source).ok()?;
        let lines = ComplexityCalculator::count_lines(text);
        let complexity = ComplexityCalculator::calculate_from_tree(node, source);
        let parameters = ComplexityCalculator::count_parameters(node, source);
        let tokens = ComplexityCalculator::extract_tokens(node, source);

        let is_async = node.children(&mut node.walk()).any(|c| c.kind() == "async");

        Some(MethodMetrics {
            name: name.into(),
            lines,
            complexity,
            parameters,
            called_methods: Arc::new([]),
            accessed_fields: Arc::new([]),
            return_type: "unknown".into(),
            is_public: true,
            is_static: false,
            is_async,
            tokens: tokens.into_iter().map(|s| s.into()).collect(),
        })
    }

    fn extract_arrow_functions(&self, node: &tree_sitter::Node, source: &[u8], functions: &mut Vec<MethodMetrics>) {
        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            if child.kind() == "variable_declarator" {
                let name = child.child_by_field_name("name").and_then(|n| n.utf8_text(source).ok());
                let value = child.child_by_field_name("value");

                if let (Some(name), Some(value)) = (name, value) {
                    if matches!(value.kind(), "arrow_function" | "function") {
                        let text = value.utf8_text(source).unwrap_or("");
                        let lines = ComplexityCalculator::count_lines(text);
                        let complexity = ComplexityCalculator::calculate_from_tree(&value, source);
                        let parameters = ComplexityCalculator::count_parameters(&value, source);
                        let tokens = ComplexityCalculator::extract_tokens(&value, source);
                        let is_async = value.children(&mut value.walk()).any(|c| c.kind() == "async");

                        functions.push(MethodMetrics {
                            name: name.into(),
                            lines,
                            complexity,
                            parameters,
                            called_methods: Arc::new([]),
                            accessed_fields: Arc::new([]),
                            return_type: "unknown".into(),
                            is_public: true,
                            is_static: false,
                            is_async,
                            tokens: tokens.into_iter().map(|s| s.into()).collect(),
                        });
                    }
                }
            }
        }
    }
}

impl Default for JsParser {
    fn default() -> Self {
        Self::new().expect("Failed to create JS/TS parser")
    }
}
