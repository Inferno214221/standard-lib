#![feature(strict_overflow_ops)]
#![feature(box_vec_non_null)]
#![feature(extend_one)]
#![feature(extend_one_unchecked)]
#![feature(trusted_len)]
#![feature(debug_closure_helpers)]

#![allow(clippy::module_inception)]

pub mod binary_tree;
pub mod contiguous;
pub mod hash;
pub mod linked;

pub(crate) mod util;

use std::collections::BTreeMap;

use binary_tree::map::BinaryTreeMap;
use contiguous::{Array, Vector};
use hash::{HashMap, HashSet};
use linked::DoublyLinkedList;

fn main() {
    let mut map: HashMap<String, usize> = dbg!(HashMap::new());
    dbg!(map.insert("one".into(), 1));
    dbg!(&map);
    map.insert("two".into(), 2);
    dbg!(&map);
    map.insert("three".into(), 3);
    dbg!(&map);
    map.insert("four".into(), 4);
    dbg!(&map);

    dbg!(map.insert("two".into(), 5));
    dbg!(&map);

    dbg!(map.remove_entry("three"));
    dbg!(&map);

    dbg!(map.contains("three"));
    dbg!(map.contains("two"));

    dbg!(map.get("one"));
    dbg!(map.get("three"));
    dbg!(map.get_mut("one").map(|m| *m = 7));
    dbg!(&map);

    map.reserve(5);
    dbg!(&map);

    let mut set: HashSet<String> = dbg!(HashSet::with_cap(4));
    dbg!(set.insert("one".into()));
    set.insert("two".into());
    dbg!(set.insert("one".into()));
    dbg!(&set);

    // let mut a: HashSet<usize> = HashSet::new();
    // a.insert(0);
    // a.insert(1);
    // a.insert(2);
    // a.insert(3);

    // let mut b: HashSet<usize> = HashSet::new();
    // b.insert(2);
    // b.insert(3);
    // b.insert(4);
    // b.insert(5);

    // dbg!(a.difference(&b).collect::<Vec<_>>());
    // dbg!(b.difference(&a).collect::<Vec<_>>());
    // dbg!(a.symmetric_difference(&b).collect::<Vec<_>>());
    // dbg!(a.union(&b).collect::<Vec<_>>());
    // dbg!(a.intersection(&b).collect::<Vec<_>>());

    // a.remove(&0);
    // a.remove(&1);
    // a.remove(&2);
    // a.remove(&3);

    // let mut a: HashSet<BadHash> = HashSet::with_cap(5);
    // a.insert(BadHash(1, 0));
    // a.insert(BadHash(2, 0));
    // a.insert(BadHash(3, 0));
    // a.insert(BadHash(4, 1));
    // dbg!(&a);

    // a.remove(&BadHash(1, 0));
    // dbg!(&a);

    let mut bmap: BinaryTreeMap<usize, String> = BinaryTreeMap::new();
    bmap.insert(3, "three".into());
    bmap.insert(1, "one".into());
    bmap.insert(2, "two".into());
    bmap.insert(4, "four".into());

    dbg!(bmap.get(&1));
    dbg!(bmap.get(&2));

    println!("\n[Format Tests]\n");
    println!("{:?}", Array::from([0_u8, 1, 2, 3].into_iter()));
    println!("{}", Array::from([0_u8, 1, 2, 3].into_iter()));
    println!("{:?}", Vector::from([0_u8, 1, 2, 3].into_iter()));
    println!("{}", Vector::from([0_u8, 1, 2, 3].into_iter()));
    println!(
        "{:?}",
        [0_u8, 1, 2, 3].into_iter().collect::<DoublyLinkedList<_>>()
    );
    println!(
        "{}",
        [0_u8, 1, 2, 3].into_iter().collect::<DoublyLinkedList<_>>()
    );
    println!("{:?}", &map);
    println!("{}", &map);
    println!("{:?}", &set);
    println!("{}", &set);
    println!("{:?}", &bmap);
}
