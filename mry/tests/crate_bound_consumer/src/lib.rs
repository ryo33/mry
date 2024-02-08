#[test]
fn can_use_trait_declared_in_outer_crate() {
    use mry_crate_bound::Foo as _;
    let mut mock = mry_crate_bound::MockFoo::default();
    mock.mock_foo().returns(42);
    assert_eq!(mock.foo(), 42);
}
