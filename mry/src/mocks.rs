use std::any::Any;
use std::collections::HashMap;

use crate::mock_key::BoxMockKey;
use crate::Mock;

type BoxAnySend = Box<dyn Any + Send + Sync>;

#[derive(Default)]
#[doc(hidden)]
pub struct Mocks {
    pub(crate) mock_objects: HashMap<BoxMockKey, BoxAnySend>,
}

impl Mocks {
    pub(crate) fn get<T: 'static>(&self, key: &BoxMockKey) -> Option<&T> {
        self.mock_objects
            .get(key)
            .map(|mock| mock.downcast_ref::<T>().unwrap())
    }

    pub(crate) fn get_mut_or_create<I: Send + Sync + 'static, O: Send + Sync + 'static>(
        &mut self,
        key: BoxMockKey,
        name: &'static str,
    ) -> &mut Mock<I, O> {
        self.mock_objects
            .entry(key)
            .or_insert(Box::new(Mock::<I, O>::new(name)))
            .downcast_mut()
            .unwrap()
    }

    #[cfg(test)]
    pub(crate) fn insert<T: Send + Sync + 'static>(&mut self, key: BoxMockKey, item: T) {
        self.mock_objects.insert(key, Box::new(item));
    }
}

#[cfg(test)]
mod test {
    use crate::mock_key::key_a;

    use super::*;

    #[test]
    fn get_returns_none() {
        let mock_data = Mocks::default();
        assert_eq!(mock_data.get::<u8>(&key_a()), None);
    }

    #[test]
    fn get_returns_an_item() {
        let mut mock_data = Mocks::default();
        mock_data.insert(key_a(), 4u8);
        assert_eq!(mock_data.get::<u8>(&key_a()), Some(&4u8));
    }

    #[test]
    fn get_mut_or_create_returns_an_item() {
        let mut mock_data = Mocks::default();
        mock_data.insert(key_a(), Mock::<u8, u8>::new("a"));
        assert_eq!(
            mock_data.get_mut_or_create::<u8, u8>(key_a(), &"meow").name,
            "a"
        );
    }

    #[test]
    fn get_mut_or_create_returns_default() {
        let mut mock_data = Mocks::default();
        assert_eq!(
            mock_data.get_mut_or_create::<u8, u8>(key_a(), &"meow").name,
            "meow"
        );
    }
}
