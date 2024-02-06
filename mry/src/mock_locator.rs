pub mod times;

use std::any::TypeId;
use std::marker::PhantomData;
use std::sync::Arc;

use parking_lot::Mutex;

use crate::{mock::Logs, Behavior, Matcher, MockGetter};

use self::times::Times;

/// Mock locator returned by mock_* methods
pub struct MockLocator<I, O, B> {
    pub(crate) mocks: Arc<Mutex<dyn MockGetter<I, O>>>,
    pub(crate) key: TypeId,
    pub(crate) name: &'static str,
    pub(crate) matcher: Arc<Mutex<Matcher<I>>>,
    pub(crate) logs: Option<Arc<Mutex<Logs<I>>>>,
    #[allow(clippy::type_complexity)]
    _phantom: PhantomData<fn() -> (I, O, B)>,
}

impl<I, O, B> MockLocator<I, O, B> {
    #[doc(hidden)]
    pub fn new(
        mocks: Arc<Mutex<dyn MockGetter<I, O>>>,
        key: TypeId,
        name: &'static str,
        matcher: Matcher<I>,
    ) -> Self {
        Self {
            mocks,
            key,
            name,
            matcher: matcher.wrapped(),
            logs: None,
            _phantom: Default::default(),
        }
    }
}

macro_rules! get_mut_or_default {
    ($self:ident, $mock:ident) => {
        let mut lock = $self.mocks.lock();
        let $mock = lock.get_mut_or_create($self.key, $self.name);
        if $self.logs.is_none() {
            $self.logs = Some($mock.logs.clone());
        }
    };
}

impl<I, O, B> MockLocator<I, O, B>
where
    I: Clone + PartialEq + Send + 'static,
    O: Send + 'static,
    B: Into<Behavior<I, O>>,
{
    /// Returns value with using a closure.
    /// Arguments of a method call are passed to the given closure.
    pub fn returns_with<T: Into<B>>(&mut self, behavior: T) {
        get_mut_or_default!(self, mock);
        mock.returns_with(self.matcher.clone(), behavior.into().into());
    }

    /// Returns value once. After that, it panics.
    pub fn returns_once(&mut self, ret: O) {
        get_mut_or_default!(self, mock);
        mock.returns_once(self.matcher.clone(), ret);
    }
}

impl<I, O, B> MockLocator<I, O, B>
where
    I: Clone + PartialEq + Send + 'static,
    O: Send + 'static,
{
    /// This make the mock calls real impl. This is used for partial mocking.
    pub fn calls_real_impl(&mut self) {
        get_mut_or_default!(self, mock);
        mock.calls_real_impl(self.matcher.clone());
    }

    /// Assert the mock is called.
    /// Returns `MockResult` allows to call `times(n)`
    /// Panics if not called
    pub fn assert_called(&mut self, times: impl Into<Times>) {
        self.mocks
            .lock()
            .get(&self.key, self.name)
            .unwrap_or_else(|| panic!("no mock is found for {}", self.name))
            .assert_called(&self.matcher.lock(), times.into());
    }
}

impl<I, O, B> MockLocator<I, O, B>
where
    I: Clone + PartialEq + Send + 'static,
    O: Clone + Send + 'static,
{
    /// This makes the mock returns the given constant value.
    /// This requires `Clone`. For returning not clone value, use `returns_with`.
    pub fn returns(&mut self, ret: O) {
        get_mut_or_default!(self, mock);
        mock.returns(self.matcher.clone(), ret);
    }
}

impl<I, B, R>
    MockLocator<I, std::pin::Pin<Box<dyn std::future::Future<Output = R> + Send + 'static>>, B>
where
    I: Clone + PartialEq + Send + 'static,
    R: Clone + Send + 'static,
{
    /// This makes the mock returns the given constant value with `std::future::ready`.
    /// This requires `Clone`. For returning not clone value, use `returns_with`.
    pub fn returns_ready(&mut self, ret: R) {
        get_mut_or_default!(self, mock);
        mock.returns_ready(self.matcher.clone(), ret);
    }
}

impl<I, B, R>
    MockLocator<I, std::pin::Pin<Box<dyn std::future::Future<Output = R> + Send + 'static>>, B>
where
    I: Clone + PartialEq + Send + 'static,
    R: Send + 'static,
{
    /// This makes the mock returns the given constant value with `std::future::ready`.
    /// This requires `Clone`. For returning not clone value, use `returns_with`.
    pub fn returns_ready_once(&mut self, ret: R) {
        get_mut_or_default!(self, mock);
        mock.returns_ready_once(self.matcher.clone(), ret);
    }
}
