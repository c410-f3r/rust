// check-pass

#![feature(let_chains)]
#![allow(irrefutable_let_patterns)]

fn let_chain() {
    let foo = Some(Some(Some(123u32)));
    if let Some(bar) = foo && let Some(baz) = bar {
        let _something = baz;
    }
}

fn main() {
}
