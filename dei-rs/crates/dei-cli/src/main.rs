//! dei - Code analysis CLI
//! 
//! Beautiful, fast, and extensible

mod commands;
mod report;

use anyhow::Result;
use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "dei")]
#[command(version, about = "Detect god classes and code smells", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Check a directory for god classes
    Check {
        /// Path to analyze
        path: std::path::PathBuf,
        
        /// Maximum class lines
        #[arg(long, default_value = "300")]
        max_lines: usize,
        
        /// Maximum methods per class
        #[arg(long, default_value = "20")]
        max_methods: usize,
        
        /// Maximum cyclomatic complexity
        #[arg(long, default_value = "50")]
        max_complexity: usize,
        
        /// Output format (text, json)
        #[arg(long, default_value = "text")]
        format: String,
        
        /// Show detailed analysis
        #[arg(long, short)]
        verbose: bool,
    },
    
    /// Analyze architecture quality
    Arch {
        /// Path to analyze
        path: std::path::PathBuf,
    },
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Check {
            path,
            max_lines,
            max_methods,
            max_complexity,
            format,
            verbose,
        } => {
            commands::check::run(
                path,
                max_lines,
                max_methods,
                max_complexity,
                format,
                verbose,
            )
            .await?;
        }
        Commands::Arch { path } => {
            commands::arch::run(path).await?;
        }
    }

    Ok(())
}

