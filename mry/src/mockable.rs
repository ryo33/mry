#[diagnostic::on_unimplemented(
    message = "`{Self}` is not mockable argument because it is not `Send + 'static`",
    // note = "If you don't need to mock this argument, you can add it to the skip list: `#[mry::mry(skip({Self}))]`"
)]
pub trait MockableArg: Send + 'static {}

#[diagnostic::on_unimplemented(
    message = "`{Self}` is not mockable output because it is not `Send + 'static`",
    // note = "If you don't need to mock this argument, you can add it to the skip list: `#[mry::mry(skip({Self}))]`"
)]
pub trait MockableRet: Send + 'static {}

impl<T: Send + 'static> MockableArg for T {}

impl<T: Send + 'static> MockableRet for T {}

#[test]
fn a() {
    fn assert_mockable<T: MockableArg>(arg: T) -> T {
        arg
    }
    assert_mockable::<&str>("a");
}
