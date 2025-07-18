#[allow(unused_macros)]
macro_rules! assert_panics {
    ($run:block) => {
        assert_panics!($run, "assertion failed to panic")
    };
    ($run:block, $msg:literal) => {
        assert!(std::panic::catch_unwind(|| $run).is_err(), $msg);
        println!("^ panic caught");
    };
}

#[allow(unused_imports)]
pub(crate) use assert_panics;
