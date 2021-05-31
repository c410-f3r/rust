#[test]
fn let_chains() {
    let foo = Some(Some(Some(1u32)));
    if let Some(bar) = foo && let Some(baz) = bar && let Some(a) = baz {
        assert_eq!(a, 1u32);
        return;
    }
    unreachable!()
}
