#[cfg(test)]
mod tests {
    use crate::{arena::*, node::*};
    use dei_core::models::Language;
    use std::path::PathBuf;
    use std::sync::Arc;

    #[test]
    fn test_arena_allocation() {
        let arena = Arena::new();
        
        let node = Node::new_file(NodeId(0), PathBuf::from("/test.rs"), 0);
        let id = arena.alloc(node.clone());
        
        let retrieved = arena.get(id);
        assert!(retrieved.is_some());
        assert_eq!(retrieved.unwrap().name, node.name);
    }

    #[test]
    fn test_node_language_detection() {
        let rust_node = Node::new_file(NodeId(0), PathBuf::from("test.rs"), 0);
        assert_eq!(rust_node.language(), Some(dei_core::models::Language::Rust));

        let csharp_node = Node::new_file(NodeId(1), PathBuf::from("test.cs"), 0);
        assert_eq!(csharp_node.language(), Some(dei_core::models::Language::CSharp));

        let unknown_node = Node::new_file(NodeId(2), PathBuf::from("test.txt"), 0);
        assert_eq!(unknown_node.language(), None);
    }

    #[test]
    fn test_shared_arena() {
        let arena = SharedArena::new();
        
        let node1 = Node::new_file(NodeId(0), PathBuf::from("/test1.rs"), 0);
        let node2 = Node::new_file(NodeId(1), PathBuf::from("/test2.rs"), 0);
        
        let id1 = arena.alloc(node1);
        let id2 = arena.alloc(node2);
        
        assert_ne!(id1, id2);
        assert_eq!(arena.len(), 2);
    }

    #[test]
    fn test_node_with_children() {
        let arena = SharedArena::new();
        
        let child1_id = arena.alloc(Node::new_file(NodeId(0), PathBuf::from("/child1.rs"), 1));
        let child2_id = arena.alloc(Node::new_file(NodeId(1), PathBuf::from("/child2.rs"), 1));
        
        let parent = Node::new_directory(NodeId(2), PathBuf::from("/parent"), 0)
            .with_children(Arc::new([child1_id, child2_id]));
        
        assert_eq!(parent.children.len(), 2);
    }
}

