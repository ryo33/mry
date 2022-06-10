use mry::Any;

#[mry::mry]
fn meow<'a>(base: &'a str) -> &'a str {
    base
}

#[test]
fn keeps_original_function() {
    assert_eq!(meow("a"), "a");
}

#[test]
#[mry::lock(meow)]
fn meow_returns() {
    mock_meow(Any).returns_with(|_| "a");
    assert_eq!(meow("a"), "a");
}
