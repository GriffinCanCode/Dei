//! Perl language parser using tree-sitter

use dei_core::{error::Result, metrics::*, thresholds::*, Error};
use once_cell::sync::Lazy;
use std::path::Path;
use std::sync::Arc;
use tree_sitter::Parser;

use crate::complexity::ComplexityCalculator;

static PERL_LANGUAGE: Lazy<tree_sitter::Language> = Lazy::new(|| tree_sitter_perl::LANGUAGE.into());

/// Perl-specific parser
pub struct PerlParser {
    parser: Parser,
}

impl PerlParser {
    pub fn new() -> Result<Self> {
        let mut parser = Parser::new();
        parser
            .set_language(&*PERL_LANGUAGE)
            .map_err(|e| Error::Analysis(format!("Failed to set Perl language: {}", e)))?;
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
                message: "Failed to parse Perl file".into(),
            })?;

        let root = tree.root_node();
        let mut packages = std::collections::HashMap::new();
        let mut standalone_subs = Vec::new();

        self.traverse_nodes(&root, source_bytes, path, &mut packages, &mut standalone_subs);

        // If no packages found but subs exist, create synthetic class from file
        let classes: Vec<ClassMetrics> = if packages.is_empty() && !standalone_subs.is_empty() {
            let total_complexity = standalone_subs.iter().map(|m| m.complexity.0).sum::<usize>();
            vec![ClassMetrics {
                name: path.file_stem().map(|s| s.to_string_lossy().into()).unwrap_or("main".into()),
                fully_qualified_name: "main".into(),
                file_path: path.to_string_lossy().to_string().into(),
                lines: ComplexityCalculator::count_lines(&source),
                method_count: MethodCount(standalone_subs.len()),
                property_count: 0,
                field_count: 0,
                complexity: Complexity(total_complexity.max(1)),
                methods: standalone_subs.into(),
                dependencies: Arc::new([]),
            }]
        } else {
            packages.into_values().collect()
        };

        Ok(FileMetrics {
            path: path.to_string_lossy().to_string().into(),
            lines: ComplexityCalculator::count_lines(&source),
            classes: classes.into(),
        })
    }

    fn traverse_nodes(
        &self,
        node: &tree_sitter::Node,
        source: &[u8],
        path: &Path,
        packages: &mut std::collections::HashMap<String, ClassMetrics>,
        standalone_subs: &mut Vec<MethodMetrics>,
    ) {
        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            match child.kind() {
                "package_statement" => {
                    if let Some(pkg) = self.parse_package(&child, source, path) {
                        packages.insert(pkg.name.to_string(), pkg);
                    }
                }
                "subroutine_declaration_statement" | "anonymous_subroutine_expression" => {
                    if let Some(method) = self.parse_subroutine(&child, source) {
                        standalone_subs.push(method);
                    }
                }
                _ => self.traverse_nodes(&child, source, path, packages, standalone_subs),
            }
        }
    }

    fn parse_package(
        &self,
        node: &tree_sitter::Node,
        source: &[u8],
        path: &Path,
    ) -> Option<ClassMetrics> {
        let name = self.find_package_name(node, source)?;
        let text = node.utf8_text(source).ok()?;
        let lines = ComplexityCalculator::count_lines(text);

        let mut methods = Vec::new();
        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            if child.kind() == "subroutine_declaration_statement" {
                if let Some(m) = self.parse_subroutine(&child, source) {
                    methods.push(m);
                }
            }
        }

        let total_complexity = methods.iter().map(|m| m.complexity.0).sum::<usize>();

        Some(ClassMetrics {
            name: name.clone().into(),
            fully_qualified_name: name.into(),
            file_path: path.to_string_lossy().to_string().into(),
            lines,
            method_count: MethodCount(methods.len()),
            property_count: 0,
            field_count: 0,
            complexity: Complexity(total_complexity.max(1)),
            methods: methods.into(),
            dependencies: Arc::new([]),
        })
    }

    fn parse_subroutine(&self, node: &tree_sitter::Node, source: &[u8]) -> Option<MethodMetrics> {
        let name = self.find_sub_name(node, source).unwrap_or_else(|| "anonymous".into());
        let text = node.utf8_text(source).ok()?;
        let lines = ComplexityCalculator::count_lines(text);
        let complexity = self.calculate_perl_complexity(node);
        let parameters = self.count_perl_parameters(node, source);
        let is_public = !name.starts_with('_');

        Some(MethodMetrics {
            name: name.into(),
            lines,
            complexity,
            parameters: ParamCount(parameters),
            called_methods: Arc::new([]),
            accessed_fields: Arc::new([]),
            return_type: "scalar".into(),
            is_public,
            is_static: false,
            is_async: false,
            tokens: ComplexityCalculator::extract_tokens(node, source).into_iter().map(|s| s.into()).collect(),
        })
    }

    fn calculate_perl_complexity(&self, node: &tree_sitter::Node) -> Complexity {
        Complexity(self.count_complexity_nodes(node))
    }

    fn count_complexity_nodes(&self, node: &tree_sitter::Node) -> usize {
        let complexity = match node.kind() {
            "subroutine_declaration_statement" | "anonymous_subroutine_expression" => 1,
            "if_statement" | "elsif_clause" | "unless_statement" => 1,
            "for_statement" | "foreach_statement" | "while_statement" | "until_statement" => 1,
            "conditional_expression" => 1,
            "and" | "or" => 1,
            "given_statement" | "when_clause" => 1,
            _ => 0,
        };

        let mut total = complexity;
        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            total += self.count_complexity_nodes(&child);
        }
        total
    }

    fn count_perl_parameters(&self, node: &tree_sitter::Node, source: &[u8]) -> usize {
        // Look for prototype or signature
        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            if child.kind() == "prototype" || child.kind() == "signature" {
                if let Ok(text) = child.utf8_text(source) {
                    return text.matches(',').count() + 1;
                }
            }
        }
        0
    }

    fn find_package_name(&self, node: &tree_sitter::Node, source: &[u8]) -> Option<String> {
        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            if child.kind() == "package_name" || child.kind() == "identifier" {
                return child.utf8_text(source).ok().map(|s| s.to_string());
            }
            if let Some(name) = self.find_package_name(&child, source) {
                return Some(name);
            }
        }
        None
    }

    fn find_sub_name(&self, node: &tree_sitter::Node, source: &[u8]) -> Option<String> {
        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            if child.kind() == "identifier" || child.kind() == "bareword" {
                return child.utf8_text(source).ok().map(|s| s.to_string());
            }
        }
        None
    }
}

impl Default for PerlParser {
    fn default() -> Self {
        Self::new().expect("Failed to create Perl parser")
    }
}
