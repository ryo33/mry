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

#[mry::mry]
impl<'a, T> Test<'a, T>
where
    T: Clone + 'static,
{
    fn fun3<'b>(&self, value: &'b T) -> String
    where
        T: Send,
    {
        todo!()
    }
}

#[allow(dead_code)]
trait Trait<'a, T> {
    fn fun4<'b, U>(&self, value: &'a T, value2: &'b U) -> String
    where
        T: Send,
        U: Send + Clone + 'static;
}

#[mry::mry]
impl<'a, T> Trait<'a, T> for Test<'a, T>
where
    T: Clone + 'static,
{
    fn fun4<'b, U>(&self, value: &'a T, value2: &'b U) -> String
    where
        T: Send,
        U: Send + Clone + 'static,
    {
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

// https://github.com/ryo33/mry/issues/22
mod issue_22 {
    #[allow(dead_code)]
    trait SomeTrait {
        fn create() -> Self;
    }
    #[mry::mry]
    struct Example<T> {
        something: T,
    }
    #[mry::mry]
    impl<T> Example<T>
    where
        T: SomeTrait + Send + 'static,
    {
        fn new() -> Self {
            Self {
                something: T::create(),
                mry: Default::default(),
            }
        }
    }
}
