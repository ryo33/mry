use std::fmt::Debug;
use std::marker::PhantomData;

use once_cell::sync::Lazy;
use parking_lot::Mutex;

use crate::Matcher;
use crate::{Behavior, Mock, MockObjects, Mry};

pub static MOCK_DATA: Lazy<Mutex<MockObjects>> = Lazy::new(|| Mutex::new(MockObjects::default()));

pub struct MockLocator<'a, I, O, B> {
    pub id: &'a Mry,
    pub name: &'static str,
    pub _phantom: PhantomData<fn() -> (I, O, B)>,
}

impl<'a, I, O, B> MockLocator<'a, I, O, B>
where
    I: Clone + PartialEq + Debug + Send + 'static,
    O: Default + 'static,
    B: Into<Behavior<I, O>>,
{
    pub fn behaves<T: Into<B>>(&self, behavior: T) {
        let mut lock = MOCK_DATA.lock();
        self.get_mut_or_default(&mut lock).behaves(behavior.into());
    }

    pub fn behaves_when<M: Into<Matcher<I>>, T: Into<B>>(&self, matcher: M, behavior: T) {
        let mut lock = MOCK_DATA.lock();
        self.get_mut_or_default(&mut lock)
            .behaves_when(matcher, behavior.into());
    }

    pub fn assert_called_with<M: Into<Matcher<I>> + std::fmt::Debug>(&self, matcher: M) {
        let lock = MOCK_DATA.lock();
        self.get_or_error(&lock).assert_called_with(matcher)
    }

    pub fn assert_called(&self) {
        let lock = MOCK_DATA.lock();
        self.get_or_error(&lock).assert_called()
    }

    fn get_mut_or_default(&self, lock: &'a mut MockObjects) -> &'a mut Mock<I, O> {
        lock.get_mut_or_create::<I, O>(self.id, self.name)
    }

    fn get_or_error(&self, lock: &'a MockObjects) -> &'a Mock<I, O> {
        lock.get::<Mock<I, O>>(self.id, self.name)
            .expect(&format!("no mock is found for {}", self.name))
    }
}
