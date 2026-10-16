use crate::mockable::MockableArg;
use crate::mockable::MockableRet;
#[cfg(debug_assertions)]
use parking_lot::Mutex;
use std::any::TypeId;
#[cfg(not(debug_assertions))]
use std::cmp::Ordering;
#[cfg(debug_assertions)]
use std::sync::atomic::AtomicU16;
#[cfg(debug_assertions)]
use std::sync::atomic::AtomicUsize;
#[cfg(debug_assertions)]
use std::sync::Arc;

#[cfg(debug_assertions)]
use crate::MockGetter;
#[cfg(debug_assertions)]
use crate::Mocks;

/// A unique id for an object
pub type MryId = u16;
#[cfg(debug_assertions)]
static ID: AtomicU16 = AtomicU16::new(0);
#[cfg(debug_assertions)]
static DUMMY_ID: AtomicUsize = AtomicUsize::new(0);

#[derive(Clone)]
/// Mock container that has blank and harmless trait implementation for major traits such as `Eq` and `Ord`
///
/// The only exception is a dummy created by `mry::dummy!`, which is equal only to itself and its clones in debug build.
pub struct Mry {
    #[cfg(debug_assertions)]
    id: MryId,
    #[cfg(debug_assertions)]
    dummy_id: Option<usize>,
    #[cfg(debug_assertions)]
    mocks: Option<Arc<Mutex<Mocks>>>,
}

#[test]
fn assert_send_mry() {
    fn assert_send<T: Send>(_: T) {}
    fn assert_sync<T: Sync>(_: T) {}
    assert_send(Mry::default());
    assert_sync(Mry::default());
}

impl std::fmt::Debug for Mry {
    #[cfg(debug_assertions)]
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut debug_struct = f.debug_struct("Mry");
        match self.dummy_id {
            Some(dummy_id) => debug_struct.field("dummy_id", &dummy_id),
            None => debug_struct.field("id", &self.id),
        };
        debug_struct.finish()
    }
    #[cfg(not(debug_assertions))]
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Mry").finish()
    }
}

impl Mry {
    #[doc(hidden)]
    #[cfg(debug_assertions)]
    pub fn dummy() -> Self {
        Self {
            dummy_id: Some(DUMMY_ID.fetch_add(1, std::sync::atomic::Ordering::Relaxed)),
            ..Self::default()
        }
    }

    #[doc(hidden)]
    #[cfg(not(debug_assertions))]
    pub fn dummy() -> Self {
        Self::default()
    }

    #[cfg(debug_assertions)]
    pub(crate) fn generate(&mut self) -> &mut Self {
        self.mocks
            .get_or_insert(Arc::new(Mutex::new(Default::default())));
        self
    }

    #[doc(hidden)]
    #[cfg(debug_assertions)]
    #[track_caller]
    pub fn record_call_and_find_mock_output<I: MockableArg, O: MockableRet>(
        &self,
        key: TypeId,
        name: &'static str,
        input: I,
    ) -> Option<O> {
        if let Some(mocks) = self.mocks.as_ref() {
            mocks
                .lock()
                .record_call_and_find_mock_output(key, name, input)
        } else {
            None
        }
    }

    #[cfg(not(debug_assertions))]
    pub fn record_call_and_find_mock_output<
        I: PartialEq + std::fmt::Debug + Clone + Send + 'static,
        O: std::fmt::Debug + Send + 'static,
    >(
        &self,
        _key: TypeId,
        _name: &'static str,
        _input: I,
    ) -> Option<O> {
        None
    }

    #[doc(hidden)]
    #[cfg(debug_assertions)]
    pub fn mocks<I: MockableArg, O: MockableRet>(&mut self) -> Arc<Mutex<dyn MockGetter<I, O>>> {
        self.generate().mocks.as_ref().unwrap().clone()
    }
}

impl Default for Mry {
    #[cfg(debug_assertions)]
    fn default() -> Self {
        Self {
            id: ID.fetch_add(1, std::sync::atomic::Ordering::Relaxed),
            dummy_id: None,
            mocks: None,
        }
    }

    #[cfg(not(debug_assertions))]
    fn default() -> Self {
        Self {}
    }
}

impl PartialOrd for Mry {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Eq for Mry {
    fn assert_receiver_is_total_eq(&self) {}
}

impl Ord for Mry {
    #[cfg(debug_assertions)]
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.dummy_id.cmp(&other.dummy_id)
    }

    #[cfg(not(debug_assertions))]
    fn cmp(&self, _: &Self) -> std::cmp::Ordering {
        Ordering::Equal
    }
}

impl std::hash::Hash for Mry {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        std::hash::Hash::hash(&None as &Option<MryId>, state);
    }
}

impl PartialEq for Mry {
    #[cfg(debug_assertions)]
    fn eq(&self, other: &Self) -> bool {
        self.dummy_id == other.dummy_id
    }

    #[cfg(not(debug_assertions))]
    fn eq(&self, _: &Self) -> bool {
        true
    }
}

#[cfg(feature = "serde")]
impl serde::Serialize for Mry {
    fn serialize<S: serde::Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
        s.serialize_unit()
    }
}

#[cfg(feature = "serde")]
impl<'de> serde::Deserialize<'de> for Mry {
    fn deserialize<D: serde::Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        d.deserialize_unit(serde::de::IgnoredAny)
            .map(|_| Self::default())
    }
}

#[cfg(test)]
mod test {
    use std::cmp::Ordering;
    use std::collections::HashSet;

    use crate::mock::Mock;
    use crate::Matcher;

    use super::*;

    #[test]
    fn mry_unique() {
        let mut mry1 = Mry::default();
        let mry2 = Mry::default();
        assert_ne!(mry1.generate().id, mry2.id);
    }

    #[test]
    fn mry_default_is_none() {
        assert!(Mry::default().mocks.is_none());
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
        #[allow(clippy::mutable_key_type)]
        let mut set = HashSet::new();
        set.insert(Mry::default());
        set.insert(Mry::default());
        assert_eq!(set.len(), 1);
    }

    #[test]
    fn dummy_is_not_equal_to_another_dummy() {
        assert_ne!(Mry::dummy(), Mry::dummy());
    }

    #[test]
    fn dummy_is_equal_to_its_clone() {
        let dummy = Mry::dummy();
        assert_eq!(dummy.clone(), dummy);
        assert_eq!(dummy.clone().cmp(&dummy), Ordering::Equal);
    }

    #[test]
    fn debug_shows_dummy_id_instead_of_id_for_dummy() {
        let mry = Mry::default();
        assert_eq!(format!("{mry:?}"), format!("Mry {{ id: {} }}", mry.id));

        let dummy = Mry::dummy();
        assert_eq!(
            format!("{dummy:?}"),
            format!("Mry {{ dummy_id: {} }}", dummy.dummy_id.unwrap())
        );
    }

    #[test]
    fn dummy_is_not_equal_to_default() {
        assert_ne!(Mry::dummy(), Mry::default());
        assert_ne!(Mry::dummy().cmp(&Mry::default()), Ordering::Equal);
    }

    #[test]
    fn generate_create_mock() {
        let mut mry = Mry::default();
        assert!(mry.mocks.is_none());
        mry.generate();
        assert!(mry.mocks.is_some());
    }

    #[test]
    fn generate_does_not_overwrite() {
        let mut mry = Mry::default();
        mry.generate();
        mry.mocks
            .as_ref()
            .unwrap()
            .lock()
            .insert(TypeId::of::<usize>(), Mock::<usize, usize>::new(""));
        mry.generate();
        assert_eq!(mry.mocks.unwrap().lock().mock_objects.len(), 1);
    }

    #[test]
    fn clone() {
        let mut mry = Mry::default();
        mry.generate();
        mry.mocks
            .as_ref()
            .unwrap()
            .lock()
            .insert(TypeId::of::<usize>(), Mock::<usize, usize>::new(""));

        assert_eq!(mry.clone().mocks.unwrap().lock().mock_objects.len(), 1);
    }

    #[test]
    fn inner_called_returns_none_when_no_mocks() {
        let mry = Mry::default();

        assert_eq!(
            mry.record_call_and_find_mock_output::<u8, u16>(TypeId::of::<usize>(), "name", 1u8),
            None
        );
    }

    #[test]
    fn inner_called_forwards_to_mock() {
        let mut mry = Mry::default();

        mry.mocks()
            .lock()
            .get_mut_or_create(TypeId::of::<usize>(), "name")
            .returns(Matcher::new_eq(1u8).wrapped(), 1u8);

        assert_eq!(
            mry.record_call_and_find_mock_output::<u8, u8>(TypeId::of::<usize>(), "name", 1u8),
            Some(1u8)
        );
    }
}
