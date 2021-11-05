use std::any::{Any, TypeId};
use std::collections::HashMap;
use std::fmt::Debug;

use crate::mock::Mock;

type BoxAnySend = Box<dyn Any + Send + Sync>;

#[derive(Default)]
#[doc(hidden)]
pub struct Mocks {
    pub(crate) mock_objects: HashMap<TypeId, BoxAnySend>,
}

impl Mocks {
    pub(crate) fn get<I: 'static, O: 'static>(&self, key: &TypeId) -> Option<&Mock<I, O>> {
        self.mock_objects
            .get(key)
            .map(|mock| mock.downcast_ref().unwrap())
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
    pub(crate) fn insert<I: Send + Sync + 'static, O: 'static>(
        &mut self,
        key: TypeId,
        item: Mock<I, O>,
    ) {
        self.mock_objects.insert(key, Box::new(item));
    }

    pub(crate) fn remove(&mut self, key: &TypeId) {
        self.mock_objects.remove(key);
    }
}

#[cfg(test)]
mod test {
    use crate::{Behavior, Matcher};

    use super::*;

    #[test]
    fn get_returns_none() {
        let mock_data = Mocks::default();
        assert!(mock_data
            .get::<usize, usize>(&TypeId::of::<usize>())
            .is_none());
    }

    #[test]
    fn get_returns_an_item() {
        let mut mock_data = Mocks::default();
        mock_data.insert(TypeId::of::<usize>(), Mock::<usize, usize>::new(""));
        assert!(mock_data
            .get::<usize, usize>(&TypeId::of::<usize>())
            .is_some());
    }

    #[test]
    fn get_mut_or_create_returns_an_item() {
        let mut mock_data = Mocks::default();
        let mut mock = Mock::<u8, u8>::new("a");
        mock.returns_with(Matcher::Any, Behavior::Function(Box::new(|_| 4u8)));
        mock_data.insert(TypeId::of::<usize>(), mock);
        assert_eq!(
            mock_data
                .get_mut_or_create::<u8, u8>(TypeId::of::<usize>(), &"meow")
                .record_call_and_find_mock_output(1),
            Some(4)
        );
    }

    #[test]
    // should not panic
    fn get_mut_or_create_returns_default() {
        let mut mock_data = Mocks::default();

        mock_data.get_mut_or_create::<u8, u8>(TypeId::of::<usize>(), &"meow");
    }
}
