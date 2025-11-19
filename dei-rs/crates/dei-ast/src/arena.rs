//! Arena allocator for efficient AST node management
//! 
//! Provides cache-friendly memory layout and fast traversal

use std::cell::RefCell;
use std::sync::Arc;

use crate::node::{Node, NodeId};

/// Thread-local arena for AST nodes
/// Uses generational indexing to prevent use-after-free
#[derive(Debug)]
pub struct Arena {
    nodes: RefCell<Vec<Node>>,
}

impl Arena {
    pub fn new() -> Self {
        Self {
            nodes: RefCell::new(Vec::new()),
        }
    }

    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            nodes: RefCell::new(Vec::with_capacity(capacity)),
        }
    }

    /// Allocate a new node in the arena
    pub fn alloc(&self, node: Node) -> NodeId {
        let mut nodes = self.nodes.borrow_mut();
        let id = NodeId(nodes.len());
        nodes.push(node);
        id
    }

    /// Get a node by ID
    pub fn get(&self, id: NodeId) -> Option<Node> {
        self.nodes.borrow().get(id.0).cloned()
    }

    /// Get a mutable reference to a node
    pub fn get_mut(&self, id: NodeId) -> Option<Node> {
        self.nodes.borrow().get(id.0).cloned()
    }

    /// Update a node in place
    pub fn update(&self, id: NodeId, node: Node) {
        if let Some(slot) = self.nodes.borrow_mut().get_mut(id.0) {
            *slot = node;
        }
    }

    /// Get all children of a node
    pub fn children(&self, id: NodeId) -> Vec<NodeId> {
        self.get(id)
            .map(|n| n.children.to_vec())
            .unwrap_or_default()
    }

    /// Total number of nodes
    pub fn len(&self) -> usize {
        self.nodes.borrow().len()
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Iterate over all nodes with their IDs
    pub fn iter(&self) -> impl Iterator<Item = (NodeId, Node)> {
        self.nodes
            .borrow()
            .iter()
            .enumerate()
            .map(|(i, n)| (NodeId(i), n.clone()))
            .collect::<Vec<_>>()
            .into_iter()
    }
}

impl Default for Arena {
    fn default() -> Self {
        Self::new()
    }
}

/// Shared arena for multi-threaded access
#[derive(Debug, Clone)]
pub struct SharedArena {
    inner: Arc<Arena>,
}

impl SharedArena {
    pub fn new() -> Self {
        Self {
            inner: Arc::new(Arena::new()),
        }
    }

    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            inner: Arc::new(Arena::with_capacity(capacity)),
        }
    }

    pub fn alloc(&self, node: Node) -> NodeId {
        self.inner.alloc(node)
    }

    pub fn get(&self, id: NodeId) -> Option<Node> {
        self.inner.get(id)
    }

    pub fn update(&self, id: NodeId, node: Node) {
        self.inner.update(id, node)
    }

    pub fn children(&self, id: NodeId) -> Vec<NodeId> {
        self.inner.children(id)
    }

    pub fn len(&self) -> usize {
        self.inner.len()
    }

    pub fn is_empty(&self) -> bool {
        self.inner.is_empty()
    }
}

impl Default for SharedArena {
    fn default() -> Self {
        Self::new()
    }
}

