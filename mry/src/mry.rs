use std::cmp::Ordering;
use std::fmt::Debug;
use std::ops::DerefMut;
use std::sync::atomic::AtomicU16;
use std::sync::Arc;

use parking_lot::RwLock;

use crate::Mocks;

/// A unique id for an object
pub type MryId = u16;
static ID: AtomicU16 = AtomicU16::new(0);

#[derive(Clone)]
/// Mock container that has blank and harmless trait implementation for major traits such as `Eq` and `Ord`
pub struct Mry {
    id: MryId,
    _mocks: Option<Arc<RwLock<Mocks>>>,
}

impl std::fmt::Debug for Mry {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Mry").field("id", &self.id).finish()
    }
}

impl Mry {
    pub(crate) fn generate(&mut self) -> &mut Self {
        self._mocks
            .get_or_insert(Arc::new(RwLock::new(Default::default())));
        self
    }

    #[doc(hidden)]
    pub fn record_call_and_find_mock_output<
        I: PartialEq + Debug + Clone + Send + Sync + 'static,
        O: Debug + Send + Sync + 'static,
    >(
        &self,
        name: &'static str,
        input: I,
    ) -> Option<O> {
        if let Some(ref mocks) = self._mocks {
            mocks
                .write()
                .get_mut_or_create::<I, O>(name)
                .record_call_and_find_mock_output(input)
        } else {
            None
        }
    }

    #[doc(hidden)]
    pub fn mocks_write<'a>(&'a mut self) -> impl DerefMut<Target = Mocks> + 'a {
        self.generate()._mocks.as_ref().unwrap().write()
    }

    /// Returns a unique object ID
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

    use crate::Matcher;

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

    #[test]
    fn inner_called_returns_none_when_no_mocks() {
        let mry = Mry::default();

        assert_eq!(
            mry.record_call_and_find_mock_output::<u8, u16>("name", 1u8),
            None
        );
    }

    #[test]
    fn inner_called_forwards_to_mock() {
        let mut mry = Mry::default();

        mry.mocks_write()
            .get_mut_or_create::<u8, u16>("name")
            .returns(Matcher::Any, 1);

        assert_eq!(
            mry.record_call_and_find_mock_output::<u8, u16>("name", 1u8),
            Some(1)
        );
    }
}
