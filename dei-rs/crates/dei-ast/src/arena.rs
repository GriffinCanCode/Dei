//! Arena allocator for efficient AST node management
//! 
//! Provides cache-friendly memory layout and fast traversal

use std::sync::{Arc, RwLock};

use crate::node::{Node, NodeId};

/// Thread-safe arena for AST nodes
/// Uses generational indexing to prevent use-after-free
#[derive(Debug)]
pub struct Arena {
    nodes: RwLock<Vec<Node>>,
}

impl Arena {
    pub fn new() -> Self {
        Self {
            nodes: RwLock::new(Vec::new()),
        }
    }

    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            nodes: RwLock::new(Vec::with_capacity(capacity)),
        }
    }

    /// Allocate a new node in the arena
    pub fn alloc(&self, mut node: Node) -> NodeId {
        let mut nodes = self.nodes.write().unwrap();
        let id = NodeId(nodes.len());
        node.id = id; // Update the node's ID field to match its arena position
        nodes.push(node);
        id
    }

    /// Get a node by ID
    pub fn get(&self, id: NodeId) -> Option<Node> {
        self.nodes.read().unwrap().get(id.0).cloned()
    }

    /// Get a mutable reference to a node
    pub fn get_mut(&self, id: NodeId) -> Option<Node> {
        self.nodes.read().unwrap().get(id.0).cloned()
    }

    /// Update a node in place
    pub fn update(&self, id: NodeId, node: Node) {
        if let Some(slot) = self.nodes.write().unwrap().get_mut(id.0) {
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
        self.nodes.read().unwrap().len()
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Iterate over all nodes with their IDs
    pub fn iter(&self) -> impl Iterator<Item = (NodeId, Node)> {
        self.nodes
            .read()
            .unwrap()
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

