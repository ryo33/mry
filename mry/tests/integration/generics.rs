#[mry::mry]
struct Test<'a, T> {
    value: &'a T,
}

#[mry::mry]
impl<'a, T: Clone + Send + 'static> Test<'a, T> {
    fn fun1<'b>(&self, value: &'b T) -> String {
        todo!()
    }
}

#[mry::mry]
impl Test<'_, i32> {
    fn fun2(&self, value: &i32) -> String {
        todo!()
    }
}

#[test]
fn test() {
    let value = 42;
    let _test = Test {
        value: &value,
        mry: Default::default(),
    };
}
