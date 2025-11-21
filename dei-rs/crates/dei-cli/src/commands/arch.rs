//! Architecture analysis command

use anyhow::Result;
use colored::Colorize;
use dei_ast::{AstBuilder, ParallelTraverser};
use dei_core::thresholds::Thresholds;
use dei_languages::MultiLanguageParser;
use dei_metrics::CouplingAnalyzer;
use std::path::PathBuf;

pub async fn run(path: PathBuf) -> Result<()> {
    println!("{}", "╔════════════════════════════════════════════════════════════╗".bright_cyan());
    println!("{}", "║         DEI - ARCHITECTURE QUALITY ANALYSIS                ║".bright_cyan());
    println!("{}", "╚════════════════════════════════════════════════════════════╝".bright_cyan());
    println!();

    // Build AST and analyze
    let builder = AstBuilder::new();
    let root_id = builder.build(&path)?;

    let parser = MultiLanguageParser::new()?;
    let traverser = ParallelTraverser::new(parser, builder.arena().clone());
    let thresholds = Thresholds::default();
    traverser.traverse_and_analyze(root_id, &thresholds)?;

    let all_results = traverser.all_results();

    // Extract all classes
    let classes: Vec<_> = all_results
        .iter()
        .map(|r| r.class_metrics.clone())
        .collect();

    // Build coupling graph
    let mut coupling_analyzer = CouplingAnalyzer::new();
    coupling_analyzer.build_graph(&classes);

    let metrics = coupling_analyzer.architecture_quality();

    println!("{}", "ARCHITECTURE METRICS:".bright_green().bold());
    println!();
    println!("  {} {:.2}%", "Graph Density:".bold(), metrics.density * 100.0);
    println!("  {} {}", "Circular Dependencies:".bold(), metrics.n_cycles);
    println!("  {} {:.2}", "Cyclomatic Quality:".bold(), metrics.cyclomatic_quality);
    println!("  {} {:.2}", "Maintainability Index:".bold(), metrics.maintainability_index);
    println!();

    if metrics.n_cycles > 0 {
        println!("{}", "⚠️  CIRCULAR DEPENDENCIES DETECTED:".yellow().bold());
        println!();
        
        for cycle in coupling_analyzer.find_tight_coupling() {
            println!("  🔄 {}", cycle.join(" → ").red());
        }
        println!();
    }

    // Quality assessment
    let quality = if metrics.maintainability_index > 0.8 {
        "Excellent".green()
    } else if metrics.maintainability_index > 0.6 {
        "Good".bright_green()
    } else if metrics.maintainability_index > 0.4 {
        "Fair".yellow()
    } else {
        "Poor".red()
    };

    println!("{} {}", "Overall Quality:".bold(), quality.bold());
    println!();

    Ok(())
}

