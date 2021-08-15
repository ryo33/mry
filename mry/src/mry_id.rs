use parking_lot::Mutex;
use std::cmp::Ordering;
use std::collections::HashMap;
use std::ops::{Deref, DerefMut};

use once_cell::sync::Lazy;

use crate::MOCK_DATA;

pub type InnerMryId = u16;
static ID: Lazy<Mutex<InnerMryId>> = Lazy::new(|| Mutex::new(0));
static CLONE_COUNT: Lazy<Mutex<HashMap<InnerMryId, u8>>> = Lazy::new(|| Mutex::new(HashMap::new()));

#[derive(Debug, Eq, Default)]
pub struct MryId(pub(crate) Option<InnerMryId>);

impl PartialOrd for MryId {
    fn partial_cmp(&self, _: &Self) -> Option<std::cmp::Ordering> {
        Some(Ordering::Equal)
    }
}

impl Ord for MryId {
    fn cmp(&self, _: &Self) -> std::cmp::Ordering {
        Ordering::Equal
    }
}

impl Deref for MryId {
    type Target = Option<InnerMryId>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for MryId {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl std::hash::Hash for MryId {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        (None as Option<InnerMryId>).hash(state);
    }
}

impl PartialEq for MryId {
    fn eq(&self, _: &Self) -> bool {
        true
    }
}

impl MryId {
    pub fn generate() -> Self {
        let mut id = ID.lock();
        *id = *id + 1;
        Self(Some(*id))
    }
}

impl Drop for MryId {
    fn drop(&mut self) {
        if let Some(inner_id) = self.0 {
            let mut lock = CLONE_COUNT.lock();
            let count = lock.get_mut(&inner_id);
            let count = match count {
                Some(count) => {
                    assert_ne!(*count, 0);
                    *count = *count - 1;
                    *count
                }
                None => 0,
            };
            if count == 0 {
                lock.remove(&inner_id);
                MOCK_DATA.lock().remove(inner_id);
            }
        }
    }
}

impl Clone for MryId {
    fn clone(&self) -> Self {
        if let Some(id) = self.0 {
            let mut lock = CLONE_COUNT.lock();
            let count = lock.entry(id).or_insert(1);
            *count = *count + 1;
        }
        Self(self.0.clone())
    }
}

#[cfg(test)]
mod test {
    use std::cmp::Ordering;
    use std::collections::HashSet;

    use super::*;

    #[test]
    fn mry_id_unique() {
        assert_ne!(MryId::generate().0, MryId::generate().0);
    }

    #[test]
    fn mry_id_default_is_none() {
        assert_eq!(MryId::default(), MryId(None));
    }

    #[test]
    fn mry_id_always_equal() {
        assert_eq!(MryId(Some(0)), MryId(Some(1)));
        assert_eq!(MryId(Some(0)), MryId(None));
        assert_eq!(MryId(None), MryId(None));
    }

    #[test]
    fn mry_id_always_equal_ord() {
        assert_eq!(MryId(Some(0)).cmp(&MryId(Some(1))), Ordering::Equal);
        assert_eq!(MryId(Some(0)).cmp(&MryId(None)), Ordering::Equal);
        assert_eq!(MryId(None).cmp(&MryId(None)), Ordering::Equal);
    }

    #[test]
    fn mry_id_always_equal_partial_ord() {
        assert_eq!(
            MryId(Some(0)).partial_cmp(&MryId(Some(1))),
            Some(Ordering::Equal)
        );
        assert_eq!(
            MryId(Some(0)).partial_cmp(&MryId(None)),
            Some(Ordering::Equal)
        );
        assert_eq!(MryId(None).partial_cmp(&MryId(None)), Some(Ordering::Equal));
    }

    #[test]
    fn mry_id_hash_returns_consistent_value() {
        let mut set = HashSet::new();
        set.insert(MryId(Some(0)));
        set.insert(MryId(Some(1)));
        set.insert(MryId(None));
        assert_eq!(set.len(), 1);
    }

    #[test]
    fn delete_mock_data_on_drop() {
        let inner_id;
        {
            let id = MryId::generate();
            inner_id = id.0.unwrap();
            MOCK_DATA.lock().insert(inner_id, "mock", 1);
        }
        assert!(!MOCK_DATA.lock().contains_key(inner_id));
    }

    #[test]
    fn delete_clone_count_on_drop() {
        let inner_id;
        {
            let id = MryId::generate().clone();
            inner_id = id.0.unwrap();
            assert!(CLONE_COUNT.lock().contains_key(&inner_id));
        }
        assert!(!CLONE_COUNT.lock().contains_key(&inner_id));
    }

    #[test]
    fn do_not_panic_on_clone_and_dropn_with_none() {
        let _ = MryId(None).clone();
    }

    #[test]
    fn delete_mock_data_on_drop_clone() {
        let inner_id;
        {
            let id = MryId::generate();
            inner_id = id.0.unwrap();
            {
                let mut mock_data = MOCK_DATA.lock();
                mock_data.insert(inner_id, "mock", "some");
            }
            {
                let cloned = id.clone();
                {
                    let a = cloned.clone();
                    let b = id.clone();
                    println!("{:?}, {:?}", a.0, b.0);
                }
                assert!(MOCK_DATA.lock().contains_key(inner_id));
            }
            assert!(MOCK_DATA.lock().contains_key(inner_id));
        }
        assert!(!MOCK_DATA.lock().contains_key(inner_id));
    }
}
