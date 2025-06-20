#![feature(strict_overflow_ops)]
#![feature(box_vec_non_null)]
#![feature(extend_one)]
#![feature(extend_one_unchecked)]

pub mod types;

use types::{Array, Vector};

#[derive(Debug, Clone)]
struct MyZST;

impl Drop for MyZST {
    fn drop(&mut self) {
        println!("Dropped MyZST");
    }
}

fn main() {
    println!("\n[Array]\n");

    // let mut arr = Array::repeat("ea", 10);
    // arr[0] = "a";
    // let e = &mut *arr;
    // e[1] = "f";

    // println!("{:?}", arr.get(0));
    // println!("{:?}", &*arr);
    // println!("{:?}", Array::from([1_u32, 2, 3]));
    // println!("{:?}", Array::<u32>::from([]).get(0));

    // let mut b = Box::new_uninit_slice(10);
    // b.fill(MaybeUninit::new("ea"));
    // let a = unsafe { b.assume_init() };
    // println!("{:?}", a);
    // dbg!(size_of::<Box<[u8]>>());
    // dbg!(size_of::<Array<u8>>());

    let mut vec = Vector::<u8>::new();
    println!("{:?}", vec);

    for i in 0..8 {
        vec.push(i);
        println!("{:?}", vec);
    }

    vec.insert(2, 100);

    // for _ in 0..=10 {
    //     println!("{:?}", vec.pop());
    // }
    
    println!("{:?}", vec);
    println!("{:?}, {:?}", vec.remove(3), vec);
    // println!("{:?}", Vector::<u8>::with_cap(14));

    println!("ZST Testing");

    let mut vec = Vector::<MyZST>::new();
    println!("{:?}", vec);

    for _ in 0..10 {
        vec.push(MyZST);
    }

    println!("{:?}", vec);
}
