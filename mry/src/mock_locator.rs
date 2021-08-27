use std::fmt::Debug;
use std::marker::PhantomData;
use std::ops::DerefMut;

use crate::{Behavior, Matcher, Mock, MockResult, Mocks};

pub struct MockLocator<M, I, O, B> {
    pub mocks: M,
    pub name: &'static str,
    pub matcher: Option<Matcher<I>>,
    pub _phantom: PhantomData<fn() -> (I, O, B)>,
}

impl<M, I, O, B> MockLocator<M, I, O, B>
where
    M: DerefMut<Target = Mocks>,
    I: Clone + PartialEq + Debug + Send + Sync + 'static,
    O: Clone + Send + Sync + 'static,
    B: Into<Behavior<I, O>>,
{
    pub fn returns_with<T: Into<B>>(&mut self, behavior: T) {
        let matcher = self.matcher.take().unwrap();
        self.get_mut_or_default()
            .returns_with(matcher, behavior.into());
    }

    pub fn returns(&mut self, ret: O) {
        let matcher = self.matcher.take().unwrap();
        self.get_mut_or_default().returns(matcher, ret);
    }

    pub fn calls_real_impl(&mut self) {
        self.get_mut_or_default().calls_real_impl();
    }

    pub fn assert_called(&mut self) -> MockResult<I> {
        let matcher = self.matcher.take().unwrap();
        self.get_or_error().assert_called(matcher)
    }

    fn get_mut_or_default(&mut self) -> &mut Mock<I, O> {
        self.mocks.get_mut_or_create::<I, O>(self.name)
    }

    fn get_or_error(&self) -> &Mock<I, O> {
        self.mocks
            .get::<Mock<I, O>>(self.name)
            .expect(&format!("no mock is found for {}", self.name))
    }
}
