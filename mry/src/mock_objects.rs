use std::any::Any;
use std::collections::HashMap;

use crate::{InnerMryId, Mock, MryId};

type HashMapMocks = HashMap<&'static str, Box<dyn Any + Send>>;

#[derive(Default)]
pub struct MockObjects {
    mock_objects: HashMap<InnerMryId, HashMapMocks>,
}

impl MockObjects {
    pub(crate) fn get<T: 'static>(&self, id: &MryId, name: &'static str) -> Option<&T> {
        id.0.and_then(move |id| {
            self.mock_objects
                .get(&id)
                .and_then(|mocks| mocks.get(name))
                .map(|mock| mock.downcast_ref::<T>().unwrap())
        })
    }

    pub fn get_mut_or_create<I: Send + 'static, O: 'static>(
        &mut self,
        id: &MryId,
        name: &'static str,
    ) -> &mut Mock<I, O> {
        self.mock_objects
            .entry(id.0.unwrap())
            .or_default()
            .entry(name)
            .or_insert(Box::new(Mock::<I, O>::new(name)))
            .downcast_mut()
            .unwrap()
    }

    pub(crate) fn remove(&mut self, id: InnerMryId) {
        self.mock_objects.remove(&id);
    }

    #[cfg(test)]
    pub(crate) fn insert<T: Send + 'static>(
        &mut self,
        id: InnerMryId,
        name: &'static str,
        item: T,
    ) {
        self.mock_objects
            .entry(id)
            .or_default()
            .insert(name, Box::new(item));
    }

    #[cfg(test)]
    pub(crate) fn contains_key(&mut self, id: InnerMryId) -> bool {
        self.mock_objects.contains_key(&id)
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn get_returns_none() {
        let mry_id = MryId::generate();
        let mock_data = MockObjects::default();
        assert_eq!(mock_data.get::<u8>(&mry_id, "meow"), None);
    }

    #[test]
    fn get_returns_none_with_blank_mry_id() {
        let mry_id = MryId(None);
        let mock_data = MockObjects::default();
        assert_eq!(mock_data.get::<u8>(&mry_id, "meow"), None);
    }

    #[test]
    fn get_returns_an_item() {
        let mry_id = MryId::generate();
        let mut mock_data = MockObjects::default();
        mock_data.insert(mry_id.0.unwrap(), "meow", 4u8);
        assert_eq!(mock_data.get::<u8>(&mry_id, "meow"), Some(&4u8));
    }

    #[test]
    fn get_mut_or_create_returns_an_item() {
        let mry_id = MryId::generate();
        let mut mock_data = MockObjects::default();
        mock_data.insert(mry_id.0.unwrap(), "meow", Mock::<u8, u8>::new("a"));
        assert_eq!(
            mock_data.get_mut_or_create::<u8, u8>(&mry_id, "meow").name,
            "a"
        );
    }

    #[test]
    fn get_mut_or_create_returns_default() {
        let mry_id = MryId::generate();
        let mut mock_data = MockObjects::default();
        assert_eq!(
            mock_data.get_mut_or_create::<u8, u8>(&mry_id, "meow").name,
            "meow"
        );
    }

    #[test]
    fn remove() {
        let mry_id = MryId::generate();
        let mut mock_data = MockObjects::default();
        mock_data.insert(mry_id.0.unwrap(), "meow", 4u8);
        mock_data.remove(mry_id.0.unwrap());
        assert_eq!(mock_data.get::<u8>(&mry_id, "meow"), None);
    }
}
