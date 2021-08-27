#[mry::mry]
#[async_trait::async_trait]
pub trait Cat {
    async fn meow(&self, count: usize) -> &'static str;
}

#[async_std::test]
async fn meow_called() {
    let mut cat = MockCat::default();

    cat.mock_meow(2).returns("Called");

    assert_eq!(cat.meow(2).await, "Called");
}
