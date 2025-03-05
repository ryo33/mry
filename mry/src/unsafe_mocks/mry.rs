use super::mockable::UnsafeMockableArg;
use super::mockable::UnsafeMockableRet;

use std::any::TypeId;
use std::cell::Cell;
#[cfg(debug_assertions)]
use std::cell::RefCell;
use std::cmp::Ordering;
#[cfg(debug_assertions)]
use std::rc::Rc;

#[cfg(debug_assertions)]
use super::UnsafeMockGetter;
#[cfg(debug_assertions)]
use super::UnsafeMocks;

/// A unique id for an object
pub type MryId = u16;

thread_local! {
    #[cfg(debug_assertions)]
    static ID: Cell<u16> = Cell::new(0);
}

#[derive(Clone)]
/// Mock container that has blank and harmless trait implementation for major traits such as `Eq` and `Ord`
pub struct UnsafeMry {
    #[cfg(debug_assertions)]
    id: MryId,
    #[cfg(debug_assertions)]
    mocks: Option<Rc<RefCell<UnsafeMocks>>>,
}

impl std::fmt::Debug for UnsafeMry {
    #[cfg(debug_assertions)]
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Mry").field("id", &self.id).finish()
    }
    #[cfg(not(debug_assertions))]
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Mry").finish()
    }
}

impl UnsafeMry {
    #[cfg(debug_assertions)]
    pub(crate) fn generate(&mut self) -> &mut Self {
        self.mocks
            .get_or_insert(Rc::new(RefCell::new(Default::default())));
        self
    }

    #[doc(hidden)]
    #[cfg(debug_assertions)]
    pub fn record_call_and_find_mock_output<I: UnsafeMockableArg, O: UnsafeMockableRet>(
        &self,
        key: TypeId,
        name: &'static str,
        input: I,
    ) -> Option<O> {
        self.mocks.as_ref().and_then(|mocks| {
            mocks
                .borrow_mut()
                .record_call_and_find_mock_output(key, name, input)
        })
    }

    #[cfg(not(debug_assertions))]
    pub fn record_call_and_find_mock_output<
        I: PartialEq + std::fmt::Debug + Clone + 'static,
        O: std::fmt::Debug + 'static,
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
    pub fn mocks<I: UnsafeMockableArg, O: UnsafeMockableRet>(
        &mut self,
    ) -> Rc<RefCell<dyn UnsafeMockGetter<I, O>>> {
        self.generate().mocks.as_ref().unwrap().clone()
    }
}

impl Default for UnsafeMry {
    #[cfg(debug_assertions)]
    fn default() -> Self {
        Self {
            id: ID.replace(ID.get() + 1),
            mocks: None,
        }
    }

    #[cfg(not(debug_assertions))]
    fn default() -> Self {
        Self {}
    }
}

impl PartialOrd for UnsafeMry {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Eq for UnsafeMry {
    fn assert_receiver_is_total_eq(&self) {}
}

impl Ord for UnsafeMry {
    fn cmp(&self, _: &Self) -> std::cmp::Ordering {
        Ordering::Equal
    }
}

impl std::hash::Hash for UnsafeMry {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        std::hash::Hash::hash(&None as &Option<MryId>, state);
    }
}

impl PartialEq for UnsafeMry {
    fn eq(&self, _: &Self) -> bool {
        true
    }
}

#[cfg(feature = "serde")]
impl serde::Serialize for UnsafeMry {
    fn serialize<S: serde::Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
        s.serialize_unit()
    }
}

#[cfg(feature = "serde")]
impl<'de> serde::Deserialize<'de> for UnsafeMry {
    fn deserialize<D: serde::Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        d.deserialize_unit(serde::de::IgnoredAny)
            .map(|_| Self::default())
    }
}

#[cfg(test)]
mod test {
    use std::cmp::Ordering;
    use std::collections::HashSet;

    use crate::unsafe_mocks::matcher::UnsafeMatcher;
    use crate::unsafe_mocks::mock::UnsafeMock;

    use super::*;

    #[test]
    fn mry_unique() {
        let mut mry1 = UnsafeMry::default();
        let mry2 = UnsafeMry::default();
        assert_ne!(mry1.generate().id, mry2.id);
    }

    #[test]
    fn mry_default_is_none() {
        assert!(UnsafeMry::default().mocks.is_none());
    }

    #[test]
    fn mry_always_equal() {
        assert_eq!(*UnsafeMry::default().generate(), UnsafeMry::default());
    }

    #[test]
    fn mry_always_equal_ord() {
        assert_eq!(
            UnsafeMry::default().cmp(UnsafeMry::default().generate()),
            Ordering::Equal
        );
    }

    #[test]
    fn mry_always_equal_partial_ord() {
        assert_eq!(
            UnsafeMry::default().partial_cmp(&UnsafeMry::default()),
            Some(Ordering::Equal)
        );
    }

    #[test]
    fn mry_hash_returns_consistent_value() {
        #[allow(clippy::mutable_key_type)]
        let mut set = HashSet::new();
        set.insert(UnsafeMry::default());
        set.insert(UnsafeMry::default());
        assert_eq!(set.len(), 1);
    }

    #[test]
    fn generate_create_mock() {
        let mut mry = UnsafeMry::default();
        assert!(mry.mocks.is_none());
        mry.generate();
        assert!(mry.mocks.is_some());
    }

    #[test]
    fn generate_does_not_overwrite() {
        let mut mry = UnsafeMry::default();
        mry.generate();
        mry.mocks
            .as_ref()
            .unwrap()
            .borrow_mut()
            .insert(TypeId::of::<usize>(), UnsafeMock::<usize, usize>::new(""));
        mry.generate();
        assert_eq!(mry.mocks.unwrap().borrow().mock_objects.len(), 1);
    }

    #[test]
    fn clone() {
        let mut mry = UnsafeMry::default();
        mry.generate();
        mry.mocks
            .as_ref()
            .unwrap()
            .borrow_mut()
            .insert(TypeId::of::<usize>(), UnsafeMock::<usize, usize>::new(""));

        assert_eq!(mry.clone().mocks.unwrap().borrow().mock_objects.len(), 1);
    }

    #[test]
    fn inner_called_returns_none_when_no_mocks() {
        let mry = UnsafeMry::default();

        assert_eq!(
            mry.record_call_and_find_mock_output::<u8, u16>(TypeId::of::<usize>(), "name", 1u8),
            None
        );
    }

    #[test]
    fn inner_called_forwards_to_mock() {
        let mut mry = UnsafeMry::default();

        mry.mocks()
            .borrow_mut()
            .get_mut_or_create(TypeId::of::<usize>(), "name")
            .returns(UnsafeMatcher::new_eq(1u8).wrapped(), 1u8);

        assert_eq!(
            mry.record_call_and_find_mock_output::<u8, u8>(TypeId::of::<usize>(), "name", 1u8),
            Some(1u8)
        );
    }
}
