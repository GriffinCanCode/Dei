//! Visitor pattern for AST traversal with compile-time polymorphism

use dei_core::error::Result;

use crate::{arena::SharedArena, node::{Node, NodeId}};

/// Visitor trait for AST traversal
/// 
/// Uses compile-time polymorphism (static dispatch) for zero-cost abstraction
pub trait Visitor: Send + Sync {
    /// Visit a node before visiting its children
    fn visit_pre(&mut self, node: &Node, arena: &SharedArena) -> Result<()> {
        let _ = (node, arena);
        Ok(())
    }

    /// Visit a node after visiting its children
    fn visit_post(&mut self, node: &Node, arena: &SharedArena) -> Result<()> {
        let _ = (node, arena);
        Ok(())
    }

    /// Visit a file node
    fn visit_file(&mut self, node: &Node, arena: &SharedArena) -> Result<()> {
        self.visit_pre(node, arena)
    }

    /// Visit a directory node
    fn visit_directory(&mut self, node: &Node, arena: &SharedArena) -> Result<()> {
        self.visit_pre(node, arena)
    }
}

/// Walk the AST depth-first with a visitor
pub fn walk<V: Visitor>(
    visitor: &mut V,
    node_id: NodeId,
    arena: &SharedArena,
) -> Result<()> {
    let node = arena.get(node_id).ok_or_else(|| {
        dei_core::Error::Analysis(format!("Node {:?} not found", node_id))
    })?;

    // Pre-visit
    if node.is_file() {
        visitor.visit_file(&node, arena)?;
    } else {
        visitor.visit_directory(&node, arena)?;
    }

    // Visit children
    for child_id in node.children.iter() {
        walk(visitor, *child_id, arena)?;
    }

    // Post-visit
    visitor.visit_post(&node, arena)?;

    Ok(())
}

/// Collect all nodes matching a predicate
pub struct CollectVisitor<F>
where
    F: Fn(&Node) -> bool + Send + Sync,
{
    predicate: F,
    collected: Vec<NodeId>,
}

impl<F> CollectVisitor<F>
where
    F: Fn(&Node) -> bool + Send + Sync,
{
    pub fn new(predicate: F) -> Self {
        Self {
            predicate,
            collected: Vec::new(),
        }
    }

    pub fn into_collected(self) -> Vec<NodeId> {
        self.collected
    }
}

impl<F> Visitor for CollectVisitor<F>
where
    F: Fn(&Node) -> bool + Send + Sync,
{
    fn visit_pre(&mut self, node: &Node, _arena: &SharedArena) -> Result<()> {
        if (self.predicate)(node) {
            self.collected.push(node.id);
        }
        Ok(())
    }
}

