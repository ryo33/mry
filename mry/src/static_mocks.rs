use crate::{mock::Mock, MockGetter, Mocks};
use async_recursion::async_recursion;
use once_cell::sync::Lazy;
use parking_lot::Mutex;
use std::{
    any::TypeId, collections::HashMap, fmt::Debug, future::Future, ops::Deref, pin::Pin, sync::Arc,
};

pub static STATIC_MOCKS: Lazy<Arc<Mutex<StaticMocks>>> =
    Lazy::new(|| Arc::new(Mutex::new(StaticMocks::default())));

pub static STATIC_MOCK_LOCKS: Lazy<Mutex<HashMap<TypeId, Arc<Mutex<()>>>>> =
    Lazy::new(|| Mutex::new(HashMap::new()));

#[doc(hidden)]
pub struct StaticMockMutex {
    pub key: TypeId,
    pub name: String,
    pub mutex: Arc<Mutex<()>>,
}

#[doc(hidden)]
pub struct StaticMockLock<'a> {
    pub key: TypeId,
    pub name: String,
    pub lock: Box<dyn Deref<Target = ()> + 'a>,
}

impl<'a> Drop for StaticMockLock<'a> {
    fn drop(&mut self) {
        let lock = &mut STATIC_MOCKS.lock().0;
        if lock.remove(&self.key).is_none() {
            panic!(
                "{} is locked but no used. Remove {} from mry::lock",
                self.name, self.name
            );
        };
    }
}

#[doc(hidden)]
#[derive(Default)]
pub struct StaticMocks(Mocks);

fn check_locked(key: &TypeId) -> bool {
    STATIC_MOCK_LOCKS
        .lock()
        .get(key)
        .map(|lock| lock.try_lock().is_none())
        .unwrap_or(false)
}

impl<I: Send + 'static, O: Send + 'static> MockGetter<I, O> for StaticMocks {
    fn get(&self, key: &TypeId, name: &'static str) -> Option<&Mock<I, O>> {
        if !check_locked(key) {
            panic!("the lock of `{}` is not acquired. See `mry::lock`.", name);
        }
        self.0.get(key, name)
    }

    fn get_mut_or_create(&mut self, key: TypeId, name: &'static str) -> &mut Mock<I, O> {
        if !check_locked(&key) {
            panic!("the lock of `{}` is not acquired. See `mry::lock`.", name);
        }
        self.0.get_mut_or_create(key, name)
    }
}

impl StaticMocks {
    pub fn record_call_and_find_mock_output<
        I: PartialEq + Debug + Clone + Send + 'static,
        O: Debug + Send + 'static,
    >(
        &mut self,
        key: TypeId,
        name: &'static str,
        input: I,
    ) -> Option<O> {
        if check_locked(&key) {
            self.0.record_call_and_find_mock_output(key, name, input)
        } else {
            None
        }
    }
}

#[doc(hidden)]
pub fn __mutexes(mut keys: Vec<(TypeId, String)>) -> Vec<StaticMockMutex> {
    // Prevent deadlock by sorting the keys.
    keys.sort();
    keys.into_iter()
        .map(|(key, name)| StaticMockMutex {
            key,
            name,
            mutex: STATIC_MOCK_LOCKS
                .lock()
                .entry(key)
                .or_insert(Arc::new(Default::default()))
                .clone(),
        })
        .collect()
}

#[doc(hidden)]
pub fn __lock_and_run<T>(mut mutexes: Vec<StaticMockMutex>, function: fn() -> T) -> T {
    if let Some(mutex) = mutexes.pop() {
        let _lock = StaticMockLock {
            key: mutex.key,
            name: mutex.name,
            lock: Box::new(mutex.mutex.lock()),
        };
        __lock_and_run(mutexes, function)
    } else {
        function()
    }
}

#[doc(hidden)]
#[async_recursion(?Send)]
pub async fn __async_lock_and_run<T>(
    mut mutexes: Vec<StaticMockMutex>,
    function: fn() -> Pin<Box<dyn Future<Output = T>>>,
) -> T {
    if let Some(mutex) = mutexes.pop() {
        let _lock = StaticMockLock {
            key: mutex.key,
            name: mutex.name,
            lock: Box::new(mutex.mutex.lock()),
        };
        __async_lock_and_run(mutexes, function).await
    } else {
        function().await
    }
}

#[cfg(test)]
mod tests {
    use std::any::Any;

    use crate::{mock::Mock, Matcher, MockGetter};

    use super::*;

    #[test]
    fn returns_none_if_not_mocked() {
        STATIC_MOCK_LOCKS.lock().insert(
            returns_none_if_not_mocked.type_id(),
            Arc::new(Default::default()),
        );

        assert_eq!(
            STATIC_MOCKS
                .lock()
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
            .returns(Matcher::Eq(()).wrapped(), ());
        let mut static_mocks = StaticMocks(mocks);

        let mutex = Arc::new(Mutex::default());
        let _lock = mutex.lock();

        STATIC_MOCK_LOCKS
            .lock()
            .insert(returns_some_if_mocked.type_id(), mutex.clone());

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
    fn panic_if_lock_is_not_created() {
        MockGetter::<(), ()>::get(
            &STATIC_MOCKS.lock(),
            &panic_if_lock_is_not_created.type_id(),
            "meow",
        );
    }

    #[test]
    #[should_panic(expected = "the lock of `meow` is not acquired.")]
    fn panic_if_lock_is_not_created_mut() {
        MockGetter::<(), ()>::get_mut_or_create(
            &mut STATIC_MOCKS.lock(),
            panic_if_lock_is_not_created_mut.type_id(),
            "meow",
        );
    }

    #[test]
    #[should_panic(expected = "the lock of `meow` is not acquired.")]
    fn panic_if_lock_is_not_acquired() {
        STATIC_MOCK_LOCKS.lock().insert(
            panic_if_lock_is_not_acquired.type_id(),
            Arc::new(Default::default()),
        );
        MockGetter::<(), ()>::get(
            &STATIC_MOCKS.lock(),
            &panic_if_lock_is_not_acquired.type_id(),
            "meow",
        );
    }

    #[test]
    #[should_panic(expected = "the lock of `meow` is not acquired.")]
    fn panic_if_lock_is_not_acquired_mut() {
        STATIC_MOCK_LOCKS.lock().insert(
            panic_if_lock_is_not_acquired_mut.type_id(),
            Arc::new(Default::default()),
        );
        MockGetter::<(), ()>::get_mut_or_create(
            &mut STATIC_MOCKS.lock(),
            panic_if_lock_is_not_acquired_mut.type_id(),
            "meow",
        );
    }

    #[test]
    fn delete_mock_when_lock_is_dropped() {
        STATIC_MOCKS.lock().0.insert(
            delete_mock_when_lock_is_dropped.type_id(),
            Mock::<usize, usize>::new(""),
        );

        drop(StaticMockLock {
            key: delete_mock_when_lock_is_dropped.type_id(),
            name: "name".to_string(),
            lock: Box::new(Box::new(())),
        });

        assert!(MockGetter::<usize, usize>::get(
            &STATIC_MOCKS.lock().0,
            &delete_mock_when_lock_is_dropped.type_id(),
            "meow"
        )
        .is_none());
    }

    #[test]
    fn __mutexes_creates_mutexes() {
        let mutexes = __mutexes(vec![(
            __mutexes_creates_mutexes.type_id(),
            "name".to_string(),
        )]);

        assert_eq!(mutexes.len(), 1);
        assert!(STATIC_MOCK_LOCKS
            .lock()
            .get(&__mutexes_creates_mutexes.type_id())
            .is_some());

        cleanup_static_mock_lock(__mutexes_creates_mutexes.type_id());
    }

    #[test]
    fn __mutexes_does_not_overwrite_mutexes() {
        let mutexes = __mutexes(vec![(
            __mutexes_does_not_overwrite_mutexes.type_id(),
            "name".to_string(),
        )]);

        let _lock = mutexes[0].mutex.try_lock().unwrap();

        let mutexes = __mutexes(vec![(
            __mutexes_does_not_overwrite_mutexes.type_id(),
            "name".to_string(),
        )]);

        assert!(mutexes[0].mutex.try_lock().is_none());

        cleanup_static_mock_lock(__mutexes_creates_mutexes.type_id());
    }

    #[test]
    fn __mutexes_sorts_keys() {
        let mutexes = __mutexes(vec![
            (0u16.type_id(), "name".to_string()),
            (0u8.type_id(), "name".to_string()),
            (0u32.type_id(), "name".to_string()),
        ]);

        let mut keys = vec![0u16.type_id(), 0u8.type_id(), 0u32.type_id()];
        keys.sort();
        assert_eq!(mutexes.iter().map(|m| m.key).collect::<Vec<_>>(), keys);
    }

    #[test]
    fn __lock_and_run_just_runs() {
        assert_eq!(__lock_and_run(vec![], || 42), 42)
    }

    #[test]
    fn __lock_and_run_locks() {
        fn a() {}
        fn b() {}
        STATIC_MOCKS
            .lock()
            .0
            .insert(a.type_id(), Mock::<usize, usize>::new(""));

        STATIC_MOCKS
            .lock()
            .0
            .insert(b.type_id(), Mock::<usize, usize>::new(""));

        let mutexes = __mutexes(vec![(a.type_id(), "a".into()), (b.type_id(), "b".into())]);
        __lock_and_run(mutexes, || {
            assert!(STATIC_MOCK_LOCKS
                .lock()
                .get(&a.type_id())
                .unwrap()
                .try_lock()
                .is_none());

            assert!(STATIC_MOCK_LOCKS
                .lock()
                .get(&b.type_id())
                .unwrap()
                .try_lock()
                .is_none());
        });
    }

    #[test]
    fn __lock_and_run_delete_mocks_on_free() {
        fn a() {}
        fn b() {}
        STATIC_MOCKS
            .lock()
            .0
            .insert(a.type_id(), Mock::<usize, usize>::new(""));

        STATIC_MOCKS
            .lock()
            .0
            .insert(b.type_id(), Mock::<usize, usize>::new(""));

        __lock_and_run(
            __mutexes(vec![(a.type_id(), "a".into()), (b.type_id(), "b".into())]),
            || {
                assert!(
                    MockGetter::<usize, usize>::get(&STATIC_MOCKS.lock(), &a.type_id(), "a")
                        .is_some()
                );

                assert!(
                    MockGetter::<usize, usize>::get(&STATIC_MOCKS.lock(), &b.type_id(), "b")
                        .is_some()
                );
            },
        );

        assert!(
            MockGetter::<usize, usize>::get(&STATIC_MOCKS.lock().0, &a.type_id(), "a").is_none()
        );

        assert!(
            MockGetter::<usize, usize>::get(&STATIC_MOCKS.lock().0, &b.type_id(), "b").is_none()
        );
    }

    fn cleanup_static_mock_lock(key: TypeId) {
        STATIC_MOCK_LOCKS.lock().remove(&key);
    }
}
