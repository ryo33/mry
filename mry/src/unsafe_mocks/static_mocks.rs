use std::{any::TypeId, cell::RefCell, collections::HashMap, ops::Deref, rc::Rc};

use super::{
    mock::UnsafeMock,
    mockable::{UnsafeMockableArg, UnsafeMockableRet},
    mocks::{UnsafeMockGetter, UnsafeMocks},
};

thread_local! {
    pub static STATIC_UNSAFE_MOCKS: Rc<RefCell<UnsafeStaticMocks>> = Rc::new(RefCell::new(UnsafeStaticMocks::default()));
}

thread_local! {
    pub static STATIC_UNSAFE_MOCK_LOCKS: RefCell<HashMap<TypeId, Rc<RefCell<()>>>> = RefCell::new(HashMap::new());
}

#[doc(hidden)]
pub fn get_static_mocks() -> Rc<RefCell<UnsafeStaticMocks>> {
    STATIC_UNSAFE_MOCKS.with(Clone::clone)
}

#[doc(hidden)]
pub fn static_record_call_and_find_mock_output<I: UnsafeMockableArg, O: UnsafeMockableRet>(
    key: TypeId,
    name: &'static str,
    input: I,
) -> Option<O> {
    STATIC_UNSAFE_MOCKS.with(|mocks| {
        mocks
            .borrow_mut()
            .record_call_and_find_mock_output(key, name, input)
    })
}

#[doc(hidden)]
pub struct UnsafeStaticMockCell {
    pub key: TypeId,
    pub name: String,
    pub refcell: Rc<RefCell<()>>,
}

#[doc(hidden)]
pub struct UnsafeStaticMockLock<'a> {
    pub key: TypeId,
    pub name: String,
    pub lock: Box<dyn Deref<Target = ()> + 'a>,
}

impl Drop for UnsafeStaticMockLock<'_> {
    fn drop(&mut self) {
        let mocks = STATIC_UNSAFE_MOCKS.with(Clone::clone);
        if mocks.borrow_mut().0.remove(&self.key).is_none() {
            panic!(
                "{} is locked but no used. Remove {} from mry::lock",
                self.name, self.name
            );
        };
    }
}

#[doc(hidden)]
#[derive(Default)]
pub struct UnsafeStaticMocks(UnsafeMocks);

fn check_locked(key: &TypeId) -> bool {
    STATIC_UNSAFE_MOCK_LOCKS.with(|locks| {
        locks
            .borrow()
            .get(key)
            .map(|lock| lock.try_borrow_mut().is_err())
            .unwrap_or(false)
    })
}

impl<I: UnsafeMockableArg, O: UnsafeMockableRet> UnsafeMockGetter<I, O> for UnsafeStaticMocks {
    fn get(&self, key: &TypeId, name: &'static str) -> Option<&UnsafeMock<I, O>> {
        if !check_locked(key) {
            panic!(
                "the lock of `{name}` is not acquired. Try `#[mry::lock({name})]`",
                name = name
            );
        }
        self.0.get(key, name)
    }

    fn get_mut_or_create(&mut self, key: TypeId, name: &'static str) -> &mut UnsafeMock<I, O> {
        if !check_locked(&key) {
            panic!(
                "the lock of `{name}` is not acquired. Try `#[mry::lock({name})]`",
                name = name
            );
        }
        self.0.get_mut_or_create(key, name)
    }
}

impl UnsafeStaticMocks {
    pub fn record_call_and_find_mock_output<I: UnsafeMockableArg, O: UnsafeMockableRet>(
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
pub fn __mutexes(mut keys: Vec<(TypeId, String)>) -> Vec<UnsafeStaticMockCell> {
    // Prevent deadlock by sorting the keys.
    keys.sort();
    keys.into_iter()
        .map(|(key, name)| UnsafeStaticMockCell {
            key,
            name,
            refcell: STATIC_UNSAFE_MOCK_LOCKS.with(|locks| {
                locks
                    .borrow_mut()
                    .entry(key)
                    .or_insert(Rc::new(Default::default()))
                    .clone()
            }),
        })
        .collect()
}

#[doc(hidden)]
pub fn __lock_and_run<T>(mut mutexes: Vec<UnsafeStaticMockCell>, function: fn() -> T) -> T {
    if let Some(mutex) = mutexes.pop() {
        let _lock = UnsafeStaticMockLock {
            key: mutex.key,
            name: mutex.name,
            lock: Box::new(mutex.refcell.borrow_mut()),
        };
        __lock_and_run(mutexes, function)
    } else {
        function()
    }
}

// #[doc(hidden)]
// #[async_recursion(?Send)]
// pub async fn __async_lock_and_run<T>(
//     mut mutexes: Vec<StaticMockMutex>,
//     function: fn() -> Pin<Box<dyn Future<Output = T>>>,
// ) -> T {
//     if let Some(mutex) = mutexes.pop() {
//         let _lock = StaticMockLock {
//             key: mutex.key,
//             name: mutex.name,
//             lock: Box::new(mutex.mutex.lock()),
//         };
//         __async_lock_and_run(mutexes, function).await
//     } else {
//         function().await
//     }
// }

#[cfg(test)]
mod tests {
    use std::any::Any;

    use crate::unsafe_mocks::matcher::UnsafeMatcher;

    use super::*;

    #[test]
    fn returns_none_if_not_mocked() {
        insert_lock(
            returns_none_if_not_mocked.type_id(),
            Rc::new(Default::default()),
        );

        assert_eq!(
            STATIC_UNSAFE_MOCKS.with(|mocks| mocks
                .borrow_mut()
                .record_call_and_find_mock_output::<(), ()>(
                    returns_none_if_not_mocked.type_id(),
                    "meow",
                    ()
                )),
            None
        );
    }

    #[test]
    fn returns_some_if_mocked() {
        let mut mocks = UnsafeMocks::default();
        mocks
            .get_mut_or_create(returns_some_if_mocked.type_id(), "meow")
            .returns(UnsafeMatcher::new_eq(()).wrapped(), ());
        let mut static_mocks = UnsafeStaticMocks(mocks);

        let mutex: Rc<_> = Rc::new(RefCell::default());
        let _lock = mutex.borrow_mut();

        insert_lock(returns_some_if_mocked.type_id(), mutex.clone());

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
        let mocks = STATIC_UNSAFE_MOCKS.with(Clone::clone);
        UnsafeMockGetter::<(), ()>::get(
            mocks.borrow().deref(),
            &panic_if_lock_is_not_created.type_id(),
            "meow",
        );
    }

    #[test]
    #[should_panic(expected = "the lock of `meow` is not acquired.")]
    fn panic_if_lock_is_not_created_mut() {
        let mocks = STATIC_UNSAFE_MOCKS.with(Clone::clone);
        UnsafeMockGetter::<(), ()>::get_mut_or_create(
            &mut mocks.borrow_mut(),
            panic_if_lock_is_not_created_mut.type_id(),
            "meow",
        );
    }

    #[test]
    #[should_panic(expected = "the lock of `meow` is not acquired.")]
    fn panic_if_lock_is_not_acquired() {
        insert_lock(
            panic_if_lock_is_not_acquired.type_id(),
            Rc::new(Default::default()),
        );
        let mocks = STATIC_UNSAFE_MOCKS.with(Clone::clone);
        UnsafeMockGetter::<(), ()>::get(
            mocks.borrow().deref(),
            &panic_if_lock_is_not_acquired.type_id(),
            "meow",
        );
    }

    #[test]
    #[should_panic(expected = "the lock of `meow` is not acquired.")]
    fn panic_if_lock_is_not_acquired_mut() {
        insert_lock(
            panic_if_lock_is_not_acquired_mut.type_id(),
            Rc::new(Default::default()),
        );
        let mocks = STATIC_UNSAFE_MOCKS.with(Clone::clone);
        UnsafeMockGetter::<(), ()>::get_mut_or_create(
            &mut mocks.borrow_mut(),
            panic_if_lock_is_not_acquired_mut.type_id(),
            "meow",
        );
    }

    #[test]
    fn delete_mock_when_lock_is_dropped() {
        insert_mock(
            delete_mock_when_lock_is_dropped.type_id(),
            UnsafeMock::<usize, usize>::new(""),
        );

        drop(UnsafeStaticMockLock {
            key: delete_mock_when_lock_is_dropped.type_id(),
            name: "name".to_string(),
            lock: Box::new(Box::new(())),
        });

        let mocks = STATIC_UNSAFE_MOCKS.with(Clone::clone);
        assert!(UnsafeMockGetter::<usize, usize>::get(
            &mocks.borrow().0,
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
        assert!(get_lock(__mutexes_creates_mutexes.type_id()).is_some());

        cleanup_static_mock_lock(__mutexes_creates_mutexes.type_id());
    }

    #[test]
    fn __mutexes_does_not_overwrite_mutexes() {
        let mutexes = __mutexes(vec![(
            __mutexes_does_not_overwrite_mutexes.type_id(),
            "name".to_string(),
        )]);

        let _lock = mutexes[0].refcell.try_borrow_mut().unwrap();

        let mutexes = __mutexes(vec![(
            __mutexes_does_not_overwrite_mutexes.type_id(),
            "name".to_string(),
        )]);

        assert!(mutexes[0].refcell.try_borrow_mut().is_err());

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
        insert_mock(a.type_id(), UnsafeMock::<usize, usize>::new(""));

        insert_mock(b.type_id(), UnsafeMock::<usize, usize>::new(""));

        let mutexes = __mutexes(vec![(a.type_id(), "a".into()), (b.type_id(), "b".into())]);
        __lock_and_run(mutexes, || {
            assert!(get_lock(a.type_id()).unwrap().try_borrow_mut().is_err());

            assert!(get_lock(b.type_id()).unwrap().try_borrow_mut().is_err());
        });
    }

    #[test]
    fn __lock_and_run_delete_mocks_on_free() {
        fn a() {}
        fn b() {}
        insert_mock(a.type_id(), UnsafeMock::<usize, usize>::new(""));

        insert_mock(b.type_id(), UnsafeMock::<usize, usize>::new(""));

        __lock_and_run(
            __mutexes(vec![(a.type_id(), "a".into()), (b.type_id(), "b".into())]),
            || {
                let mocks = STATIC_UNSAFE_MOCKS.with(Clone::clone);
                assert!(UnsafeMockGetter::<usize, usize>::get(
                    mocks.borrow().deref(),
                    &a.type_id(),
                    "a"
                )
                .is_some());

                assert!(UnsafeMockGetter::<usize, usize>::get(
                    mocks.borrow().deref(),
                    &b.type_id(),
                    "b"
                )
                .is_some());
            },
        );

        let mocks = STATIC_UNSAFE_MOCKS.with(Clone::clone);

        assert!(
            UnsafeMockGetter::<usize, usize>::get(&mocks.borrow().0, &a.type_id(), "a").is_none()
        );

        assert!(
            UnsafeMockGetter::<usize, usize>::get(&mocks.borrow().0, &b.type_id(), "b").is_none()
        );
    }

    fn insert_mock<I: UnsafeMockableArg, O: UnsafeMockableRet>(
        key: TypeId,
        mock: UnsafeMock<I, O>,
    ) {
        STATIC_UNSAFE_MOCKS.with(|mocks| mocks.borrow_mut().0.insert(key, mock));
    }

    fn insert_lock(key: TypeId, lock: Rc<RefCell<()>>) {
        STATIC_UNSAFE_MOCK_LOCKS.with(|locks| {
            locks.borrow_mut().insert(key, lock);
        });
    }

    fn get_lock(key: TypeId) -> Option<Rc<RefCell<()>>> {
        STATIC_UNSAFE_MOCK_LOCKS.with(|locks| locks.borrow().get(&key).cloned())
    }

    fn cleanup_static_mock_lock(key: TypeId) {
        STATIC_UNSAFE_MOCK_LOCKS.with(|locks| locks.borrow_mut().remove(&key));
    }
}
