use std::{cell::RefCell, ops::Deref, rc::Rc};

use crate::times::Times;

use super::matcher::UnsafeMatcher;

pub struct UnsafeLogs<I>(Vec<Rc<RefCell<I>>>);

impl<I: 'static> UnsafeLogs<I> {
    pub(crate) fn push(&mut self, item: Rc<RefCell<I>>) {
        self.0.push(item);
    }

    pub fn filter_matches(&self, matcher: &UnsafeMatcher<I>) -> Self {
        Self(
            self.0
                .iter()
                .filter(|log| matcher.matches(&log.borrow()))
                .cloned()
                .collect(),
        )
    }

    pub(crate) fn assert_called(
        &self,
        name: &str,
        matcher: &UnsafeMatcher<I>,
        times: Times,
    ) -> Self {
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
        self.0.iter().map(|log| log.borrow())
    }
}

impl<I> Default for UnsafeLogs<I> {
    fn default() -> Self {
        Self(Default::default())
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn filter_matches() {
        let mut logs = UnsafeLogs::default();
        logs.push(Rc::new(RefCell::new(1)));
        logs.push(Rc::new(RefCell::new(2)));
        logs.push(Rc::new(RefCell::new(3)));
        logs.push(Rc::new(RefCell::new(2)));

        let matcher = UnsafeMatcher::new_eq(2);

        let filtered = logs.filter_matches(&matcher);
        assert_eq!(filtered.0.len(), 2);
    }
}
