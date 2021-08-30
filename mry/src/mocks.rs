use std::any::Any;
use std::collections::HashMap;

use crate::Mock;

type BoxAnySend = Box<dyn Any + Send + Sync>;

#[derive(Default)]
#[doc(hidden)]
pub struct Mocks {
    pub(crate) mock_objects: HashMap<&'static str, BoxAnySend>,
}

impl Mocks {
    pub(crate) fn get<T: 'static>(&self, name: &'static str) -> Option<&T> {
        self.mock_objects
            .get(name)
            .map(|mock| mock.downcast_ref::<T>().unwrap())
    }

    pub(crate) fn get_mut_or_create<I: Send + Sync + 'static, O: Send + Sync + 'static>(
        &mut self,
        name: &'static str,
    ) -> &mut Mock<I, O> {
        self.mock_objects
            .entry(name)
            .or_insert(Box::new(Mock::<I, O>::new(name)))
            .downcast_mut()
            .unwrap()
    }

    #[cfg(test)]
    pub(crate) fn insert<T: Send + Sync + 'static>(&mut self, name: &'static str, item: T) {
        self.mock_objects.insert(name, Box::new(item));
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn get_returns_none() {
        let mock_data = Mocks::default();
        assert_eq!(mock_data.get::<u8>("meow"), None);
    }

    #[test]
    fn get_returns_an_item() {
        let mut mock_data = Mocks::default();
        mock_data.insert("meow", 4u8);
        assert_eq!(mock_data.get::<u8>("meow"), Some(&4u8));
    }

    #[test]
    fn get_mut_or_create_returns_an_item() {
        let mut mock_data = Mocks::default();
        mock_data.insert("meow", Mock::<u8, u8>::new("a"));
        assert_eq!(mock_data.get_mut_or_create::<u8, u8>("meow").name, "a");
    }

    #[test]
    fn get_mut_or_create_returns_default() {
        let mut mock_data = Mocks::default();
        assert_eq!(mock_data.get_mut_or_create::<u8, u8>("meow").name, "meow");
    }
}
