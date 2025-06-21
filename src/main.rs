#![feature(strict_overflow_ops)]
#![feature(box_vec_non_null)]
#![feature(extend_one)]
#![feature(extend_one_unchecked)]

pub mod types;

use types::{contiguous::{Array, Vector}, linked::DLinkedList};

#[derive(Debug, Clone)]
struct MyZST;

impl Drop for MyZST {
    fn drop(&mut self) {
        println!("Dropped MyZST");
    }
}

fn main() {
    println!("\n[Array]\n");

    let mut vec = Vector::<u8>::new();
    println!("{:?}", vec);

    for i in 0..8 {
        vec.push(i);
        println!("{:?}", vec);
    }

    vec.insert(2, 100);

    println!("{:?}, {:?}", vec.remove(3), vec);

    for _ in 0..=10 {
        println!("{:?}", vec.pop());
    }

    println!("{:?}", vec);
    // println!("{:?}", Vector::<u8>::with_cap(14));

    // println!("ZST Testing");

    // // let mut vec = Vector::<MyZST>::new();
    // // println!("{:?}", vec);

    // // for _ in 0..10 {
    // //     vec.push(MyZST);
    // // }

    // // println!("{:?}", vec);

    println!("{:?}", Array::from([&1, &2, &3]));

    let v = Vector::from(Array::from([1, 2, 3]));

    println!("{:?}", v);

    println!("{:?}", Array::from(v));

    let mut ll = DLinkedList::<u8>::new();

    println!("{:?}", ll);

    for i in 0..8 {
        ll.push_back(i);
        println!("{:?}", ll);
    }
    println!("{:?}", ll);

    println!("{:?}", ll.get(4));

    // println!("{:?}", ll.pop_back());
    println!("{:?}", ll);

    // let mut ll = DLinkedList::<MyZST>::new();
    // println!("{:?}", ll);

    // for _ in 0..10 {
    //     ll.push_back(MyZST);
    // }

    println!("{:?}", ll.seek(3).value);

    (&ll).into_iter();

    println!("{:?}", ll);
}
