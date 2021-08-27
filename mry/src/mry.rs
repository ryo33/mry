use std::cmp::Ordering;
use std::ops::{Deref, DerefMut};
use std::sync::atomic::AtomicU16;
use std::sync::Arc;

use parking_lot::RwLock;

use crate::Mocks;

pub type MryId = u16;
static ID: AtomicU16 = AtomicU16::new(0);

#[derive(Clone)]
pub struct Mry {
    id: MryId,
    pub _mocks: Option<Arc<RwLock<Mocks>>>,
}

impl std::fmt::Debug for Mry {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Mry").field("id", &self.id).finish()
    }
}

impl Mry {
    pub fn generate(&mut self) -> &mut Self {
        self._mocks
            .get_or_insert(Arc::new(RwLock::new(Default::default())));
        self
    }

    pub fn id(&self) -> MryId {
        self.id
    }
}

impl Default for Mry {
    fn default() -> Self {
        Self {
            id: ID.fetch_add(1, std::sync::atomic::Ordering::Relaxed),
            _mocks: None,
        }
    }
}

impl PartialOrd for Mry {
    fn partial_cmp(&self, _: &Self) -> Option<std::cmp::Ordering> {
        Some(Ordering::Equal)
    }
}

impl Eq for Mry {
    fn assert_receiver_is_total_eq(&self) {}
}

impl Ord for Mry {
    fn cmp(&self, _: &Self) -> std::cmp::Ordering {
        Ordering::Equal
    }
}

impl std::hash::Hash for Mry {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        (None as Option<MryId>).hash(state);
    }
}

impl PartialEq for Mry {
    fn eq(&self, _: &Self) -> bool {
        true
    }
}

#[cfg(test)]
mod test {
    use std::cmp::Ordering;
    use std::collections::HashSet;

    use super::*;

    #[test]
    fn mry_unique() {
        let mut mry1 = Mry::default();
        let mry2 = Mry::default();
        assert_ne!(mry1.generate().id(), mry2.id());
    }

    #[test]
    fn mry_default_is_none() {
        assert!(Mry::default()._mocks.is_none());
    }

    #[test]
    fn mry_always_equal() {
        assert_eq!(*Mry::default().generate(), Mry::default());
    }

    #[test]
    fn mry_always_equal_ord() {
        assert_eq!(
            Mry::default().cmp(Mry::default().generate()),
            Ordering::Equal
        );
    }

    #[test]
    fn mry_always_equal_partial_ord() {
        assert_eq!(
            Mry::default().partial_cmp(&Mry::default()),
            Some(Ordering::Equal)
        );
    }

    #[test]
    fn mry_hash_returns_consistent_value() {
        let mut set = HashSet::new();
        set.insert(Mry::default());
        set.insert(Mry::default());
        assert_eq!(set.len(), 1);
    }

    #[test]
    fn generate_create_mock() {
        let mut mry = Mry::default();
        assert!(mry._mocks.is_none());
        mry.generate();
        assert!(mry._mocks.is_some());
    }

    #[test]
    fn generate_does_not_overwrite() {
        let mut mry = Mry::default();
        mry.generate();
        mry._mocks.as_ref().unwrap().write().insert("a", 4u8);
        mry.generate();
        assert_eq!(mry._mocks.unwrap().read().mock_objects.len(), 1);
    }

    #[test]
    fn clone() {
        let mut mry = Mry::default();
        mry.generate();
        mry._mocks.as_ref().unwrap().write().insert("a", 4u8);

        assert_eq!(mry.clone()._mocks.unwrap().read().mock_objects.len(), 1);
    }
}
