//! Beautiful report generation

use colored::Colorize;
use dei_core::{models::*, thresholds::Thresholds};

pub struct ReportGenerator {
    thresholds: Thresholds,
}

impl ReportGenerator {
    pub fn new(thresholds: Thresholds) -> Self {
        Self { thresholds }
    }

    pub fn print_text_report(&self, results: &[AnalysisResult], verbose: bool) {
        let total_classes = results.len();
        let god_classes: Vec<_> = results.iter().filter(|r| r.is_god_class).collect();
        let classes_with_god_methods: Vec<_> = results
            .iter()
            .filter(|r| !r.god_methods.is_empty())
            .collect();
        let healthy_classes = results.iter().filter(|r| !r.has_issues()).count();

        // Summary
        println!("{}", "SUMMARY:".bright_green().bold());
        println!();
        println!("  {} {}", "Total Classes:".bold(), total_classes);
        println!("  {} {}", "God Classes:".bold(), god_classes.len().to_string().red());
        println!("  {} {}", "Classes with God Methods:".bold(), classes_with_god_methods.len().to_string().yellow());
        println!("  {} {}", "Healthy Classes:".bold(), healthy_classes.to_string().green());
        println!();

        // God classes
        if !god_classes.is_empty() {
            println!("{}", "⚠️  GOD CLASSES DETECTED:".red().bold());
            println!();

            for result in &god_classes {
                let metrics = &result.class_metrics;
                println!("  {} {}", "❌".red(), metrics.name.bright_red().bold());
                println!("     File: {}", metrics.file_path);
                println!("     Lines: {} | Methods: {} | Complexity: {}",
                    metrics.lines.0.to_string().yellow(),
                    metrics.method_count.0.to_string().yellow(),
                    metrics.complexity.0.to_string().yellow()
                );

                if !result.suggested_extractions.is_empty() {
                    println!("     {} {}", "Suggested Extractions:".cyan(), result.suggested_extractions.len());
                    
                    if verbose {
                        for cluster in result.suggested_extractions.iter() {
                            println!("       → {} (cohesion: {:.2})",
                                cluster.suggested_name.bright_cyan(),
                                cluster.cohesion_score
                            );
                            println!("         Methods: {}", cluster.methods.len());
                        }
                    }
                }

                if !result.god_methods.is_empty() {
                    println!("     {} {}", "God Methods:".yellow(), result.god_methods.len());
                }

                println!();
            }
        }

        // Classes with god methods
        if !classes_with_god_methods.is_empty() {
            println!("{}", "⚠️  GOD METHODS DETECTED:".yellow().bold());
            println!();

            for result in &classes_with_god_methods {
                if result.is_god_class {
                    continue; // Already shown above
                }

                let metrics = &result.class_metrics;
                println!("  {} {}", "📝".yellow(), metrics.name.bright_yellow());
                println!("     File: {}", metrics.file_path);
                println!("     {} {}", "God Methods:".bold(), result.god_methods.len());
                println!();

                if verbose {
                    for god_method in result.god_methods.iter() {
                        println!("       ⚠️  {}", god_method.method_name.yellow());
                        println!("          Lines: {} | Complexity: {} | Parameters: {}",
                            god_method.metrics.lines.0,
                            god_method.metrics.complexity.0,
                            god_method.metrics.parameters.0
                        );

                        for violation in god_method.violations.iter() {
                            println!("          • {:?}: {} exceeds {}",
                                violation.kind,
                                violation.actual.to_string().red(),
                                violation.threshold.to_string().green()
                            );
                        }
                        println!();
                    }
                }
            }
        }

        // Success message
        if god_classes.is_empty() && classes_with_god_methods.is_empty() {
            println!("{}", "✅ No god classes or methods detected!".green().bold());
            println!("{}", "   Your code is well-structured.".green());
            println!();
        }
    }
}

