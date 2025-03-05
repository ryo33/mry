use std::any::{Any, TypeId};
use std::cell::RefCell;
use std::collections::HashMap;
use std::ops::{Deref, DerefMut};
use std::rc::Rc;

use super::mock::UnsafeMock;
use super::mockable::{UnsafeMockableArg, UnsafeMockableRet};

type BoxAny = Box<dyn Any>;

#[doc(hidden)]
pub trait UnsafeMockGetter<I, O> {
    fn get(&self, key: &TypeId, name: &'static str) -> Option<&UnsafeMock<I, O>>;
    fn get_mut_or_create(&mut self, key: TypeId, name: &'static str) -> &mut UnsafeMock<I, O>;
}

impl<I, O, T, M: 'static> UnsafeMockGetter<I, O> for T
where
    T: DerefMut<Target = M> + Deref<Target = M>,
    M: UnsafeMockGetter<I, O>,
{
    fn get<'a>(&'a self, key: &TypeId, name: &'static str) -> Option<&'a UnsafeMock<I, O>> {
        self.deref().get(key, name)
    }

    fn get_mut_or_create(&mut self, key: TypeId, name: &'static str) -> &mut UnsafeMock<I, O> {
        self.deref_mut().get_mut_or_create(key, name)
    }
}

#[derive(Default)]
#[doc(hidden)]
pub struct UnsafeMocks {
    pub(crate) mock_objects: HashMap<TypeId, BoxAny>,
}

impl<I: UnsafeMockableArg, O: UnsafeMockableRet> UnsafeMockGetter<I, O> for UnsafeMocks {
    fn get(&self, key: &TypeId, _name: &'static str) -> Option<&UnsafeMock<I, O>> {
        self.mock_objects
            .get(key)
            .map(|mock| mock.downcast_ref().unwrap())
    }

    fn get_mut_or_create(&mut self, key: TypeId, name: &'static str) -> &mut UnsafeMock<I, O> {
        self.mock_objects
            .entry(key)
            .or_insert(Box::new(UnsafeMock::<I, O>::new(name)))
            .downcast_mut()
            .unwrap()
    }
}

impl UnsafeMocks {
    #[doc(hidden)]
    pub fn record_call_and_find_mock_output<I: UnsafeMockableArg, O: UnsafeMockableRet>(
        &mut self,
        key: TypeId,
        name: &'static str,
        input: I,
    ) -> Option<O> {
        let mock = self.get_mut_or_create(key, name);
        let result = mock.find_mock_output(&input);
        mock.record_call(Rc::new(RefCell::new(input)));
        result
    }

    #[cfg(test)]
    pub(crate) fn insert<I: UnsafeMockableArg, O: UnsafeMockableRet>(
        &mut self,
        key: TypeId,
        item: UnsafeMock<I, O>,
    ) {
        self.mock_objects.insert(key, Box::new(item));
    }

    pub(crate) fn remove(&mut self, key: &TypeId) -> Option<()> {
        self.mock_objects.remove(key).map(|_| ())
    }
}

#[cfg(test)]
mod test {
    use crate::unsafe_mocks::{behavior::UnsafeBehavior, matcher::UnsafeMatcher};

    use super::*;

    #[test]
    fn get_returns_none() {
        let mock_data = UnsafeMocks::default();
        assert!(
            UnsafeMockGetter::<usize, usize>::get(&mock_data, &TypeId::of::<usize>(), "meow")
                .is_none()
        );
    }

    #[test]
    fn get_returns_an_item() {
        let mut mock_data = UnsafeMocks::default();
        mock_data.insert(TypeId::of::<usize>(), UnsafeMock::<usize, usize>::new(""));
        assert!(
            UnsafeMockGetter::<usize, usize>::get(&mock_data, &TypeId::of::<usize>(), "name")
                .is_some()
        );
    }

    #[test]
    fn get_mut_or_create_returns_an_item() {
        let mut mock_data = UnsafeMocks::default();
        let mut mock = UnsafeMock::<u8, u8>::new("a");
        mock.returns_with(
            UnsafeMatcher::any().wrapped(),
            UnsafeBehavior::Function {
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
        let mut mock_data = UnsafeMocks::default();

        UnsafeMockGetter::<usize, usize>::get_mut_or_create(
            &mut mock_data,
            TypeId::of::<usize>(),
            "meow",
        );
    }
}
