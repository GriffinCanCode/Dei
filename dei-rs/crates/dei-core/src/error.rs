//! Error types for the dei analyzer

use std::path::PathBuf;
use thiserror::Error;

pub type Result<T, E = Error> = std::result::Result<T, E>;

#[derive(Debug, Error)]
pub enum Error {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Parse error in {path}: {message}")]
    Parse { path: PathBuf, message: String },

    #[error("Analysis error: {0}")]
    Analysis(String),

    #[error("Clustering error: {0}")]
    Clustering(String),

    #[error("Invalid configuration: {0}")]
    Config(String),

    #[error("Path not found: {0}")]
    PathNotFound(PathBuf),

    #[error("Unsupported language: {0}")]
    UnsupportedLanguage(String),
}

