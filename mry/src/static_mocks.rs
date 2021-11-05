use crate::Mocks;
use once_cell::sync::Lazy;
use parking_lot::RwLock;
use std::{
    any::TypeId,
    collections::HashMap,
    fmt::Debug,
    ops::{Deref, DerefMut},
};

pub static STATIC_MOCKS: Lazy<RwLock<StaticMocks>> =
    Lazy::new(|| RwLock::new(StaticMocks::default()));

pub static STATIC_MOCK_LOCKS: Lazy<RwLock<HashMap<TypeId, StaticMockMutex>>> =
    Lazy::new(|| RwLock::new(HashMap::new()));

#[doc(hidden)]
pub struct StaticMockLock<T> {
    pub key: TypeId,
    pub lock: T,
}

impl<T> Drop for StaticMockLock<T> {
    fn drop(&mut self) {
        STATIC_MOCKS.write().0.remove(&self.key);
    }
}

#[doc(hidden)]
#[derive(Default)]
pub struct StaticMockMutex {}

#[doc(hidden)]
#[derive(Default)]
pub struct StaticMocks(Mocks);

pub struct DerefMocks<T>(pub T);

impl<T: DerefMut<Target = StaticMocks>> DerefMut for DerefMocks<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0.deref_mut().0
    }
}

impl<T: Deref<Target = StaticMocks>> Deref for DerefMocks<T> {
    type Target = Mocks;

    fn deref(&self) -> &Self::Target {
        &self.0.deref().0
    }
}

impl StaticMocks {
    pub fn record_call_and_find_mock_output<
        I: PartialEq + Debug + Clone + Send + Sync + 'static,
        O: Debug + Send + Sync + 'static,
    >(
        &mut self,
        key: TypeId,
        name: &'static str,
        input: I,
    ) -> Option<O> {
        if STATIC_MOCK_LOCKS.read().contains_key(&key) {
            self.0.record_call_and_find_mock_output(key, name, input)
        } else {
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use std::any::Any;

    use crate::{
        mock::{Mock, MockObjectReturns},
        Matcher,
    };

    use super::*;

    #[test]
    fn returns_none_if_not_mocked() {
        assert_eq!(
            STATIC_MOCKS
                .write()
                .record_call_and_find_mock_output::<(), ()>(
                    returns_none_if_not_mocked.type_id(),
                    "meow",
                    ()
                ),
            None
        );
    }

    #[test]
    fn returns_some_if_mocked() {
        let mut mocks = Mocks::default();
        mocks
            .get_mut_or_create(returns_some_if_mocked.type_id(), "meow")
            .returns(Matcher::Eq(()), ());
        let mut static_mocks = StaticMocks(mocks);

        STATIC_MOCK_LOCKS
            .write()
            .insert(returns_some_if_mocked.type_id(), Default::default());

        assert_eq!(
            static_mocks.record_call_and_find_mock_output::<(), ()>(
                returns_some_if_mocked.type_id(),
                "meow",
                ()
            ),
            Some(())
        );
    }

    #[test]
    #[should_panic(expected = "the lock of `meow` is not acquired.")]
    fn panic_if_lock_is_not_acquired() {
        STATIC_MOCKS
            .write()
            .record_call_and_find_mock_output::<(), ()>(
                panic_if_lock_is_not_acquired.type_id(),
                "meow",
                (),
            );
    }

    #[test]
    fn delete_mock_when_lock_is_dropped() {
        STATIC_MOCKS.write().0.insert(
            delete_mock_when_lock_is_dropped.type_id(),
            Box::new(Mock::<usize, usize>::new("")),
        );

        drop(StaticMockLock {
            key: delete_mock_when_lock_is_dropped.type_id(),
            lock: (),
        });

        assert!(STATIC_MOCKS
            .read()
            .0
            .get::<usize, usize>(&delete_mock_when_lock_is_dropped.type_id())
            .is_none());
    }
}
