//! Cyclomatic complexity and metrics calculation
//! 
//! Improved algorithm using tree-sitter for accurate AST-based analysis

use dei_core::{metrics::*, thresholds::*};
use tree_sitter::Node;

/// Calculate complexity from tree-sitter AST
pub struct ComplexityCalculator;

impl ComplexityCalculator {
    /// Calculate cyclomatic complexity using tree-sitter nodes
    /// More accurate than regex-based approaches
    pub fn calculate_from_tree(node: &Node, source: &[u8]) -> Complexity {
        let mut complexity = 1; // Base complexity

        let mut visit_stack = vec![node.clone()];

        while let Some(current) = visit_stack.pop() {
            match current.kind() {
                // Decision points
                "if_expression" | "if_statement" => complexity += 1,
                "while_statement" | "while_expression" => complexity += 1,
                "for_statement" | "for_expression" => complexity += 1,
                "match_expression" | "switch_statement" => complexity += 1,
                "catch_clause" | "catch" => complexity += 1,
                "conditional_expression" => complexity += 1,
                
                // Logical operators (each adds a path)
                "binary_expression" => {
                    if let Some(op) = current.child_by_field_name("operator") {
                        let op_text = op.utf8_text(source).unwrap_or("");
                        if op_text == "&&" || op_text == "||" {
                            complexity += 1;
                        }
                    }
                }
                
                // Match arms in Rust
                "match_arm" => complexity += 1,
                
                // Case labels in C#/Java/etc
                "switch_label" | "case" => complexity += 1,
                
                _ => {}
            }

            // Add children to stack
            for i in 0..current.child_count() {
                if let Some(child) = current.child(i) {
                    visit_stack.push(child);
                }
            }
        }

        Complexity(complexity)
    }

    /// Count non-blank, non-comment lines
    pub fn count_lines(source: &str) -> Lines {
        let count = source
            .lines()
            .filter(|line| {
                let trimmed = line.trim();
                !trimmed.is_empty() 
                    && !trimmed.starts_with("//")
                    && !trimmed.starts_with("#")
                    && !trimmed.starts_with("/*")
                    && !trimmed.starts_with("*")
            })
            .count();
        
        Lines(count)
    }

    /// Extract parameter count from function node
    pub fn count_parameters(node: &Node, _source: &[u8]) -> ParamCount {
        let mut count = 0;

        if let Some(params) = node.child_by_field_name("parameters") {
            // Count direct parameter children
            let mut cursor = params.walk();
            for child in params.children(&mut cursor) {
                match child.kind() {
                    "parameter" | "parameter_declaration" | "identifier" => count += 1,
                    _ => {}
                }
            }
        }

        ParamCount(count)
    }

    /// Extract tokens for semantic analysis
    /// Uses tree-sitter to get accurate identifiers
    pub fn extract_tokens(node: &Node, source: &[u8]) -> Vec<String> {
        let mut tokens = Vec::new();
        let mut cursor = node.walk();

        for child in node.children(&mut cursor) {
            match child.kind() {
                "identifier" | "type_identifier" | "field_identifier" => {
                    if let Ok(text) = child.utf8_text(source) {
                        // Split camelCase/PascalCase
                        tokens.extend(split_identifier(text));
                    }
                }
                _ => {
                    // Recurse
                    tokens.extend(Self::extract_tokens(&child, source));
                }
            }
        }

        tokens
    }
}

/// Split camelCase and PascalCase identifiers
fn split_identifier(s: &str) -> Vec<String> {
    let re = regex::Regex::new(r"([a-z0-9])([A-Z])").unwrap();
    let with_spaces = re.replace_all(s, "$1 $2");
    
    with_spaces
        .split(|c: char| !c.is_alphanumeric())
        .filter(|s| s.len() > 2)
        .map(|s| s.to_lowercase())
        .collect()
}

impl dei_core::traits::ComplexityCalculator for ComplexityCalculator {
    fn calculate_complexity(&self, source: &str) -> Complexity {
        // Fallback for when we don't have parsed tree - estimate from lines
        let lines = Self::count_lines(source);
        Complexity(lines.0 / 10 + 1) // Rough estimate: 1 complexity per 10 lines
    }

    fn count_lines(&self, source: &str) -> Lines {
        Self::count_lines(source)
    }
}

