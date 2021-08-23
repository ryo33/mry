use std::cmp::Ordering;
use std::ops::{Deref, DerefMut};
use std::sync::atomic::AtomicU16;
use std::sync::Arc;

use crate::MOCK_DATA;

#[derive(Debug, PartialEq, Eq, Default, Clone, Hash, PartialOrd, Ord)]
pub struct Mry(Arc<InnerMry>);

impl Mry {
    pub fn generate() -> Self {
        let id = ID.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        Self(Arc::new(InnerMry(Some(id))))
    }

    pub fn none() -> Self {
        Self(Arc::new(InnerMry(None)))
    }

    pub(crate) fn some(value: u16) -> Self {
        Self(Arc::new(InnerMry(Some(value))))
    }

    pub fn id(&self) -> Option<MryId> {
        self.0 .0
    }
}

impl Deref for Mry {
    type Target = InnerMry;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

pub type MryId = u16;
static ID: AtomicU16 = AtomicU16::new(0);

#[derive(Debug, Eq, Default)]
pub struct InnerMry(pub(crate) Option<MryId>);

impl PartialOrd for InnerMry {
    fn partial_cmp(&self, _: &Self) -> Option<std::cmp::Ordering> {
        Some(Ordering::Equal)
    }
}

impl Ord for InnerMry {
    fn cmp(&self, _: &Self) -> std::cmp::Ordering {
        Ordering::Equal
    }
}

impl Deref for InnerMry {
    type Target = Option<MryId>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for InnerMry {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl std::hash::Hash for InnerMry {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        (None as Option<MryId>).hash(state);
    }
}

impl PartialEq for InnerMry {
    fn eq(&self, _: &Self) -> bool {
        true
    }
}

impl Drop for InnerMry {
    fn drop(&mut self) {
        if let Some(inner_id) = self.0 {
            MOCK_DATA.remove(inner_id);
        }
    }
}

#[cfg(test)]
mod test {
    use std::cmp::Ordering;
    use std::collections::HashSet;

    use super::*;

    #[test]
    fn mry_unique() {
        assert_ne!(Mry::generate().id(), Mry::generate().id());
    }

    #[test]
    fn mry_default_is_none() {
        assert_eq!(Mry::default(), Mry::none());
    }

    #[test]
    fn mry_always_equal() {
        assert_eq!(Mry::some(0), Mry::some(1));
        assert_eq!(Mry::some(0), Mry::none());
        assert_eq!(Mry::none(), Mry::none());
    }

    #[test]
    fn mry_always_equal_ord() {
        assert_eq!(Mry::some(0).cmp(&Mry::some(1)), Ordering::Equal);
        assert_eq!(Mry::some(0).cmp(&Mry::none()), Ordering::Equal);
        assert_eq!(Mry::none().cmp(&Mry::none()), Ordering::Equal);
    }

    #[test]
    fn mry_always_equal_partial_ord() {
        assert_eq!(
            Mry::some(0).partial_cmp(&Mry::some(1)),
            Some(Ordering::Equal)
        );
        assert_eq!(
            Mry::some(0).partial_cmp(&Mry::none()),
            Some(Ordering::Equal)
        );
        assert_eq!(Mry::none().partial_cmp(&Mry::none()), Some(Ordering::Equal));
    }

    #[test]
    fn mry_hash_returns_consistent_value() {
        let mut set = HashSet::new();
        set.insert(Mry::some(0));
        set.insert(Mry::some(1));
        set.insert(Mry::none());
        assert_eq!(set.len(), 1);
    }

    #[test]
    fn delete_mock_data_on_drop() {
        let inner_id;
        {
            let id = Mry::generate();
            inner_id = id.0.unwrap();
            MOCK_DATA.insert(inner_id, "mock", 1);
        }
        assert!(!MOCK_DATA.contains_key(inner_id));
    }

    #[test]
    fn do_not_panic_on_clone_and_dropn_with_none() {
        let _ = Mry::none().clone();
    }

    #[test]
    fn delete_mock_data_on_drop_clone() {
        let inner_id;
        {
            let id = Mry::generate();
            inner_id = id.0.unwrap();
            {
                MOCK_DATA.insert(inner_id, "mock", "some");
            }
            {
                let cloned = id.clone();
                {
                    let a = cloned.clone();
                    let b = id.clone();
                    println!("{:?}, {:?}", a.0, b.0);
                }
                assert!(MOCK_DATA.contains_key(inner_id));
            }
            assert!(MOCK_DATA.contains_key(inner_id));
        }
        assert!(!MOCK_DATA.contains_key(inner_id));
    }
}
