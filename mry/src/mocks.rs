use std::any::{Any, TypeId};
use std::collections::HashMap;
use std::fmt::Debug;

use crate::Mock;

type BoxAnySend = Box<dyn Any + Send + Sync>;

#[derive(Default)]
#[doc(hidden)]
pub struct Mocks {
    pub(crate) mock_objects: HashMap<TypeId, BoxAnySend>,
}

impl Mocks {
    pub(crate) fn get<T: 'static>(&self, key: &TypeId) -> Option<&T> {
        self.mock_objects
            .get(key)
            .map(|mock| mock.downcast_ref::<T>().unwrap())
    }

    pub(crate) fn get_mut_or_create<I: Send + Sync + 'static, O: Send + Sync + 'static>(
        &mut self,
        key: TypeId,
        name: &'static str,
    ) -> &mut Mock<I, O> {
        self.mock_objects
            .entry(key)
            .or_insert(Box::new(Mock::<I, O>::new(name)))
            .downcast_mut()
            .unwrap()
    }

    #[doc(hidden)]
    pub fn record_call_and_find_mock_output<
        I: PartialEq + Debug + Clone + Send + Sync + 'static,
        O: Debug + Send + Sync + 'static,
    >(
        &mut self,
        key: TypeId,
        name: &'static str,
        input: I,
    ) -> Option<O> {
        self.get_mut_or_create::<I, O>(key, name)
            .record_call_and_find_mock_output(input)
    }

    #[cfg(test)]
    pub(crate) fn insert<T: Send + Sync + 'static>(&mut self, key: TypeId, item: T) {
        self.mock_objects.insert(key, Box::new(item));
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn get_returns_none() {
        let mock_data = Mocks::default();
        assert_eq!(mock_data.get::<u8>(&TypeId::of::<usize>()), None);
    }

    #[test]
    fn get_returns_an_item() {
        let mut mock_data = Mocks::default();
        mock_data.insert(TypeId::of::<usize>(), 4u8);
        assert_eq!(mock_data.get::<u8>(&TypeId::of::<usize>()), Some(&4u8));
    }

    #[test]
    fn get_mut_or_create_returns_an_item() {
        let mut mock_data = Mocks::default();
        mock_data.insert(TypeId::of::<usize>(), Mock::<u8, u8>::new("a"));
        assert_eq!(
            mock_data
                .get_mut_or_create::<u8, u8>(TypeId::of::<usize>(), &"meow")
                .name,
            "a"
        );
    }

    #[test]
    fn get_mut_or_create_returns_default() {
        let mut mock_data = Mocks::default();
        assert_eq!(
            mock_data
                .get_mut_or_create::<u8, u8>(TypeId::of::<usize>(), &"meow")
                .name,
            "meow"
        );
    }
}
