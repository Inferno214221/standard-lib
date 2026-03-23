use super::*;

// ============================================================================
// ConsTree::new / Default / is_empty
// ============================================================================

#[test]
fn test_new_is_empty() {
    let tree: ConsTree<i32> = ConsTree::new();
    assert!(tree.is_empty());
}

#[test]
fn test_default_is_empty() {
    let tree: ConsTree<i32> = ConsTree::default();
    assert!(tree.is_empty());
}

#[test]
fn test_push_makes_non_empty() {
    let mut tree = ConsTree::new();
    tree.push(1);
    assert!(!tree.is_empty());
}

// ============================================================================
// push / iter
// ============================================================================

#[test]
fn test_iter_empty() {
    let tree: ConsTree<i32> = ConsTree::new();
    assert_eq!(tree.iter().count(), 0);
}

#[test]
fn test_push_iter_order() {
    let mut tree = ConsTree::new();
    tree.push(1);
    tree.push(2);
    tree.push(3);
    let items: Vec<_> = tree.iter().copied().collect();
    assert_eq!(items, vec![3, 2, 1]);
}

#[test]
fn test_iter_does_not_consume() {
    let mut tree = ConsTree::new();
    tree.push(1);
    let _ = tree.iter().count();
    assert!(!tree.is_empty());
}

// ============================================================================
// Clone (shallow)
// ============================================================================

#[test]
fn test_clone_shares_tail() {
    let mut tree1 = ConsTree::new();
    tree1.push(1);
    tree1.push(2);
    let mut tree2 = tree1.clone();
    tree2.push(3);

    let items1: Vec<_> = tree1.iter().copied().collect();
    let items2: Vec<_> = tree2.iter().copied().collect();
    assert_eq!(items1, vec![2, 1]);
    assert_eq!(items2, vec![3, 2, 1]);
}

// ============================================================================
// is_unique / is_head_unique
// ============================================================================

#[test]
fn test_is_unique_empty() {
    assert!(ConsTree::<i32>::new().is_unique());
}

#[test]
fn test_is_unique_sole_owner() {
    let mut tree = ConsTree::new();
    tree.push(1);
    assert!(tree.is_unique());
}

#[test]
fn test_is_unique_shared() {
    let mut tree1 = ConsTree::new();
    tree1.push(1);
    let _tree2 = tree1.clone();
    assert!(!tree1.is_unique());
}

#[test]
fn test_is_head_unique_empty() {
    assert!(ConsTree::<i32>::new().is_head_unique());
}

#[test]
fn test_is_head_unique_sole_owner() {
    let mut tree = ConsTree::new();
    tree.push(1);
    assert!(tree.is_head_unique());
}

#[test]
fn test_is_head_unique_shared_head() {
    let mut tree1 = ConsTree::new();
    tree1.push(1);
    let _tree2 = tree1.clone();
    assert!(!tree1.is_head_unique());
}

#[test]
fn test_is_head_unique_with_shared_tail() {
    let mut tree1 = ConsTree::new();
    tree1.push(1);
    let _tree2 = tree1.clone(); // shares node(1)
    tree1.push(2);              // unique head on top
    assert!(tree1.is_head_unique());
    let _ = _tree2;
}

// ============================================================================
// pop_if_unique
// ============================================================================

#[test]
fn test_pop_if_unique_empty() {
    let mut tree: ConsTree<i32> = ConsTree::new();
    assert_eq!(tree.pop_if_unique(), None);
    assert!(tree.is_empty());
}

#[test]
fn test_pop_if_unique_sole_owner() {
    let mut tree = ConsTree::new();
    tree.push(42);
    assert_eq!(tree.pop_if_unique(), Some(42));
    assert!(tree.is_empty());
}

#[test]
fn test_pop_if_unique_shared_returns_none() {
    let mut tree1 = ConsTree::new();
    tree1.push(1);
    let _tree2 = tree1.clone();
    assert_eq!(tree1.pop_if_unique(), None);
    assert!(!tree1.is_empty());
}

// ============================================================================
// keep_unique
// ============================================================================

#[test]
fn test_keep_unique_empty() {
    let mut tree: ConsTree<i32> = ConsTree::new();
    let shared = tree.split_off_unique();
    assert!(tree.is_empty());
    assert!(shared.is_empty());
}

#[test]
fn test_keep_unique_all_unique() {
    let mut tree = ConsTree::new();
    tree.push(1);
    tree.push(2);
    let unique = tree.split_off_unique();
    assert!(tree.is_empty());
    assert_eq!(unique.iter().copied().collect::<Vec<_>>(), vec![2, 1]);
}

#[test]
fn test_keep_unique_partially_shared() {
    let mut tree1 = ConsTree::new();
    tree1.push(1);
    let _shared_ref = tree1.clone(); // node(1) shared
    tree1.push(2);                   // unique head

    let unique = tree1.split_off_unique();
    assert_eq!(tree1.iter().copied().collect::<Vec<_>>(), vec![1]);
    assert_eq!(unique.iter().copied().collect::<Vec<_>>(), vec![2]);
}

// ============================================================================
// into_iter_unique
// ============================================================================

#[test]
fn test_into_iter_unique_empty() {
    let tree: ConsTree<i32> = ConsTree::new();
    assert_eq!(tree.into_iter_unique().collect::<Vec<_>>(), vec![]);
}

#[test]
fn test_into_iter_unique_all_unique() {
    let mut tree = ConsTree::new();
    tree.push(1);
    tree.push(2);
    assert_eq!(tree.into_iter_unique().collect::<Vec<_>>(), vec![2, 1]);
}

#[test]
fn test_into_iter_unique_stops_at_shared() {
    let mut tree1 = ConsTree::new();
    tree1.push(1);
    let _tree2 = tree1.clone();
    tree1.push(2);

    let items: Vec<_> = tree1.into_iter_unique().collect();
    assert_eq!(items, vec![2]);
}

#[test]
fn test_into_iter_unique_remainder() {
    let mut tree1 = ConsTree::new();
    tree1.push(1);
    let _tree2 = tree1.clone();
    tree1.push(2);

    let mut iter = tree1.into_iter_unique();
    assert_eq!(iter.next(), Some(2));
    assert_eq!(iter.next(), None);

    let remainder = iter.remainder();
    assert_eq!(remainder.iter().copied().collect::<Vec<_>>(), vec![1]);
}

// ============================================================================
// pop_to_owned / into_iter_owned
// ============================================================================

#[test]
fn test_pop_to_owned_empty() {
    let mut tree: ConsTree<i32> = ConsTree::new();
    assert_eq!(tree.pop_to_owned(), None);
}

#[test]
fn test_pop_to_owned_unique() {
    let mut tree = ConsTree::new();
    tree.push(42);
    assert_eq!(tree.pop_to_owned(), Some(42));
    assert!(tree.is_empty());
}

#[test]
fn test_pop_to_owned_shared_clones_value() {
    let mut tree1 = ConsTree::new();
    tree1.push(1);
    let _tree2 = tree1.clone();
    assert_eq!(tree1.pop_to_owned(), Some(1));
    assert!(tree1.is_empty());
}

#[test]
fn test_into_iter_owned_empty() {
    let tree: ConsTree<i32> = ConsTree::new();
    assert_eq!(tree.into_iter_owned().collect::<Vec<_>>(), vec![]);
}

#[test]
fn test_into_iter_owned_single() {
    let mut tree = ConsTree::new();
    tree.push(42);
    assert_eq!(tree.into_iter_owned().collect::<Vec<_>>(), vec![42]);
}

#[test]
fn test_into_iter_owned_multiple() {
    let mut tree = ConsTree::new();
    tree.push(1);
    tree.push(2);
    tree.push(3);
    assert_eq!(tree.into_iter_owned().collect::<Vec<_>>(), vec![3, 2, 1]);
}

#[test]
fn test_into_iter_owned_shared_clones() {
    let mut tree1 = ConsTree::new();
    tree1.push(1);
    let _tree2 = tree1.clone();
    tree1.push(2);

    assert_eq!(tree1.into_iter_owned().collect::<Vec<_>>(), vec![2, 1]);
}

#[test]
fn test_into_iter_owned_remainder() {
    let mut tree = ConsTree::new();
    tree.push(1);
    tree.push(2);

    let mut iter = tree.into_iter_owned();
    assert_eq!(iter.next(), Some(2));

    let remainder = iter.remainder();
    assert_eq!(remainder.into_iter_owned().collect::<Vec<_>>(), vec![1]);
}

// ============================================================================
// into_iter_rc
// ============================================================================

#[test]
fn test_into_iter_rc_values() {
    let mut tree = ConsTree::new();
    tree.push(1);
    tree.push(2);

    let items: Vec<_> = tree.into_iter_rc().map(|n| n.value).collect();
    assert_eq!(items, vec![2, 1]);
}

#[test]
fn test_into_iter_rc_remainder() {
    let mut tree = ConsTree::new();
    tree.push(1);
    tree.push(2);

    let mut iter = tree.into_iter_rc();
    let _ = iter.next();
    let remainder = iter.remainder();
    assert_eq!(remainder.iter().copied().collect::<Vec<_>>(), vec![1]);
}

// ============================================================================
// deep_clone
// ============================================================================

#[test]
fn test_deep_clone_same_values() {
    let mut tree = ConsTree::new();
    tree.push(1);
    tree.push(2);

    let clone = tree.deep_clone();
    assert_eq!(
        tree.iter().copied().collect::<Vec<_>>(),
        clone.iter().copied().collect::<Vec<_>>()
    );
}

#[test]
fn test_deep_clone_is_unique() {
    let mut tree = ConsTree::new();
    tree.push(1);
    let _tree2 = tree.clone(); // shared

    let deep = tree.deep_clone();
    assert!(deep.is_unique());
}

#[test]
fn test_deep_clone_independence() {
    let mut tree = ConsTree::new();
    tree.push(1);

    let mut deep = tree.deep_clone();
    deep.push(2);

    assert_eq!(tree.iter().copied().collect::<Vec<_>>(), vec![1]);
    assert_eq!(deep.iter().copied().collect::<Vec<_>>(), vec![2, 1]);
}

// ============================================================================
// FromIterator
// ============================================================================

#[test]
fn test_from_iterator() {
    let tree: ConsTree<i32> = vec![1, 2, 3].into_iter().collect();
    assert_eq!(tree.iter().copied().collect::<Vec<_>>(), vec![3, 2, 1]);
}

#[test]
fn test_from_iterator_empty() {
    let tree: ConsTree<i32> = std::iter::empty().collect();
    assert!(tree.is_empty());
}

// ============================================================================
// Debug
// ============================================================================

#[test]
fn test_debug_empty() {
    let tree: ConsTree<i32> = ConsTree::new();
    assert_eq!(format!("{:?}", tree), "()");
}

#[test]
fn test_debug_single() {
    let mut tree = ConsTree::new();
    tree.push(42);
    let s = format!("{:?}", tree);
    assert!(s.contains("42"));
}

#[test]
fn test_debug_multiple() {
    let mut tree = ConsTree::new();
    tree.push(1);
    tree.push(2);
    let s = format!("{:?}", tree);
    assert!(s.contains("1") && s.contains("2"));
}

#[test]
fn test_debug_node() {
    let mut tree = ConsTree::new();
    tree.push(7);
    let node = tree.inner.as_ref().unwrap();
    let s = format!("{:?}", node);
    assert!(s.contains("7"));
}

// ============================================================================
// ConsTreeNode::Deref
// ============================================================================

#[test]
fn test_node_deref() {
    let mut tree = ConsTree::new();
    tree.push(99i32);
    let node = tree.inner.as_ref().unwrap();
    // Deref on ConsTreeNode yields the inner value via its Deref impl
    assert_eq!(node.value, 99);
}

// ============================================================================
// Iter (borrowed)
// ============================================================================

#[test]
fn test_iter_clone_independence() {
    let mut tree = ConsTree::new();
    tree.push(1);
    tree.push(2);

    let mut iter1 = tree.iter();
    iter1.next();
    let iter2 = iter1.clone();

    assert_eq!(iter1.copied().collect::<Vec<_>>(), iter2.copied().collect::<Vec<_>>());
}
