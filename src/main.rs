use std::{mem::MaybeUninit, os::unix::ffi::OsStrExt, path::PathBuf};

use libc::dirent;
use standard_lib::{collections::*, fs::{path::Rel, *}};

use contiguous::{Array, Vector};
use hash::{HashMap, HashSet};
use linked::LinkedList;
use traits::set::SetIterator;

use file::File;
use path::OwnedPath;

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

    let mut a: HashSet<usize> = HashSet::new();
    a.insert(0);
    a.insert(1);
    a.insert(2);
    a.insert(3);

    let mut b: HashSet<usize> = HashSet::new();
    b.insert(2);
    b.insert(3);
    b.insert(4);
    b.insert(5);

    dbg!(a.difference(&b).collect::<Vector<_>>());
    dbg!(b.difference(&a).collect::<Vector<_>>());
    dbg!(a.symmetric_difference(&b).collect::<Vector<_>>());
    dbg!(a.union(&b).collect::<Vector<_>>());
    dbg!(a.intersection(&b).collect::<Vector<_>>());

    a.remove(&0);
    a.remove(&1);
    a.remove(&2);
    a.remove(&3);

    // let mut a: HashSet<BadHash> = HashSet::with_cap(5);
    // a.insert(BadHash(1, 0));
    // a.insert(BadHash(2, 0));
    // a.insert(BadHash(3, 0));
    // a.insert(BadHash(4, 1));
    // dbg!(&a);

    // a.remove(&BadHash(1, 0));
    // dbg!(&a);

    let mut list = LinkedList::new();
    list.push_back("zero");
    list.push_back("one");
    list.push_back("two");
    list.push_back("three");
    list.push_back("four");
    list.push_back("five");
    list.push_back("six");

    let mut cursor = list.cursor_front();
    cursor.move_next().move_next().move_next();
    dbg!(cursor.index());

    dbg!(cursor.get(0));
    dbg!(cursor.get(1));
    dbg!(cursor.get(2));
    dbg!(cursor.get(3));
    dbg!(cursor.get(4));
    dbg!(cursor.get(5));
    dbg!(cursor.get(6));

    println!("\n[Format Tests]\n");
    println!("{:#?}", Array::from_iter_sized([0_u8, 1, 2, 3].into_iter()));
    println!("{}", Array::from_iter_sized([0_u8, 1, 2, 3].into_iter()));
    println!("{:#?}", Vector::from_iter_sized([0_u8, 1, 2, 3].into_iter()));
    println!("{}", Vector::from_iter_sized([0_u8, 1, 2, 3].into_iter()));
    println!(
        "{:?}",
        [0_u8, 1, 2, 3].into_iter().collect::<LinkedList<_>>()
    );
    println!(
        "{}",
        [0_u8, 1, 2, 3].into_iter().collect::<LinkedList<_>>()
    );
    println!("{:#?}", &map);
    println!("{}", &map);
    println!("{:#?}", &set);
    println!("{}", &set);

    let f = File::open(OwnedPath::<Rel>::from("./hello.txt").resolve(OwnedPath::cwd().unwrap())).unwrap();
    println!("{}", f.read_all_string().unwrap());
    // unsafe {
    //     assert_ne!(
    //         libc::syscall(libc::SYS_open, PathBuf::from("./test").as_os_str().as_bytes().as_ptr().cast::<u8>(), 0),
    //         -1
    //     );
    //     assert_ne!(libc::open(PathBuf::from("./test").as_os_str().as_bytes().as_ptr().cast(), 0, 0o644), -1);
    //     assert!(File::options().read_only().if_present().extra_flags(libc::O_PATH).open(PathBuf::from("./test").as_path()).is_ok());
    // }
    // unsafe {
    //     // FIXME: This is invalid because PathBuf isn't nul-terminated.
    //     let fd = libc::open(PathBuf::from("./test").as_os_str().as_bytes().as_ptr().cast(), 0);
    //     let mut dirp: Array<MaybeUninit<dirent>> = Array::new_uninit(10);
    //     let out = libc::syscall(libc::SYS_getdents64, fd, dirp.as_mut_ptr(), size_of::<libc::dirent64>() / 2) as isize;
    //     println!("{out:x}");
    //     println!("{:02x}", dirp[1].assume_init().d_type);
    //     println!("{:02x}, {:02x}", libc::DT_REG, libc::DT_LNK);
    //     println!("{}", std::slice::from_raw_parts(
    //         dirp.as_ptr().cast::<u8>(),
    //         size_of::<libc::dirent64>()
    //     ).iter().map(|b| format!("{b:02x}")).collect::<String>());
    //     println!("{}", "file-1".as_bytes().iter().map(|b| format!("{b:02x}")).collect::<String>());
    //     println!("{}", ".".as_bytes().iter().map(|b| format!("{b:02x}")).collect::<String>());

    //     // println!("{:?}", Directory::open(PathBuf::from("./test").as_path()).unwrap().entries().next())
    // }

    let downloads = OwnedPath::<Rel>::from("./downloads");
    println!("{}, {}, {}", downloads.display(), downloads.display().slash(), downloads.display().no_lead());
    let full = downloads.resolve(OwnedPath::home().unwrap());
    println!("{}, {}", full.display(), full.display().shrink_home());
}
