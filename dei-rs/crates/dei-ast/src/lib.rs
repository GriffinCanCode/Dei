//! Arena-based AST for efficient tree management
//! 
//! Uses arena allocation for cache-friendly memory layout and zero-copy operations

pub mod arena;
pub mod node;
pub mod builder;
pub mod traverser;
pub mod visitor;

#[cfg(test)]
mod tests;

pub use arena::Arena;
pub use node::{Node, NodeId, NodeKind};
pub use builder::AstBuilder;
pub use traverser::ParallelTraverser;
pub use visitor::Visitor;

