use std::any::{Any, TypeId};
use std::collections::HashMap;
use std::ops::{Deref, DerefMut};
use std::sync::Arc;

use parking_lot::Mutex;

use crate::mock::Mock;
use crate::mockable::{MockableArg, MockableRet};

type BoxAnySend = Box<dyn Any + Send>;

#[doc(hidden)]
pub trait MockGetter<I, O> {
    fn get(&self, key: &TypeId, name: &'static str) -> Option<&Mock<I, O>>;
    fn get_mut_or_create(&mut self, key: TypeId, name: &'static str) -> &mut Mock<I, O>;
}

impl<I, O, T, M: 'static> MockGetter<I, O> for T
where
    T: DerefMut<Target = M> + Deref<Target = M>,
    M: MockGetter<I, O>,
{
    #[track_caller] // get may panic if the lock for static mocks is not acquired
    fn get<'a>(&'a self, key: &TypeId, name: &'static str) -> Option<&'a Mock<I, O>> {
        self.deref().get(key, name)
    }

    #[track_caller] // get_mut_or_create may panic if the lock for static mocks is not acquired
    fn get_mut_or_create(&mut self, key: TypeId, name: &'static str) -> &mut Mock<I, O> {
        self.deref_mut().get_mut_or_create(key, name)
    }
}

#[derive(Default)]
#[doc(hidden)]
pub struct Mocks {
    pub(crate) mock_objects: HashMap<TypeId, BoxAnySend>,
}

impl<I: MockableArg, O: MockableRet> MockGetter<I, O> for Mocks {
    fn get(&self, key: &TypeId, _name: &'static str) -> Option<&Mock<I, O>> {
        self.mock_objects
            .get(key)
            .map(|mock| mock.downcast_ref().unwrap())
    }

    fn get_mut_or_create(&mut self, key: TypeId, name: &'static str) -> &mut Mock<I, O> {
        self.mock_objects
            .entry(key)
            .or_insert(Box::new(Mock::<I, O>::new(name)))
            .downcast_mut()
            .unwrap()
    }
}

impl Mocks {
    #[track_caller]
    #[doc(hidden)]
    pub fn record_call_and_find_mock_output<I: MockableArg, O: MockableRet>(
        &mut self,
        key: TypeId,
        name: &'static str,
        input: I,
    ) -> Option<O> {
        let mock = self.get_mut_or_create(key, name);
        let result = mock.find_mock_output(&input);
        mock.record_call(Arc::new(Mutex::new(input)));
        result
    }

    #[cfg(test)]
    pub(crate) fn insert<I: MockableArg, O: MockableRet>(&mut self, key: TypeId, item: Mock<I, O>) {
        self.mock_objects.insert(key, Box::new(item));
    }

    pub(crate) fn remove(&mut self, key: &TypeId) -> Option<()> {
        self.mock_objects.remove(key).map(|_| ())
    }
}

#[cfg(test)]
mod test {
    use crate::{Behavior, Matcher};

    use super::*;

    #[test]
    fn get_returns_none() {
        let mock_data = Mocks::default();
        assert!(
            MockGetter::<usize, usize>::get(&mock_data, &TypeId::of::<usize>(), "meow").is_none()
        );
    }

    #[test]
    fn get_returns_an_item() {
        let mut mock_data = Mocks::default();
        mock_data.insert(TypeId::of::<usize>(), Mock::<usize, usize>::new(""));
        assert!(
            MockGetter::<usize, usize>::get(&mock_data, &TypeId::of::<usize>(), "name").is_some()
        );
    }

    #[test]
    fn get_mut_or_create_returns_an_item() {
        let mut mock_data = Mocks::default();
        let mut mock = Mock::<u8, u8>::new("a");
        mock.returns_with(
            Matcher::any().wrapped(),
            Behavior::Function {
                call: Box::new(|_| 4u8),
                clone: Clone::clone,
            },
        );
        mock_data.insert(TypeId::of::<usize>(), mock);
        assert_eq!(
            mock_data
                .get_mut_or_create(TypeId::of::<usize>(), "meow")
                .find_mock_output(&1u8),
            Some(4u8)
        );
    }

    #[test]
    // should not panic
    fn get_mut_or_create_returns_default() {
        let mut mock_data = Mocks::default();

        MockGetter::<usize, usize>::get_mut_or_create(
            &mut mock_data,
            TypeId::of::<usize>(),
            "meow",
        );
    }
}
