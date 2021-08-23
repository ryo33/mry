use std::fmt::Debug;
use std::marker::PhantomData;

use once_cell::sync::Lazy;

use crate::{Behavior, Matcher, Mock, MockObjects, MockResult, Mry};

pub static MOCK_DATA: Lazy<MockObjects> = Lazy::new(|| MockObjects::default());

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
        MOCK_DATA.get_mut_or_create(self.id, self.name, move |mock| {
            mock.returns_with(behavior.into())
        });
    }

    pub fn returns_when_with<M: Into<Matcher<I>>, T: Into<B>>(&self, matcher: M, behavior: T) {
        MOCK_DATA.get_mut_or_create(self.id, self.name, move |mock| {
            mock.returns_when_with(matcher, behavior.into())
        });
    }

    pub fn returns(&self, ret: O) {
        MOCK_DATA.get_mut_or_create(self.id, self.name, move |mock: &mut Mock<I, O>| {
            mock.returns(ret)
        });
    }

    pub fn returns_when<M: Into<Matcher<I>>>(&self, matcher: M, ret: O) {
        MOCK_DATA.get_mut_or_create(self.id, self.name, move |mock| {
            mock.returns_when(matcher, ret)
        });
    }

    pub fn calls_real_impl(&self) {
        MOCK_DATA.get_mut_or_create(self.id, self.name, |mock: &mut Mock<I, O>| {
            mock.calls_real_impl()
        });
    }

    pub fn asserts_called_with<M: Into<Matcher<I>> + std::fmt::Debug>(
        &self,
        matcher: M,
    ) -> MockResult<I> {
        MOCK_DATA.get_mut_or_create(self.id, self.name, move |mock: &mut Mock<I, O>| {
            mock.asserts_called_with(matcher.into())
        })
    }

    pub fn asserts_called(&self) -> MockResult<I> {
        MOCK_DATA.get_mut_or_create(self.id, self.name, |mock: &mut Mock<I, O>| {
            mock.asserts_called()
        })
    }
}
