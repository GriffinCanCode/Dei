//! R language parser using tree-sitter

use dei_core::{error::Result, metrics::*, thresholds::*, Error};
use once_cell::sync::Lazy;
use std::path::Path;
use std::sync::Arc;
use tree_sitter::Parser;

use crate::complexity::ComplexityCalculator;

static R_LANGUAGE: Lazy<tree_sitter::Language> = Lazy::new(|| tree_sitter_r::LANGUAGE.into());

/// R language parser
pub struct RParser {
    parser: Parser,
}

impl RParser {
    pub fn new() -> Result<Self> {
        let mut parser = Parser::new();
        parser
            .set_language(&*R_LANGUAGE)
            .map_err(|e| Error::Analysis(format!("Failed to set R language: {e}")))?;
        Ok(Self { parser })
    }

    pub fn parse_file(&mut self, path: &Path) -> Result<FileMetrics> {
        let source = std::fs::read_to_string(path)?;
        let source_bytes = source.as_bytes();

        let tree = self.parser.parse(&source, None).ok_or_else(|| Error::Parse {
            path: path.to_path_buf(),
            message: "Failed to parse R file".into(),
        })?;

        let root = tree.root_node();
        let mut classes: Vec<ClassMetrics> = Vec::new();

        // R uses R6, S4, or Reference Classes for OOP - collect functions and class definitions
        self.collect_classes(&root, source_bytes, path, &mut classes);

        // If no classes found, treat top-level functions as a "module"
        if classes.is_empty() {
            let methods = self.collect_top_level_functions(&root, source_bytes);
            if !methods.is_empty() {
                let total_complexity: usize = methods.iter().map(|m| m.complexity.0).sum();
                let file_name = path.file_stem().and_then(|s| s.to_str()).unwrap_or("module");
                classes.push(ClassMetrics {
                    name: file_name.into(),
                    fully_qualified_name: file_name.into(),
                    file_path: path.to_string_lossy().to_string().into(),
                    lines: ComplexityCalculator::count_lines(&source),
                    method_count: MethodCount(methods.len()),
                    property_count: 0,
                    field_count: 0,
                    complexity: Complexity(total_complexity.max(1)),
                    methods: methods.into(),
                    dependencies: Arc::new([]),
                });
            }
        }

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
            // Look for R6Class, setRefClass, setClass definitions
            if child.kind() == "binary_operator" || child.kind() == "left_assignment" {
                if let Some(class) = self.try_parse_class_assignment(&child, source, path) {
                    classes.push(class);
                }
            }
            self.collect_classes(&child, source, path, classes);
        }
    }

    fn try_parse_class_assignment(
        &self,
        node: &tree_sitter::Node,
        source: &[u8],
        path: &Path,
    ) -> Option<ClassMetrics> {
        // Pattern: ClassName <- R6Class(...) or setRefClass(...)
        let lhs = node.child_by_field_name("lhs").or_else(|| node.child(0))?;
        let rhs = node.child_by_field_name("rhs").or_else(|| node.child(2))?;

        let name = lhs.utf8_text(source).ok()?;
        
        // Check if RHS is a class constructor call
        if rhs.kind() == "call" {
            let func = rhs.child_by_field_name("function").or_else(|| rhs.child(0))?;
            let func_name = func.utf8_text(source).ok()?;
            
            if matches!(func_name, "R6Class" | "setRefClass" | "setClass" | "structure") {
                let text = node.utf8_text(source).ok()?;
                let lines = ComplexityCalculator::count_lines(text);
                let methods = self.extract_class_methods(&rhs, source);
                let total_complexity: usize = methods.iter().map(|m| m.complexity.0).sum();

                return Some(ClassMetrics {
                    name: name.into(),
                    fully_qualified_name: name.into(),
                    file_path: path.to_string_lossy().to_string().into(),
                    lines,
                    method_count: MethodCount(methods.len()),
                    property_count: 0,
                    field_count: self.count_class_fields(&rhs, source),
                    complexity: Complexity(total_complexity.max(1)),
                    methods: methods.into(),
                    dependencies: Arc::new([]),
                });
            }
        }
        None
    }

    fn extract_class_methods(&self, call_node: &tree_sitter::Node, source: &[u8]) -> Vec<MethodMetrics> {
        let mut methods = Vec::new();
        
        // R6Class methods are in public/private lists
        if let Some(args) = call_node.child_by_field_name("arguments") {
            let mut cursor = args.walk();
            for arg in args.children(&mut cursor) {
                if arg.kind() == "argument" {
                    if let Some(name) = arg.child_by_field_name("name") {
                        if let Ok(arg_name) = name.utf8_text(source) {
                            if matches!(arg_name, "public" | "private" | "active") {
                                if let Some(value) = arg.child_by_field_name("value") {
                                    self.extract_methods_from_list(&value, source, &mut methods);
                                }
                            }
                        }
                    }
                }
            }
        }
        methods
    }

    fn extract_methods_from_list(
        &self,
        node: &tree_sitter::Node,
        source: &[u8],
        methods: &mut Vec<MethodMetrics>,
    ) {
        // list(method1 = function(...) {...}, method2 = ...)
        if node.kind() == "call" {
            if let Some(args) = node.child_by_field_name("arguments") {
                let mut cursor = args.walk();
                for arg in args.children(&mut cursor) {
                    if arg.kind() == "argument" {
                        if let (Some(name), Some(value)) = (
                            arg.child_by_field_name("name"),
                            arg.child_by_field_name("value"),
                        ) {
                            if value.kind() == "function_definition" {
                                if let Some(m) = self.parse_function(&value, source, Some(name)) {
                                    methods.push(m);
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    fn count_class_fields(&self, call_node: &tree_sitter::Node, _source: &[u8]) -> usize {
        let mut count = 0;
        if let Some(args) = call_node.child_by_field_name("arguments") {
            let mut cursor = args.walk();
            for arg in args.children(&mut cursor) {
                if arg.kind() == "argument" {
                    if let Some(value) = arg.child_by_field_name("value") {
                        count += self.count_non_function_fields(&value);
                    }
                }
            }
        }
        count
    }

    fn count_non_function_fields(&self, node: &tree_sitter::Node) -> usize {
        let mut count = 0;
        if node.kind() == "call" {
            if let Some(args) = node.child_by_field_name("arguments") {
                let mut cursor = args.walk();
                for arg in args.children(&mut cursor) {
                    if arg.kind() == "argument" {
                        if let Some(value) = arg.child_by_field_name("value") {
                            if value.kind() != "function_definition" {
                                count += 1;
                            }
                        }
                    }
                }
            }
        }
        count
    }

    fn collect_top_level_functions(&self, root: &tree_sitter::Node, source: &[u8]) -> Vec<MethodMetrics> {
        let mut methods = Vec::new();
        let mut cursor = root.walk();
        
        for child in root.children(&mut cursor) {
            // name <- function(...) or name = function(...)
            if child.kind() == "binary_operator" || child.kind() == "left_assignment" {
                if let Some(m) = self.try_parse_function_assignment(&child, source) {
                    methods.push(m);
                }
            }
        }
        methods
    }

    fn try_parse_function_assignment(
        &self,
        node: &tree_sitter::Node,
        source: &[u8],
    ) -> Option<MethodMetrics> {
        let lhs = node.child_by_field_name("lhs").or_else(|| node.child(0))?;
        let rhs = node.child_by_field_name("rhs").or_else(|| node.child(2))?;

        if rhs.kind() == "function_definition" {
            self.parse_function(&rhs, source, Some(lhs))
        } else {
            None
        }
    }

    fn parse_function(
        &self,
        node: &tree_sitter::Node,
        source: &[u8],
        name_node: Option<tree_sitter::Node>,
    ) -> Option<MethodMetrics> {
        let name = name_node
            .and_then(|n| n.utf8_text(source).ok())
            .unwrap_or("anonymous");

        let text = node.utf8_text(source).ok()?;
        let lines = ComplexityCalculator::count_lines(text);
        let complexity = self.calculate_r_complexity(node);
        let parameters = self.count_r_parameters(node, source);
        let tokens = ComplexityCalculator::extract_tokens(node, source);

        Some(MethodMetrics {
            name: name.into(),
            lines,
            complexity: Complexity(complexity),
            parameters: ParamCount(parameters),
            called_methods: Arc::new([]),
            accessed_fields: Arc::new([]),
            return_type: "unknown".into(),
            is_public: !name.starts_with('.'), // R convention: .name is private
            is_static: false,
            is_async: false,
            tokens: tokens.into_iter().map(|s| s.into()).collect(),
        })
    }

    fn calculate_r_complexity(&self, node: &tree_sitter::Node) -> usize {
        self.count_complexity_nodes(node)
    }

    fn count_complexity_nodes(&self, node: &tree_sitter::Node) -> usize {
        let mut complexity = match node.kind() {
            "function_definition" => 1,
            "if_statement" | "else_clause" => 1,
            "for_statement" | "while_statement" | "repeat_statement" => 1,
            "binary_operator" => {
                let mut cursor = node.walk();
                node.children(&mut cursor)
                    .filter(|c| matches!(c.kind(), "&&" | "||" | "&" | "|"))
                    .count()
            }
            "call" => {
                node.child(0)
                    .and_then(|f| f.utf8_text(&[]).ok())
                    .filter(|&n| n == "tryCatch")
                    .map_or(0, |_| 1)
            }
            _ => 0,
        };

        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            complexity += self.count_complexity_nodes(&child);
        }
        complexity
    }

    fn count_r_parameters(&self, node: &tree_sitter::Node, source: &[u8]) -> usize {
        let params_node = node.child_by_field_name("parameters").or_else(|| {
            let mut cursor = node.walk();
            let result = node.children(&mut cursor).find(|c| c.kind() == "formal_parameters");
            result
        });
        
        params_node.map_or(0, |params| {
            let mut cursor = params.walk();
            params.children(&mut cursor)
                .filter(|c| matches!(c.kind(), "identifier" | "default_parameter" | "parameter"))
                .filter(|c| c.utf8_text(source).ok() != Some("..."))
                .count()
        })
    }
}

impl Default for RParser {
    fn default() -> Self {
        Self::new().expect("Failed to create R parser")
    }
}
