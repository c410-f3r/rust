// gate-test-param_attrs

#![deny(unused_variables)]

fn foo(
    /// Foo
    //~^ ERROR documentation comments cannot be applied to function parameters
    //~| NOTE doc comments are not allowed here
    #[allow(unused_variables)] a: u8
) {}

fn main() {}
