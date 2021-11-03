use std::{
    any::Any,
    collections::hash_map::DefaultHasher,
    hash::{Hash, Hasher},
};

pub type BoxMockKey = Box<dyn MockKey + Send + Sync>;

pub trait MockKey {
    fn hash(&self) -> u64;
    fn eq(&self, other: &dyn MockKey) -> bool;
    fn as_any(&self) -> &dyn Any;
    fn clone(&self) -> BoxMockKey;
}

impl<T: Eq + Hash + Clone + Send + Sync + 'static> MockKey for T {
    fn hash(&self) -> u64 {
        let mut h = DefaultHasher::new();
        Hash::hash(&self, &mut h);
        h.finish()
    }

    fn eq(&self, other: &dyn MockKey) -> bool {
        other
            .as_any()
            .downcast_ref()
            .into_iter()
            .any(|other| <T as PartialEq>::eq(self, other))
    }

    fn clone(&self) -> BoxMockKey {
        Box::new(<T as Clone>::clone(self))
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}

impl Eq for Box<dyn MockKey + Send + Sync> {}

impl PartialEq for Box<dyn MockKey + Send + Sync> {
    fn eq(&self, other: &Self) -> bool {
        MockKey::eq(self.as_ref(), other.as_ref())
    }
}

impl Hash for Box<dyn MockKey + Send + Sync> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        state.write_u64(MockKey::hash(self.as_ref()));
    }
}

#[cfg(test)]
pub(crate) fn key_a() -> BoxMockKey {
    fn a() {}
    Box::new(a as fn())
}

#[cfg(test)]
mod tests {
    use super::BoxMockKey;

    fn a() {}
    fn b() {}
    fn mock_key_a() -> BoxMockKey {
        Box::new(a as fn())
    }
    fn mock_key_b() -> BoxMockKey {
        Box::new(b as fn())
    }

    #[test]
    fn eq_returns_true_if_same_function() {
        assert!(mock_key_a() == mock_key_a());
    }

    #[test]
    fn eq_returns_false_if_not_same_function() {
        assert!(mock_key_a() != mock_key_b());
    }

    #[test]
    fn same_hash_if_same_function() {
        assert_eq!(mock_key_a().hash(), mock_key_a().hash());
    }

    #[test]
    fn different_hash_if_not_same_function() {
        assert_ne!(mock_key_a().hash(), mock_key_b().hash());
    }
}
