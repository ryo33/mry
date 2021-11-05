use crate::Mocks;
use once_cell::sync::Lazy;
use parking_lot::{Mutex, RwLock};
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
pub struct StaticMockLock;

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
    use crate::Matcher;

    use super::*;

    #[test]
    fn returns_none_if_not_mocked() {
        let mocks = Mocks::default();
        let mut static_mocks = StaticMocks(mocks);

        assert_eq!(
            static_mocks.record_call_and_find_mock_output::<(), ()>(
                TypeId::of::<usize>(),
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
            .get_mut_or_create(TypeId::of::<usize>(), "meow")
            .returns(Matcher::Eq(()), ());
        let mut static_mocks = StaticMocks(mocks);

        STATIC_MOCK_LOCKS
            .write()
            .insert(TypeId::of::<usize>(), Default::default());

        assert_eq!(
            static_mocks.record_call_and_find_mock_output::<(), ()>(
                TypeId::of::<usize>(),
                "meow",
                ()
            ),
            Some(())
        );
    }
}
