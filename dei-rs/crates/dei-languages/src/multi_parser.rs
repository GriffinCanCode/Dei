//! Multi-language parser dispatcher
//! 
//! Routes to appropriate language-specific parser

use dei_core::{error::Result, metrics::FileMetrics, models::Language, traits::Parser, Error};
use std::path::Path;

use crate::{csharp::CSharpParser, java::JavaParser, javascript::JsParser, perl::PerlParser, python::PythonParser, r::RParser, rust::RustParser};

/// Parser that supports multiple languages (creates parsers on-demand for thread safety)
#[derive(Default)]
pub struct MultiLanguageParser;

impl MultiLanguageParser {
    pub fn new() -> Result<Self> {
        Ok(Self)
    }

    fn detect_language(path: &Path) -> Option<Language> {
        let ext = path.extension().and_then(|e| e.to_str())?;
        match ext {
            "jsx" => Some(Language::JavaScript),
            "tsx" => Some(Language::TypeScript),
            _ => Language::from_extension(ext),
        }
    }
}

impl Parser for MultiLanguageParser {
    fn parse_file(&self, path: &Path) -> Result<FileMetrics> {
        let ext = path.extension().and_then(|e| e.to_str()).unwrap_or("unknown");
        let language = Self::detect_language(path)
            .ok_or_else(|| Error::UnsupportedLanguage(ext.to_string()))?;

        match language {
            Language::Rust => RustParser::new()?.parse_file(path),
            Language::CSharp => CSharpParser::new()?.parse_file(path),
            Language::Python => PythonParser::new()?.parse_file(path),
            Language::JavaScript | Language::TypeScript => JsParser::new()?.parse_file(path),
            Language::Java => JavaParser::new()?.parse_file(path),
            Language::Perl => PerlParser::new()?.parse_file(path),
            Language::R => RParser::new()?.parse_file(path),
            _ => Err(Error::UnsupportedLanguage(format!("{language:?}"))),
        }
    }

    fn supported_languages(&self) -> &[Language] {
        &[Language::Rust, Language::CSharp, Language::Python, Language::JavaScript, Language::TypeScript, Language::Java, Language::Perl, Language::R]
    }
}

