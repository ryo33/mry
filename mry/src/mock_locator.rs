use std::fmt::Debug;
use std::marker::PhantomData;

use once_cell::sync::Lazy;
use parking_lot::RwLock;

use crate::{Behavior, Matcher, Mock, MockObjects, MockResult, Mry};

pub static MOCK_DATA: Lazy<RwLock<MockObjects>> = Lazy::new(|| RwLock::new(MockObjects::default()));

pub struct MockLocator<'a, I, O, B> {
    pub id: &'a Mry,
    pub name: &'static str,
    pub _phantom: PhantomData<fn() -> (I, O, B)>,
}

impl<'a, I, O, B> MockLocator<'a, I, O, B>
where
    I: Clone + PartialEq + Debug + Send + Sync + 'static,
    O: Clone + Send + Sync + 'static,
    B: Into<Behavior<I, O>>,
{
    pub fn returns_with<T: Into<B>>(&self, behavior: T) {
        let mut lock = MOCK_DATA.write();
        self.get_mut_or_default(&mut lock)
            .returns_with(behavior.into());
    }

    pub fn returns_when_with<M: Into<Matcher<I>>, T: Into<B>>(&self, matcher: M, behavior: T) {
        let mut lock = MOCK_DATA.write();
        self.get_mut_or_default(&mut lock)
            .returns_when_with(matcher, behavior.into());
    }

    pub fn returns(&self, ret: O) {
        let mut lock = MOCK_DATA.write();
        self.get_mut_or_default(&mut lock).returns(ret);
    }

    pub fn returns_when<M: Into<Matcher<I>>>(&self, matcher: M, ret: O) {
        let mut lock = MOCK_DATA.write();
        self.get_mut_or_default(&mut lock)
            .returns_when(matcher, ret);
    }

    pub fn calls_real_impl(&self) {
        let mut lock = MOCK_DATA.write();
        self.get_mut_or_default(&mut lock).calls_real_impl();
    }

    pub fn asserts_called_with<M: Into<Matcher<I>> + std::fmt::Debug>(
        &self,
        matcher: M,
    ) -> MockResult<I> {
        let lock = MOCK_DATA.read();
        self.get_or_error(&lock).asserts_called_with(matcher)
    }

    pub fn asserts_called(&self) -> MockResult<I> {
        let lock = MOCK_DATA.read();
        self.get_or_error(&lock).asserts_called()
    }

    fn get_mut_or_default(&self, lock: &'a mut MockObjects) -> &'a mut Mock<I, O> {
        lock.get_mut_or_create::<I, O>(self.id, self.name)
    }

    fn get_or_error(&self, lock: &'a MockObjects) -> &'a Mock<I, O> {
        lock.get::<Mock<I, O>>(self.id, self.name)
            .expect(&format!("no mock is found for {}", self.name))
    }
}
