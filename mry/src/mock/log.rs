use std::{ops::Deref, sync::Arc};

use parking_lot::Mutex;

use crate::{times::Times, Matcher};

pub struct Logs<I>(Vec<Arc<Mutex<I>>>);

impl<I> Logs<I> {
    pub(crate) fn push(&mut self, item: Arc<Mutex<I>>) {
        self.0.push(item);
    }

    pub fn filter_matches(&self, matcher: &Matcher<I>) -> Self {
        Self(
            self.0
                .iter()
                .filter(|log| matcher.matches(&log.lock()))
                .cloned()
                .collect(),
        )
    }

    pub(crate) fn assert_called(&self, name: &str, matcher: &Matcher<I>, times: Times) -> Self {
        let logs = self.filter_matches(matcher);
        let actual = logs.0.len();
        if !times.contains(&actual) {
            panic!(
                "Expected {} to be called {} times, but it was called {} times",
                name, times, actual,
            );
        }
        logs
    }

    pub fn iter(&self) -> impl Iterator<Item = impl Deref<Target = I> + '_> {
        self.0.iter().map(|log| log.lock())
    }
}

impl<I> Default for Logs<I> {
    fn default() -> Self {
        Self(Default::default())
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn filter_matches() {
        let mut logs = Logs::default();
        logs.push(Arc::new(Mutex::new(1)));
        logs.push(Arc::new(Mutex::new(2)));
        logs.push(Arc::new(Mutex::new(3)));
        logs.push(Arc::new(Mutex::new(2)));

        let matcher = Matcher::new_eq(2);

        let filtered = logs.filter_matches(&matcher);
        assert_eq!(filtered.0.len(), 2);
    }
}
