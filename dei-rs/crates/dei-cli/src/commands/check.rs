//! Check command - main analysis entry point

use anyhow::Result;
use colored::Colorize;
use dei_ast::{AstBuilder, ParallelTraverser};
use dei_clustering::ClusteringAnalyzer;
use dei_core::{
    thresholds::{Complexity, Lines, MethodCount, Thresholds},
    traits::ClusterAnalyzer,
};
use dei_languages::MultiLanguageParser;
use indicatif::{ProgressBar, ProgressStyle};
use std::path::PathBuf;

use crate::report::ReportGenerator;

pub async fn run(
    path: PathBuf,
    max_lines: usize,
    max_methods: usize,
    max_complexity: usize,
    format: String,
    verbose: bool,
) -> Result<()> {
    let is_json = format == "json";

    if !is_json {
        println!("{}", "╔════════════════════════════════════════════════════════════╗".bright_cyan());
        println!("{}", "║           DEI - CODE ANALYSIS (Rust Edition)               ║".bright_cyan());
        println!("{}", "╚════════════════════════════════════════════════════════════╝".bright_cyan());
        println!();
    }

    // Setup thresholds
    let thresholds = Thresholds {
        max_class_lines: Lines(max_lines),
        max_methods: MethodCount(max_methods),
        max_class_complexity: Complexity(max_complexity),
        ..Default::default()
    };

    thresholds.validate().map_err(|e| anyhow::anyhow!(e))?;

    if !is_json {
        println!("📂 Analyzing: {}", path.display().to_string().bright_yellow());
        println!();
    }

    // Build AST
    let spinner = if !is_json {
        let s = ProgressBar::new_spinner();
        s.set_style(
            ProgressStyle::default_spinner()
                .template("{spinner:.cyan} {msg}")
                .unwrap(),
        );
        s.set_message("Building filesystem AST...");
        Some(s)
    } else {
        None
    };

    let builder = AstBuilder::new();
    let root_id = builder.build(&path)?;
    
    if let Some(s) = spinner {
        s.finish_and_clear();
        println!("{}", "✓ AST built".green());
    }

    // Parse and analyze
    let spinner = if !is_json {
        let s = ProgressBar::new_spinner();
        s.set_message("Analyzing files in parallel...");
        Some(s)
    } else {
        None
    };

    let parser = MultiLanguageParser::new()?;
    let traverser = ParallelTraverser::new(parser, builder.arena().clone());
    traverser.traverse_and_analyze(root_id, &thresholds)?;

    if let Some(s) = spinner {
        s.finish_and_clear();
        println!("{}", "✓ Analysis complete".green());
    }

    if !is_json {
        println!();
    }

    // Get results
    let all_results = traverser.all_results();

    // Generate report
    let generator = ReportGenerator::new(thresholds);
    
    match format.as_str() {
        "json" => {
            let json = serde_json::to_string_pretty(&all_results)?;
            println!("{}", json);
        }
        _ => {
            generator.print_text_report(&all_results, verbose);
        }
    }

    // Exit with appropriate code
    let has_issues = all_results.iter().any(|r| r.has_issues());
    std::process::exit(if has_issues { 1 } else { 0 });
}

