pub mod times;

use std::any::TypeId;
use std::marker::PhantomData;
use std::sync::Arc;

use parking_lot::Mutex;

use crate::{mockable::MockableRet, Behavior, Matcher, MockGetter};

use self::times::Times;

/// Mock locator returned by mock_* methods
pub struct MockLocator<I, O, B> {
    pub(crate) mocks: Arc<Mutex<dyn MockGetter<I, O>>>,
    pub(crate) key: TypeId,
    pub(crate) name: &'static str,
    pub(crate) matcher: Arc<Mutex<Matcher<I>>>,
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
            matcher: Arc::new(Mutex::new(matcher)),
            _phantom: Default::default(),
        }
    }
}

macro_rules! get_mut_or_default {
    ($self:ident) => {
        $self.mocks.lock().get_mut_or_create($self.key, $self.name)
    };
}

impl<I, O, B> MockLocator<I, O, B>
where
    I: 'static,
    O: 'static,
    B: Into<Behavior<I, O>>,
{
    /// Returns value with using a closure.
    /// Arguments of a method call are passed to the given closure.
    pub fn returns_with<T: Into<B>>(self, behavior: T) -> Self {
        get_mut_or_default!(self).returns_with(self.matcher.clone(), behavior.into().into());
        self
    }

    /// Returns value once. After that, it panics.
    pub fn returns_once(self, ret: O) -> Self {
        get_mut_or_default!(self).returns_once(self.matcher.clone(), ret);
        self
    }
}

impl<I, O, B> MockLocator<I, O, B>
where
    I: 'static,
    O: 'static,
{
    /// This make the mock calls real impl. This is used for partial mocking.
    pub fn calls_real_impl(self) -> Self {
        get_mut_or_default!(self).calls_real_impl(self.matcher.clone());
        self
    }

    /// Assert the mock is called.
    /// Returns `MockResult` allows to call `times(n)`
    /// Panics if not called
    pub fn assert_called(&self, times: impl Into<Times>) {
        get_mut_or_default!(self).assert_called(&self.matcher.lock(), times.into());
    }
}

impl<I, O, B> MockLocator<I, O, B>
where
    I: 'static,
    O: Clone + MockableRet,
{
    /// This makes the mock returns the given constant value.
    /// This requires `Clone`. For returning not clone value, use `returns_once`.
    pub fn returns(self, ret: O) -> Self {
        get_mut_or_default!(self).returns(self.matcher.clone(), ret);
        self
    }
}
