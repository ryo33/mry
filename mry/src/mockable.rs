#[diagnostic::on_unimplemented(
    message = "`{Self}` is not mockable argument because it is not `Send + 'static`",
    note = "Consider `#[mry::mry(non_send(Rc, YourNotSendType))]` to enable SendWrapper, `#[mry::mry(skip_args(Rc, YourNotSendType))]` to skip the argument, or `#[mry::mry(skip_methods(method_name))]` to skip the method"
)]
pub trait MockableArg: Send + 'static {}

#[diagnostic::on_unimplemented(
    message = "`{Self}` is not mockable output because it is not `Send + 'static`",
    note = "Consider `#[mry::mry(non_send(Rc, YourNotSendType))]` to enable SendWrapper, `#[mry::mry(skip_args(Rc, YourNotSendType))]` to skip the argument, or `#[mry::mry(skip_methods(method_name))]` to skip the method"
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
