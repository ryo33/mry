#[mry::mry]
pub trait Cat {
    fn meow(&self, count: usize) -> String;
    fn meow_default(&self, count: usize) -> String {
        "meow".repeat(count)
    }
}

#[test]
fn respects_default() {
    let cat = MockCat::default();

    assert_eq!(cat.meow_default(2), "meowmeow".to_string());
}

#[test]
#[should_panic(expected = "mock not found for Cat")]
fn no_mock() {
    let cat = MockCat::default();

    cat.meow(2);
}

#[test]
fn with_mock() {
    let mut cat = MockCat::default();

    cat.mock_meow(2)
        .returns_with(|count| format!("Called with {}", count));

    assert_eq!(cat.meow(2), "Called with 2".to_string());
}
