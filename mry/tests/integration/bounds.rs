use mry::Any;

#[mry::mry]
fn meow<'a>(base: &'a str) -> &'a str {
    base
}

#[test]
#[mry::lock(meow)]
fn keeps_original_function() {
    mock_meow(Any).calls_real_impl();
    assert_eq!(meow("a"), "a");
}

#[test]
#[mry::lock(meow)]
fn meow_returns() {
    mock_meow(Any).returns_with(|_| "a");
    assert_eq!(meow("a"), "a");
}
