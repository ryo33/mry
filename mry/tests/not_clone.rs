// no clone
#[derive(Debug, PartialEq)]
struct A(u8);

#[mry::mry]
struct Struct {}

#[mry::mry]
impl Struct {
    fn wrap(&self, value: u8) -> A {
        A(value)
    }
}
#[test]
fn not_clone() {
    let mut target = mry::new!(Struct {});
    target.mock_wrap(mry::Any).returns_with(|_| A(42));

    assert_eq!(target.wrap(2), A(42));
}
