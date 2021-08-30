use std::fmt::Debug;
use std::marker::PhantomData;
use std::ops::DerefMut;

use crate::{Behavior, Matcher, Mock, MockResult, Mocks};

/// Mock locator returned by mock_* methods
pub struct MockLocator<M, I, O, B> {
    #[doc(hidden)]
    pub mocks: M,
    #[doc(hidden)]
    pub name: &'static str,
    #[doc(hidden)]
    pub matcher: Option<Matcher<I>>,
    #[doc(hidden)]
    pub _phantom: PhantomData<fn() -> (I, O, B)>,
}

impl<M, I, O, B> MockLocator<M, I, O, B>
where
    M: DerefMut<Target = Mocks>,
    I: Clone + PartialEq + Debug + Send + Sync + 'static,
    O: Debug + Send + Sync + 'static,
    B: Into<Behavior<I, O>>,
{
    /// Returns value with using a clojure.
    /// Arguments of a method call are passed to the given clojure.
    pub fn returns_with<T: Into<B>>(&mut self, behavior: T) {
        let matcher = self.matcher();
        self.get_mut_or_default()
            .returns_with(matcher, behavior.into());
    }

    /// This make the mock calls real impl. This is used for partial mocking.
    pub fn calls_real_impl(&mut self) {
        let matcher = self.matcher();
        self.get_mut_or_default().calls_real_impl(matcher);
    }

    /// Assert the mock is called.
    /// Returns `MockResult` allows to call `times(n)`
    /// Panics if not called
    pub fn assert_called(&mut self) -> MockResult<I> {
        let matcher = self.matcher.take().unwrap();
        self.get_or_error().assert_called(matcher)
    }
}

impl<M, I, O, B> MockLocator<M, I, O, B>
where
    M: DerefMut<Target = Mocks>,
    I: Send + Sync + 'static,
    O: Clone + Send + Sync + 'static,
{
    /// This makes the mock returns the given constant value.
    /// This requires `Clone`. For returning not clone value, use `returns_with`.
    pub fn returns(&mut self, ret: O) {
        let matcher = self.matcher();
        self.get_mut_or_default().returns(matcher, ret);
    }
}

impl<M, I, O, B> MockLocator<M, I, O, B>
where
    M: DerefMut<Target = Mocks>,
    I: Send + Sync + 'static,
    O: Send + Sync + 'static,
{
    fn get_mut_or_default(&mut self) -> &mut Mock<I, O> {
        self.mocks.get_mut_or_create::<I, O>(self.name)
    }

    fn get_or_error(&self) -> &Mock<I, O> {
        self.mocks
            .get::<Mock<I, O>>(self.name)
            .expect(&format!("no mock is found for {}", self.name))
    }

    fn matcher(&mut self) -> Matcher<I> {
        self.matcher.take().unwrap()
    }
}
