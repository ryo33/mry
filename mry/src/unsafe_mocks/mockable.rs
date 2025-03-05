#[diagnostic::on_unimplemented(
    message = "`{Self}` is not mockable argument because it is not `'static`",
    // note = "If you don't need to mock this argument, you can add it to the skip list: `#[mry::mry(skip({Self}))]`"
)]
pub trait UnsafeMockableArg: 'static {}

#[diagnostic::on_unimplemented(
    message = "`{Self}` is not mockable output because it is not `'static`",
    // note = "If you don't need to mock this argument, you can add it to the skip list: `#[mry::mry(skip({Self}))]`"
)]
pub trait UnsafeMockableRet: 'static {}

impl<T: 'static> UnsafeMockableArg for T {}

impl<T: 'static> UnsafeMockableRet for T {}

pub fn assert_mockable<T: UnsafeMockableArg>(arg: T) -> T {
    arg
}

#[test]
fn a() {
    assert_mockable::<&str>("a");
}
