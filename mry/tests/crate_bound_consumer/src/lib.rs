#[test]
fn can_use_trait_declared_in_outer_crate() {
    use crate_bound::Foo as _;
    let mut mock = crate_bound::MockFoo::default();
    mock.mock_foo().returns(42);
    assert_eq!(mock.foo(), 42);
}
