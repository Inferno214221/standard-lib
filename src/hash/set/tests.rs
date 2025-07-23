#![cfg(test)]

use super::*;
use crate::contiguous::Vector;
use crate::util::hash::{ManualHash, BadHasherBuilder};

#[test]
fn test_hash_collisions() {
    let mut set = HashSet::with_hasher(BadHasherBuilder);
    set.insert(ManualHash::new(0, "zero"));
    set.insert(ManualHash::new(0, "one"));
    set.insert(ManualHash::new(2, "two"));
    set.insert(ManualHash::new(0, "three"));
    set.insert(ManualHash::new(2, "four"));
    set.insert(ManualHash::new(1, "five"));

    set.remove(&ManualHash::new(0, "zero"));
    set.remove(&ManualHash::new(2, "two"));

    assert_eq!(
        *set.into_iter().map(|i| i.value()).collect::<Vector<_>>(),
        ["one", "three", "five", "four"],
        "HashMap should handle hash collisions so that no elements are lost during removal."
    );

    let mut set = HashSet::with_cap_and_hasher(6, BadHasherBuilder);
    set.insert(ManualHash::new(5, "zero"));
    set.insert(ManualHash::new(5, "one"));
    set.insert(ManualHash::new(1, "two"));
    set.insert(ManualHash::new(5, "three"));

    set.remove(&ManualHash::new(5, "zero"));

    assert_eq!(
        *set.into_iter().map(|i| i.value()).collect::<Vector<_>>(),
        ["three", "two", "one"],
        "Hash collisions should be handled in a wrapping manner."
    );
}