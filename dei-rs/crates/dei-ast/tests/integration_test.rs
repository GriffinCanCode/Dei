use dei_ast::{Arena, Node, NodeId, NodeKind, AstBuilder, ParallelTraverser, Visitor};
use dei_core::models::Language;
use std::path::PathBuf;
use std::sync::Arc;

#[test]
fn test_arena_basic_operations() {
    let arena = Arena::new();
    
    let root = Node::new_directory(NodeId(0), PathBuf::from("/project"), 0);
    let root_id = arena.alloc(root.clone());
    
    let retrieved = arena.get(root_id);
    assert!(retrieved.is_some());
    assert_eq!(retrieved.unwrap().name, root.name);
}

#[test]
fn test_node_hierarchy() {
    let arena = Arena::new();
    
    // Create a simple file tree
    let file1 = Node::new_file(NodeId(0), PathBuf::from("/project/src/main.rs"), 2);
    let file2 = Node::new_file(NodeId(1), PathBuf::from("/project/src/lib.rs"), 2);
    
    let file1_id = arena.alloc(file1);
    let file2_id = arena.alloc(file2);
    
    let src_dir = Node::new_directory(NodeId(2), PathBuf::from("/project/src"), 1)
        .with_children(Arc::new([file1_id, file2_id]));
    
    assert_eq!(src_dir.children.len(), 2);
    assert_eq!(src_dir.kind, NodeKind::Directory);
}

#[test]
fn test_language_detection_from_node() {
    let rust_file = Node::new_file(NodeId(0), PathBuf::from("test.rs"), 0);
    assert_eq!(rust_file.language(), Some(Language::Rust));
    
    let csharp_file = Node::new_file(NodeId(1), PathBuf::from("test.cs"), 0);
    assert_eq!(csharp_file.language(), Some(Language::CSharp));
    
    let unknown_file = Node::new_file(NodeId(2), PathBuf::from("readme.md"), 0);
    assert_eq!(unknown_file.language(), None);
}

#[test]
fn test_node_classification() {
    let file_node = Node::new_file(
        NodeId(0),
        PathBuf::from("/test.rs"),
        0
    );
    assert_eq!(file_node.kind, NodeKind::File);
    
    let dir_node = Node::new_directory(
        NodeId(1),
        PathBuf::from("/test"),
        0
    );
    assert_eq!(dir_node.kind, NodeKind::Directory);
}

#[test]
fn test_shared_arena_concurrent_access() {
    use dei_ast::arena::SharedArena;
    use std::sync::Arc;
    use std::thread;
    
    let arena = SharedArena::new();
    let arena_ref = Arc::new(arena);
    
    let mut handles = vec![];
    
    // Spawn multiple threads to allocate nodes
    for i in 0..10 {
        let arena_clone = Arc::clone(&arena_ref);
        let handle = thread::spawn(move || {
            let node = Node::new_file(
                NodeId(i),
                PathBuf::from(format!("/test{}.rs", i)),
                0
            );
            arena_clone.alloc(node)
        });
        handles.push(handle);
    }
    
    // Wait for all threads to complete
    for handle in handles {
        handle.join().unwrap();
    }
    
    assert_eq!(arena_ref.len(), 10);
}

#[test]
fn test_node_id_uniqueness() {
    let arena = Arena::new();
    
    let id1 = arena.alloc(Node::new_file(NodeId(0), PathBuf::from("/a.rs"), 0));
    let id2 = arena.alloc(Node::new_file(NodeId(1), PathBuf::from("/b.rs"), 0));
    let id3 = arena.alloc(Node::new_file(NodeId(2), PathBuf::from("/c.rs"), 0));
    
    assert_ne!(id1, id2);
    assert_ne!(id2, id3);
    assert_ne!(id1, id3);
}

#[test]
fn test_deep_hierarchy() {
    let arena = Arena::new();
    
    // Create a deep file structure
    let file_id = arena.alloc(Node::new_file(NodeId(0), PathBuf::from("/a/b/c/d/e/f/file.rs"), 6));
    
    let dir_e = arena.alloc(Node::new_directory(NodeId(1), PathBuf::from("/a/b/c/d/e"), 5)
        .with_children(Arc::new([file_id])));
    
    let dir_d = arena.alloc(Node::new_directory(NodeId(2), PathBuf::from("/a/b/c/d"), 4)
        .with_children(Arc::new([dir_e])));
    
    let dir_c = arena.alloc(Node::new_directory(NodeId(3), PathBuf::from("/a/b/c"), 3)
        .with_children(Arc::new([dir_d])));
    
    assert_eq!(arena.len(), 4);
}

