#[mry::mry]
pub trait Cat {
    fn new(name: String) -> Self;
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
        .returns_with(|count| format!("Called with {count}"));

    assert_eq!(cat.meow(2), "Called with 2".to_string());
}

#[test]
#[mry::lock(<MockCat as Cat>::new)]
fn new_method() {
    let mut cat = MockCat::default();

    cat.mock_meow(mry::Any).returns("tama".to_string());

    MockCat::mock_new("Tama").returns(cat);

    assert_eq!(MockCat::new("Tama".into()).meow(2), "tama".to_string());
}

#[test]
#[should_panic(
    expected = "the lock of `<MockCat as Cat>::new` is not acquired. Try `#[mry::lock(<MockCat as Cat>::new)]`"
)]
fn no_lock() {
    MockCat::mock_new("Tama").returns(MockCat::default());
}
