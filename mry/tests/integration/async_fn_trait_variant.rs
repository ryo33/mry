#[trait_variant::make(Cat: Send)]
#[mry::mry]
pub trait LocalCat {
    async fn meow(&self, count: usize) -> &'static str;
}

#[async_std::test]
async fn meow_called() {
    let mut cat = MockCat::default();

    cat.mock_meow(2).returns_ready("Called");

    assert_eq!(Cat::meow(&cat, 2).await, "Called");
}
