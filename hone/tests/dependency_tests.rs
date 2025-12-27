use hone::dependency::DependencyGraph;
use hone::node::NodeKey;
use rkyv::{Archive, Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Hash, Archive, Serialize, Deserialize)]
pub struct TestKey(pub String);
impl NodeKey for TestKey {}

#[test]
fn test_dependency_graph_basic() {
    let graph = DependencyGraph::<TestKey>::new();
    let a = TestKey("a".to_string());
    let b = TestKey("b".to_string());

    graph.add_child(a.clone(), b.clone());

    {
        let children_a = graph.get_children(a.clone());
        match children_a {
            dashmap::Entry::Occupied(entry) => assert!(entry.get().contains(&b)),
            dashmap::Entry::Vacant(_) => panic!("Should be occupied"),
        }
    }

    {
        let parents_b = graph.get_parents(b.clone());
        match parents_b {
            dashmap::Entry::Occupied(entry) => assert!(entry.get().contains(&a)),
            dashmap::Entry::Vacant(_) => panic!("Should be occupied"),
        }
    }
}

#[test]
fn test_dependency_graph_clear() {
    let graph = DependencyGraph::<TestKey>::new();
    let a = TestKey("a".to_string());
    let b = TestKey("b".to_string());
    let c = TestKey("c".to_string());

    graph.add_child(a.clone(), b.clone());
    graph.add_child(a.clone(), c.clone());

    graph.clear_children_dependency_of(a.clone());

    {
        let children_a = graph.get_children(a.clone());
        match children_a {
            dashmap::Entry::Occupied(entry) => assert!(entry.get().is_empty()),
            dashmap::Entry::Vacant(_) => {} // Entry might be removed entirely
        }
    }

    {
        let parents_b = graph.get_parents(b.clone());
        match parents_b {
            dashmap::Entry::Occupied(entry) => assert!(!entry.get().contains(&a)),
            dashmap::Entry::Vacant(_) => {}
        }
    }
}
