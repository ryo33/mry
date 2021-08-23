use parking_lot::Mutex;
use std::cmp::Ordering;
use std::collections::HashMap;
use std::ops::{Deref, DerefMut};

use once_cell::sync::Lazy;

use crate::MOCK_DATA;

pub type InnerMry = u16;
static ID: Lazy<Mutex<InnerMry>> = Lazy::new(|| Mutex::new(0));
static CLONE_COUNT: Lazy<Mutex<HashMap<InnerMry, u8>>> = Lazy::new(|| Mutex::new(HashMap::new()));

#[derive(Debug, Eq, Default)]
pub struct Mry(pub(crate) Option<InnerMry>);

impl PartialOrd for Mry {
    fn partial_cmp(&self, _: &Self) -> Option<std::cmp::Ordering> {
        Some(Ordering::Equal)
    }
}

impl Ord for Mry {
    fn cmp(&self, _: &Self) -> std::cmp::Ordering {
        Ordering::Equal
    }
}

impl Deref for Mry {
    type Target = Option<InnerMry>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for Mry {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl std::hash::Hash for Mry {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        (None as Option<InnerMry>).hash(state);
    }
}

impl PartialEq for Mry {
    fn eq(&self, _: &Self) -> bool {
        true
    }
}

impl Mry {
    pub fn generate() -> Self {
        let mut id = ID.lock();
        *id = *id + 1;
        Self(Some(*id))
    }

    pub fn none() -> Self {
        Self(None)
    }

    pub fn id(&self) -> Option<InnerMry> {
        self.0
    }
}

impl Drop for Mry {
    fn drop(&mut self) {
        if let Some(inner_id) = self.0 {
            let clone_count;
            // This block is needed to free CLONE_COUNT.lock()
            {
                let mut lock = CLONE_COUNT.lock();
                let count = lock.get_mut(&inner_id);
                clone_count = match count {
                    Some(count) => {
                        assert_ne!(*count, 0);
                        *count = *count - 1;
                        *count
                    }
                    None => 0,
                };
            }
            if clone_count == 0 {
                CLONE_COUNT.lock().remove(&inner_id);
            }
            if clone_count == 0 {
                MOCK_DATA.lock().remove(inner_id);
            }
        }
    }
}

impl Clone for Mry {
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
    fn mry_unique() {
        assert_ne!(Mry::generate().0, Mry::generate().0);
    }

    #[test]
    fn mry_default_is_none() {
        assert_eq!(Mry::default(), Mry::none());
    }

    #[test]
    fn mry_always_equal() {
        assert_eq!(Mry(Some(0)), Mry(Some(1)));
        assert_eq!(Mry(Some(0)), Mry::none());
        assert_eq!(Mry::none(), Mry::none());
    }

    #[test]
    fn mry_always_equal_ord() {
        assert_eq!(Mry(Some(0)).cmp(&Mry(Some(1))), Ordering::Equal);
        assert_eq!(Mry(Some(0)).cmp(&Mry::none()), Ordering::Equal);
        assert_eq!(Mry::none().cmp(&Mry::none()), Ordering::Equal);
    }

    #[test]
    fn mry_always_equal_partial_ord() {
        assert_eq!(
            Mry(Some(0)).partial_cmp(&Mry(Some(1))),
            Some(Ordering::Equal)
        );
        assert_eq!(
            Mry(Some(0)).partial_cmp(&Mry::none()),
            Some(Ordering::Equal)
        );
        assert_eq!(Mry::none().partial_cmp(&Mry::none()), Some(Ordering::Equal));
    }

    #[test]
    fn mry_hash_returns_consistent_value() {
        let mut set = HashSet::new();
        set.insert(Mry(Some(0)));
        set.insert(Mry(Some(1)));
        set.insert(Mry::none());
        assert_eq!(set.len(), 1);
    }

    #[test]
    fn delete_mock_data_on_drop() {
        let inner_id;
        {
            let id = Mry::generate();
            inner_id = id.0.unwrap();
            MOCK_DATA.lock().insert(inner_id, "mock", 1);
        }
        assert!(!MOCK_DATA.lock().contains_key(inner_id));
    }

    #[test]
    fn delete_clone_count_on_drop() {
        let inner_id;
        {
            let id = Mry::generate().clone();
            inner_id = id.0.unwrap();
            assert!(CLONE_COUNT.lock().contains_key(&inner_id));
        }
        assert!(!CLONE_COUNT.lock().contains_key(&inner_id));
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
