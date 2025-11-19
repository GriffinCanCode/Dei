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
    println!("{}", "╔════════════════════════════════════════════════════════════╗".bright_cyan());
    println!("{}", "║           DEI - CODE ANALYSIS (Rust Edition)               ║".bright_cyan());
    println!("{}", "╚════════════════════════════════════════════════════════════╝".bright_cyan());
    println!();

    // Setup thresholds
    let thresholds = Thresholds {
        max_class_lines: Lines(max_lines),
        max_methods: MethodCount(max_methods),
        max_class_complexity: Complexity(max_complexity),
        ..Default::default()
    };

    thresholds.validate()?;

    println!("📂 Analyzing: {}", path.display().to_string().bright_yellow());
    println!();

    // Build AST
    let spinner = ProgressBar::new_spinner();
    spinner.set_style(
        ProgressStyle::default_spinner()
            .template("{spinner:.cyan} {msg}")
            .unwrap(),
    );
    spinner.set_message("Building filesystem AST...");

    let builder = AstBuilder::new();
    let root_id = builder.build(&path)?;
    spinner.finish_with_message("✓ AST built".green().to_string());

    // Parse and analyze
    let spinner = ProgressBar::new_spinner();
    spinner.set_message("Analyzing files in parallel...");

    let parser = MultiLanguageParser::new()?;
    let traverser = ParallelTraverser::new(parser, builder.arena().clone());
    traverser.traverse_and_analyze(root_id, &thresholds)?;

    spinner.finish_with_message("✓ Analysis complete".green().to_string());
    println!();

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

