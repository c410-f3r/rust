// run-pass

#![feature(let_chains)]

macro_rules! mac {
    ($e:expr) => {
        if $e {
            true
        } else {
            false
        }
    };
}

fn main() {
    assert_eq!(mac!(let 1 = 1 && true), true);
}
