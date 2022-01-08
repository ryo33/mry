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

    async fn async_meow(count: usize) -> String {
        "meow".repeat(count)
    }
}

#[mry::lock(hello)]
#[test]
fn function_keeps_original_function() {
    mock_hello(Any).calls_real_impl();
    assert_eq!(hello(3), "hellohellohello");
}

#[mry::lock(Cat::meow)]
#[test]
fn static_method_keeps_original_function() {
    Cat::mock_meow(Any).calls_real_impl();
    assert_eq!(Cat::meow(2), "meowmeow");
}

#[mry::lock(Cat::async_meow)]
#[async_std::test]
async fn static_async_method_keeps_original_function() {
    Cat::mock_async_meow(Any).calls_real_impl();

    assert_eq!(Cat::async_meow(2).await, "meowmeow");
}

#[mry::lock(Cat::meow)]
#[test]
fn meow_returns() {
    Cat::mock_meow(Any).returns("Called".to_string());

    assert_eq!(Cat::meow(2), "Called".to_string());
}

#[test]
#[mry::lock(Cat::meow)]
fn under_test_attr() {
    Cat::mock_meow(Any).returns("Called".to_string());

    assert_eq!(Cat::meow(2), "Called".to_string());
}

#[mry::lock(Cat::async_meow)]
#[async_std::test]
async fn async_test() {
    Cat::mock_async_meow(Any).returns("Called".to_string());

    assert_eq!(Cat::async_meow(2).await, "Called".to_string());
}

#[mry::lock(hello)]
#[test]
fn hello_returns() {
    mock_hello(Any).returns("Called".to_string());

    assert_eq!(hello(2), "Called".to_string());
}

#[mry::lock(hello)]
#[should_panic(expected = "hello is locked but no used.")]
#[test]
fn hello_not_used() {
}
