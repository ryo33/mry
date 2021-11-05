use mry::Any;

#[mry::mry]
fn hello(count: usize) -> String {
    "hello".repeat(count)
}

#[mry::mry]
#[derive(Default, PartialEq)]
struct Cat {}

#[mry::mry]
impl Cat {
    fn meow(count: usize) -> String {
        "meow".repeat(count)
    }
}

#[test]
fn function_keeps_original_function() {
    assert_eq!(hello(3), "hellohellohello");
}

#[test]
fn static_method_keeps_original_function() {
    assert_eq!(Cat::meow(2), "meowmeow");
}

#[test]
fn meow_returns() {
    Cat::mock_meow(Any).returns("Called".to_string());

    assert_eq!(Cat::meow(2), "Called".to_string());
}

#[test]
fn hello_returns() {
    mock_hello(Any).returns("Called".to_string());

    assert_eq!(hello(2), "Called".to_string());
}
