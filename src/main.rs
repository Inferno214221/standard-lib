#![feature(strict_overflow_ops)]
#![feature(box_vec_non_null)]
#![feature(extend_one)]
#![feature(extend_one_unchecked)]
#![feature(trusted_len)]

#![allow(clippy::module_inception)]

pub mod contiguous;
pub mod linked;
pub mod hash;

use std::alloc::Layout;

use contiguous::{Array, Vector};
use linked::DoublyLinkedList;

#[derive(Debug, Clone)]
struct MyZST;

impl Drop for MyZST {
    fn drop(&mut self) {
        println!("Dropped MyZST");
    }
}

fn main() {
    // println!("\n[Array]\n");

    // let mut vec = Vector::<u8>::new();
    // println!("{:?}", vec);

    // for i in 0..8 {
    //     vec.push(i);
    //     println!("{:?}", vec);
    // }

    // vec.insert(2, 100);

    // println!("{:?}, {:?}", vec.remove(3), vec);

    // for _ in 0..=10 {
    //     println!("{:?}", vec.pop());
    // }

    // println!("{:?}", vec);
    // // println!("{:?}", Vector::<u8>::with_cap(14));

    // // println!("ZST Testing");

    let mut vec = Vector::<MyZST>::new();
    println!("{vec:?}");

    for _ in 0..10 {
        vec.push(MyZST);
    }

    println!("{vec:?}");

    let mut iter = vec.into_iter();
    while let Some(i) = iter.next_back() {
        println!("{i:?}");
    }

    // println!("{:?}", Array::from([&1, &2, &3].into_iter()));

    // let v = Vector::from(Array::from([1, 2, 3].into_iter()));

    // println!("{:?}", v);

    // println!("{:?}", Array::from(v));

    // // let mut ll = DoublyLinkedList::<u8>::new();

    // // println!("{:?}", ll);

    // // for i in 0..8 {
    // //     ll.push_back(i);
    // //     println!("{:?}", ll);
    // // }
    // // println!("{:?}", ll);

    // // println!("{:?}", ll.get(4));

    // // // println!("{:?}", ll.pop_back());

    // // // let mut ll = DLinkedList::<MyZST>::new();
    // // // println!("{:?}", ll);

    // // // for _ in 0..10 {
    // // //     ll.push_back(MyZST);
    // // // }

    // // println!("{:?}", ll.get(3));

    // // let mut cur = ll.into_cursor().unwrap();
    // // cur.move_next();

    // // for _ in 0..4 {
    // //     cur.pop_next();
    // //     println!("{:?}", cur);
    // // }

    // // cur.push_next(3);
    // // println!("{:?}", cur);

    // // cur.as_list().verify_double_links();

    // // println!("{:?}", Vector::from(0..1));

    // let mut ll = DoublyLinkedList::<u8>::new();

    // let mut vec: Vector<_> = "Hello world!".chars().collect();
    // assert_eq!(vec.remove(1), 'e');
    // assert_eq!(vec.remove(4), ' ');
    // assert_eq!(vec, dbg!("Hlloworld!".chars()).collect());
}
