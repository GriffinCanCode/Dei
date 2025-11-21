//! End-to-end testing utilities for dei
//! 
//! This crate provides comprehensive E2E tests that exercise the entire
//! analysis pipeline with real-world scenarios.

pub mod fixtures;
pub mod harness;

pub use fixtures::*;
pub use harness::*;

