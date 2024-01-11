pub mod times;

use std::any::TypeId;
use std::marker::PhantomData;

use crate::mock::Mock;
use crate::{Behavior, Matcher, MockGetter};

use self::times::Times;

/// Mock locator returned by mock_* methods
pub struct MockLocator<'a, I, O, B> {
    #[doc(hidden)]
    pub mocks: Box<dyn MockGetter<I, O> + 'a>,
    #[doc(hidden)]
    pub key: TypeId,
    #[doc(hidden)]
    pub name: &'static str,
    #[doc(hidden)]
    pub matcher: Option<Matcher<I>>,
    #[doc(hidden)]
    #[allow(clippy::type_complexity)]
    pub _phantom: PhantomData<fn() -> (I, O, B)>,
}

impl<'a, I, O, B> MockLocator<'a, I, O, B>
where
    I: Clone + PartialEq + Send + Sync + 'static,
    O: Send + Sync + 'static,
    B: Into<Behavior<I, O>>,
{
    /// Returns value with using a closure.
    /// Arguments of a method call are passed to the given closure.
    pub fn returns_with<T: Into<B>>(&mut self, behavior: T) {
        let matcher = self.matcher();
        self.get_mut_or_default()
            .returns_with(matcher, behavior.into().into());
    }

    /// Returns value once. After that, it panics.
    pub fn returns_once(&mut self, ret: O) {
        let matcher = self.matcher();
        self.get_mut_or_default().returns_once(matcher, ret);
    }
}

impl<'a, I, O, B> MockLocator<'a, I, O, B>
where
    I: Clone + PartialEq + Send + Sync + 'static,
    O: Send + Sync + 'static,
{
    /// This make the mock calls real impl. This is used for partial mocking.
    pub fn calls_real_impl(&mut self) {
        let matcher = self.matcher();
        self.get_mut_or_default().calls_real_impl(matcher);
    }

    /// Assert the mock is called.
    /// Returns `MockResult` allows to call `times(n)`
    /// Panics if not called
    pub fn assert_called(&mut self, times: impl Into<Times>) -> Vec<I> {
        let matcher = self.matcher.take().unwrap();
        self.get_or_error().assert_called(matcher, times.into()).0
    }
}

impl<'a, I, O, B> MockLocator<'a, I, O, B>
where
    I: Clone + PartialEq + Send + Sync + 'static,
    O: Clone + Send + Sync + 'static,
{
    /// This makes the mock returns the given constant value.
    /// This requires `Clone`. For returning not clone value, use `returns_with`.
    pub fn returns(&mut self, ret: O) {
        let matcher = self.matcher();
        self.get_mut_or_default().returns(matcher, ret);
    }
}

impl<'a, I, B, R>
    MockLocator<
        'a,
        I,
        std::pin::Pin<Box<dyn std::future::Future<Output = R> + Send + Sync + 'static>>,
        B,
    >
where
    I: Clone + PartialEq + Send + Sync + 'static,
    R: Clone + Send + Sync + 'static,
{
    /// This makes the mock returns the given constant value with `std::future::ready`.
    /// This requires `Clone`. For returning not clone value, use `returns_with`.
    pub fn returns_ready(&mut self, ret: R) {
        let matcher = self.matcher();
        self.get_mut_or_default().returns_ready(matcher, ret);
    }
}

impl<'a, I, O, B> MockLocator<'a, I, O, B>
where
    I: Send + Sync + 'static,
    O: Send + Sync + 'static,
{
    fn get_mut_or_default(&mut self) -> &mut Mock<I, O> {
        self.mocks.get_mut_or_create(self.key, self.name)
    }
}
impl<'a, I, O, B> MockLocator<'a, I, O, B>
where
    I: Send + Sync + 'static,
    O: Send + Sync + 'static,
{
    fn get_or_error(&self) -> &Mock<I, O> {
        self.mocks
            .get(&self.key, self.name)
            .unwrap_or_else(|| panic!("no mock is found for {}", self.name))
    }

    fn matcher(&mut self) -> Matcher<I> {
        self.matcher.take().unwrap()
    }
}
