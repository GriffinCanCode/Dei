//! Multi-language parser dispatcher
//! 
//! Routes to appropriate language-specific parser

use dei_core::{error::Result, metrics::FileMetrics, models::Language, traits::Parser, Error};
use std::path::Path;

use crate::{csharp::CSharpParser, rust::RustParser};

/// Parser that supports multiple languages
pub struct MultiLanguageParser {
    rust_parser: RustParser,
    csharp_parser: CSharpParser,
}

impl MultiLanguageParser {
    pub fn new() -> Result<Self> {
        Ok(Self {
            rust_parser: RustParser::new()?,
            csharp_parser: CSharpParser::new()?,
        })
    }

    fn detect_language(&self, path: &Path) -> Option<Language> {
        path.extension()
            .and_then(|ext| ext.to_str())
            .and_then(Language::from_extension)
    }
}

impl Parser for MultiLanguageParser {
    fn parse_file(&self, path: &Path) -> Result<FileMetrics> {
        let language = self.detect_language(path).ok_or_else(|| {
            Error::UnsupportedLanguage(
                path.extension()
                    .and_then(|e| e.to_str())
                    .unwrap_or("unknown")
                    .to_string(),
            )
        })?;

        match language {
            Language::Rust => {
                // Create new parser for thread safety
                let mut parser = RustParser::new()?;
                parser.parse_file(path)
            }
            Language::CSharp => {
                let mut parser = CSharpParser::new()?;
                parser.parse_file(path)
            }
            _ => Err(Error::UnsupportedLanguage(format!("{:?}", language))),
        }
    }

    fn supported_languages(&self) -> &[Language] {
        &[Language::Rust, Language::CSharp]
    }
}

impl Default for MultiLanguageParser {
    fn default() -> Self {
        Self::new().expect("Failed to create multi-language parser")
    }
}

