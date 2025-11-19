//! Multi-language parsing support using tree-sitter
//! 
//! Major improvement over C# version - supports multiple languages from the start

pub mod rust;
pub mod csharp;
pub mod complexity;
pub mod multi_parser;

pub use complexity::ComplexityCalculator;
pub use multi_parser::MultiLanguageParser;


