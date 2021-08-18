#[mry::mry]
#[async_trait::async_trait]
pub trait Cat {
    async fn meow(&self, count: usize) -> String;
}

#[async_std::test]
async fn with_mock() {
    let mut cat = MockCat::default();

    cat.mock_meow()
        .behaves(|count| format!("Called with {}", count));

    assert_eq!(cat.meow(2).await, "Called with 2".to_string());
}
