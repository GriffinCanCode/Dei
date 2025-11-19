//! Core domain models and traits for the dei code analyzer
//! 
//! This crate provides language-agnostic abstractions for code analysis,
//! emphasizing zero-cost abstractions and strong typing.

pub mod error;
pub mod metrics;
pub mod models;
pub mod thresholds;
pub mod traits;

#[cfg(test)]
mod tests;

pub use error::{Error, Result};

