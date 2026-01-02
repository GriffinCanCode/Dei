//! Python language parser using tree-sitter

use dei_core::{error::Result, metrics::*, thresholds::*, Error};
use once_cell::sync::Lazy;
use std::path::Path;
use std::sync::Arc;
use tree_sitter::Parser;

use crate::complexity::ComplexityCalculator;

static PYTHON_LANGUAGE: Lazy<tree_sitter::Language> = Lazy::new(|| tree_sitter_python::LANGUAGE.into());

/// Python-specific parser
pub struct PythonParser {
    parser: Parser,
}

impl PythonParser {
    pub fn new() -> Result<Self> {
        let mut parser = Parser::new();
        parser
            .set_language(&*PYTHON_LANGUAGE)
            .map_err(|e| Error::Analysis(format!("Failed to set Python language: {}", e)))?;
        
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
                message: "Failed to parse Python file".into(),
            })?;

        let root = tree.root_node();
        let mut classes = Vec::new();

        // Find all class definitions
        self.find_classes(&root, source_bytes, path, &mut classes);

        let lines = ComplexityCalculator::count_lines(&source);

        Ok(FileMetrics {
            path: path.to_string_lossy().to_string().into(),
            lines,
            classes: classes.into(),
        })
    }

    fn find_classes(
        &self,
        node: &tree_sitter::Node,
        source: &[u8],
        path: &Path,
        classes: &mut Vec<ClassMetrics>,
    ) {
        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            if child.kind() == "class_definition" {
                if let Some(class_metrics) = self.parse_class(&child, source, path) {
                    classes.push(class_metrics);
                }
            } else if child.named_child_count() > 0 {
                self.find_classes(&child, source, path, classes);
            }
        }
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
        let mut field_count = 0;

        // Find the class body
        if let Some(body) = node.child_by_field_name("body") {
            let mut cursor = body.walk();
            for child in body.children(&mut cursor) {
                match child.kind() {
                    "function_definition" => {
                        if let Some(method) = self.parse_method(&child, source) {
                            // Count __init__ assignments as fields
                            if method.name.as_ref() == "__init__" {
                                field_count += self.count_init_fields(&child, source);
                            }
                            methods.push(method);
                        }
                    }
                    "expression_statement" => {
                        // Class-level attribute assignments
                        if self.is_class_attribute(&child) {
                            field_count += 1;
                        }
                    }
                    _ => {}
                }
            }
        }

        let total_complexity = methods.iter().map(|m| m.complexity.0).sum::<usize>();

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
        let complexity = self.calculate_python_complexity(node, source);
        let parameters = self.count_python_parameters(node, source);

        let return_type = node
            .child_by_field_name("return_type")
            .and_then(|n| n.utf8_text(source).ok())
            .unwrap_or("None");

        // Check for decorators and method visibility
        let is_public = !name.starts_with('_') || name.starts_with("__") && name.ends_with("__");
        let is_static = self.has_decorator(node, source, "staticmethod");
        let is_async = node.kind() == "function_definition" 
            && node.children(&mut node.walk()).any(|c| c.kind() == "async");

        let tokens = ComplexityCalculator::extract_tokens(node, source);

        Some(MethodMetrics {
            name: name.into(),
            lines,
            complexity,
            parameters: ParamCount(parameters),
            called_methods: Arc::new([]),
            accessed_fields: Arc::new([]),
            return_type: return_type.into(),
            is_public,
            is_static,
            is_async,
            tokens: tokens.into_iter().map(|s| s.into()).collect(),
        })
    }

    fn calculate_python_complexity(&self, node: &tree_sitter::Node, _source: &[u8]) -> Complexity {
        Complexity(self.count_complexity_nodes(node))
    }

    fn count_complexity_nodes(&self, node: &tree_sitter::Node) -> usize {
        let mut complexity = match node.kind() {
            "function_definition" => 1, // Base complexity for functions
            "if_statement" | "elif_clause" | "for_statement" | "while_statement" => 1,
            "try_statement" | "except_clause" => 1,
            "and" | "or" => 1,
            "list_comprehension" | "dictionary_comprehension" | "set_comprehension" 
            | "generator_expression" => 1,
            "lambda" => 1,
            "match_statement" | "case_clause" => 1,
            _ => 0,
        };

        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            complexity += self.count_complexity_nodes(&child);
        }
        
        complexity
    }

    fn count_python_parameters(&self, node: &tree_sitter::Node, source: &[u8]) -> usize {
        node.child_by_field_name("parameters")
            .map(|params| {
                let mut count = 0;
                let mut cursor = params.walk();
                for child in params.children(&mut cursor) {
                    match child.kind() {
                        "identifier" | "typed_parameter" | "default_parameter" 
                        | "typed_default_parameter" | "list_splat_pattern" 
                        | "dictionary_splat_pattern" => {
                            // Skip 'self' and 'cls' parameters
                            if let Ok(text) = child.utf8_text(source) {
                                if text != "self" && text != "cls" && !text.starts_with("self:") {
                                    count += 1;
                                }
                            } else {
                                count += 1;
                            }
                        }
                        _ => {}
                    }
                }
                count
            })
            .unwrap_or(0)
    }

    fn count_init_fields(&self, node: &tree_sitter::Node, source: &[u8]) -> usize {
        let mut count = 0;
        if let Some(body) = node.child_by_field_name("body") {
            let mut cursor = body.walk();
            for child in body.children(&mut cursor) {
                if child.kind() == "expression_statement" {
                    if let Some(assignment) = child.child(0) {
                        if assignment.kind() == "assignment" {
                            if let Some(left) = assignment.child_by_field_name("left") {
                                if left.kind() == "attribute" {
                                    if let Ok(text) = left.utf8_text(source) {
                                        if text.starts_with("self.") {
                                            count += 1;
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
        count
    }

    fn is_class_attribute(&self, node: &tree_sitter::Node) -> bool {
        if let Some(child) = node.child(0) {
            child.kind() == "assignment"
        } else {
            false
        }
    }

    fn has_decorator(&self, node: &tree_sitter::Node, source: &[u8], decorator_name: &str) -> bool {
        let parent = node.parent();
        if let Some(p) = parent {
            if p.kind() == "decorated_definition" {
                let mut cursor = p.walk();
                for child in p.children(&mut cursor) {
                    if child.kind() == "decorator" {
                        if let Ok(text) = child.utf8_text(source) {
                            if text.contains(decorator_name) {
                                return true;
                            }
                        }
                    }
                }
            }
        }
        false
    }
}

impl Default for PythonParser {
    fn default() -> Self {
        Self::new().expect("Failed to create Python parser")
    }
}
