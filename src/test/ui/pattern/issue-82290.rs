// check-pass

#![feature(let_chains)]

fn main() {
    if true && let x = 1 { //~ WARN irrefutable `let` pattern
        let _ = x;
    }
}
