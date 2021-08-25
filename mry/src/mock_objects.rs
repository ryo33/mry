use std::any::Any;
use std::collections::HashMap;

use crate::{Mock, Mry, MryId};

type BoxAnySend = Box<dyn Any + Send + Sync>;
type HashMapMocks = HashMap<&'static str, BoxAnySend>;

#[derive(Default)]
pub struct MockObjects {
    mock_objects: HashMap<MryId, HashMapMocks>,
}

impl MockObjects {
    pub(crate) fn get<T: 'static>(&self, id: &Mry, name: &'static str) -> Option<&T> {
        id.id().and_then(move |id| {
            self.mock_objects
                .get(&id)
                .and_then(|mocks| mocks.get(name))
                .map(|mock| mock.downcast_ref::<T>().unwrap())
        })
    }

    pub fn get_mut_or_create<I: Send + Sync + 'static, O: Clone + Send + Sync + 'static>(
        &mut self,
        id: &Mry,
        name: &'static str,
    ) -> &mut Mock<I, O> {
        self.mock_objects
            .entry(id.id().unwrap())
            .or_default()
            .entry(name)
            .or_insert(Box::new(Mock::<I, O>::new(name)))
            .downcast_mut()
            .unwrap()
    }

    pub(crate) fn remove(&mut self, id: MryId) -> Option<HashMapMocks> {
        self.mock_objects.remove(&id)
        // .map(|map| map.into_values().collect())
    }

    #[cfg(test)]
    pub(crate) fn insert<T: Send + Sync + 'static>(
        &mut self,
        id: MryId,
        name: &'static str,
        item: T,
    ) {
        self.mock_objects
            .entry(id)
            .or_default()
            .insert(name, Box::new(item));
    }

    pub(crate) fn contains_key(&self, id: MryId) -> bool {
        self.mock_objects.contains_key(&id)
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn get_returns_none() {
        let mry = Mry::generate();
        let mock_data = MockObjects::default();
        assert_eq!(mock_data.get::<u8>(&mry, "meow"), None);
    }

    #[test]
    fn get_returns_none_with_blank_mry() {
        let mry = Mry::none();
        let mock_data = MockObjects::default();
        assert_eq!(mock_data.get::<u8>(&mry, "meow"), None);
    }

    #[test]
    fn get_returns_an_item() {
        let mry = Mry::generate();
        let mut mock_data = MockObjects::default();
        mock_data.insert(mry.id().unwrap(), "meow", 4u8);
        assert_eq!(mock_data.get::<u8>(&mry, "meow"), Some(&4u8));
    }

    #[test]
    fn get_mut_or_create_returns_an_item() {
        let mry = Mry::generate();
        let mut mock_data = MockObjects::default();
        mock_data.insert(mry.id().unwrap(), "meow", Mock::<u8, u8>::new("a"));
        assert_eq!(
            mock_data.get_mut_or_create::<u8, u8>(&mry, "meow").name,
            "a"
        );
    }

    #[test]
    fn get_mut_or_create_returns_default() {
        let mry = Mry::generate();
        let mut mock_data = MockObjects::default();
        assert_eq!(
            mock_data.get_mut_or_create::<u8, u8>(&mry, "meow").name,
            "meow"
        );
    }

    #[test]
    fn remove() {
        let mry = Mry::generate();
        let mut mock_data = MockObjects::default();
        mock_data.insert(mry.id().unwrap(), "meow", 4u8);
        mock_data.remove(mry.id().unwrap());
        assert_eq!(mock_data.get::<u8>(&mry, "meow"), None);
    }
}
