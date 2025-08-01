#![cfg(test)]

use std::borrow::Borrow;
use std::hash::{BuildHasher, RandomState};
use std::iter;

use super::*;
use crate::util::alloc::{CountedDrop, ZeroSizedType};
use crate::util::panic::assert_panics;

#[test]
fn test_zst_support() {
    let mut arr = Array::<ZeroSizedType>::repeat_default(5);
    assert_eq!(
        arr[0], ZeroSizedType,
        "Indexing with no offset should work."
    );
    assert_eq!(
        arr[4], ZeroSizedType,
        "Indexing with an in-bounds offset should work."
    );
    assert_eq!(
        arr.iter().as_slice().len(),
        5,
        "Should iterate over the right number of ZST instances."
    );

    let old_ptr = arr.ptr;

    arr.realloc_with_default(30);
    assert_eq!(
        arr.ptr, old_ptr,
        "Pointer shouldn't change when reallocated for a ZST."
    );
}

#[test]
fn test_realloc() {
    let mut arr = Array::from(0..5);
    assert_eq!(arr.size(), 5);

    let old_ptr = arr.ptr;
    arr.realloc_with_default(5);
    assert_eq!(
        arr.ptr, old_ptr,
        "When reallocating to the same size, the pointer shouldn't change."
    );

    arr.realloc_with_default(0);
    assert_ne!(
        arr.ptr, old_ptr,
        "Pointer should be replaced with a dangling one for 0 size."
    );

    let old_ptr = arr.ptr;
    arr.realloc_with_default(10);
    assert_ne!(
        arr.ptr, old_ptr,
        "Pointer should be replaced with an allocated one."
    );

    for i in 0..10 {
        arr[i] = i;
    }

    arr.realloc_with_default(15);
    for i in 0..10 {
        assert_eq!(
            arr[i], i,
            "When growing, all elements should remain in the Array."
        );
    }
    for i in 10..15 {
        assert_eq!(arr[i], 0, "When growing, all new elements should be 0.");
    }

    assert_panics!({
        let mut arr = Array::from(0..5);
        arr.realloc_with_default(isize::MAX as usize + 1)
    });

    let counter = CountedDrop::new(0);
    let mut arr = Array::from(iter::repeat_with(|| counter.clone()).take(10));
    arr.realloc_with(|| unreachable!(), 5);

    assert_eq!(
        counter.take(),
        5,
        "5 elements should have been dropped during shrinking reallocation."
    );
}

#[test]
fn test_drop() {
    let counter = CountedDrop::new(0);
    let arr = Array::from(iter::repeat_with(|| counter.clone()).take(10));

    drop(arr);

    assert_eq!(counter.take(), 10, "10 elements should have been dropped.");
}

#[test]
fn test_equality_and_hash() {
    let arr = Array::from(0_usize..5);

    assert_eq!(
        arr,
        Array::from([0, 1, 2, 3, 4].into_iter()),
        "Different construction methods should produce equal results."
    );
    assert_ne!(Array::from([0, 1, 2, 5, 4].into_iter()), Array::from(0..5));

    assert_eq!(
        &arr.borrow(),
        &[0, 1, 2, 3, 4],
        "Borrow equality should be upheld."
    );
    assert_eq!(&*arr, &[0, 1, 2, 3, 4], "Deref equality should be upheld.");

    let state = RandomState::new();
    assert_eq!(
        state.hash_one(&arr),
        state.hash_one(Array::from(0_usize..5)),
        "Equal arrays should produce the same hash."
    );
    assert_eq!(
        state.hash_one(&arr),
        state.hash_one([0_usize, 1, 2, 3, 4]),
        "Borrow hash equality should be upheld."
    );
}

#[test]
fn test_iterators() {
    let mut arr = Array::from(0_usize..5);
    let collected = Array::from(arr.iter().cloned());
    assert_eq!(arr, collected, "Collected iter should be equal.");

    for i in arr.iter_mut() {
        *i *= 2;
    }
    assert_eq!(
        *arr,
        [0_usize, 2, 4, 6, 8],
        "Array mutated by iterator should equal this slice."
    );

    assert_eq!(
        arr,
        Array::from(arr.clone().into_iter()),
        "Cloned and collected array should be equal."
    );

    let mut iter = arr.into_iter();
    assert_eq!(iter.next(), Some(0));
    assert_eq!(iter.next_back(), Some(8));
    assert_eq!(iter.next_back(), Some(6));
    assert_eq!(iter.next(), Some(2));
    assert_eq!(iter.next_back(), Some(4));
    assert_eq!(iter.next(), None);

    let counter = CountedDrop::new(0);
    let arr = Array::from(iter::repeat_with(|| counter.clone()).take(10));

    drop(arr.into_iter());
    assert_eq!(
        counter.take(),
        10,
        "Dropping an owned iterator should drop all elements."
    );
}
