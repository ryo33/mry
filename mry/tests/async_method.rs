use std::time::Duration;

use async_std::task::sleep;

#[mry::mry]
#[derive(Default)]
struct Cat {
    name: String,
}

#[mry::mry]
impl Cat {
    async fn meow(&self, count: usize) -> String {
        sleep(Duration::from_secs(1)).await;
        format!("{}: {}", self.name, "meow".repeat(count))
    }
}

#[async_std::test]
async fn meow_returns_with() {
    let mut cat: Cat = Cat {
        name: "Tama".into(),
        ..Default::default()
    };
    cat.mock_meow()
        .returns_with(|count| format!("Called with {}", count));

    assert_eq!(cat.meow(2).await, "Called with 2".to_string());
}
