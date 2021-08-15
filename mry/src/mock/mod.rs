mod logs;
use std::fmt::Debug;

pub use logs::*;

use parking_lot::Mutex;

use crate::{Behavior, Matcher, Rule};

pub struct Mock<I, O> {
    pub name: &'static str,
    logs: Mutex<Logs<I>>,
    rules: Vec<Rule<I, O>>,
}

impl<I, O> Mock<I, O> {
    pub fn new(name: &'static str) -> Self {
        Self {
            name,
            logs: Default::default(),
            rules: Default::default(),
        }
    }
}

impl<I: Clone + PartialEq + Debug, O> Mock<I, O> {
    pub(crate) fn behaves<B: Into<Behavior<I, O>>>(&mut self, behavior: B) {
        self.rules.push(Rule {
            matcher: Matcher::Always,
            behavior: behavior.into(),
        });
    }

    pub(crate) fn behaves_when<M: Into<Matcher<I>>, B: Into<Behavior<I, O>>>(
        &mut self,
        matcher: M,
        behavior: B,
    ) {
        self.rules.push(Rule {
            matcher: matcher.into(),
            behavior: behavior.into(),
        });
    }
    pub(crate) fn assert_called_with<T: Into<Matcher<I>>>(&self, matcher: T) {
        let matcher = matcher.into();
        self.handle_assert_called(&matcher, || {
            panic!("{} was not called with {:?}", self.name, matcher)
        });
    }
    pub(crate) fn assert_called(&self) {
        self.handle_assert_called(&Matcher::Always, || panic!("{} was not called", self.name));
    }

    fn handle_assert_called(&self, matcher: &Matcher<I>, f: impl FnOnce()) -> Logs<I> {
        let logs = self.logs.lock().filter_matches(matcher);
        if logs.is_empty() {
            f();
        }
        logs
    }

    pub fn _inner_called(&mut self, input: &I) -> O {
        self.logs.lock().push(input.clone());
        for rule in &mut self.rules {
            if let Some(output) = rule.called(input) {
                return output;
            }
        }
        panic!("mock not found for {}", self.name);
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn behaves() {
        let mut mock = Mock::<usize, String>::new("a");
        mock.behaves(|a| "a".repeat(a));

        assert_eq!(mock._inner_called(&3), "aaa".to_string());
    }

    #[test]
    #[should_panic(expected = "mock not found for a")]
    fn behaves_when_never() {
        let mut mock = Mock::<usize, String>::new("a");
        mock.behaves_when(Matcher::Never, |a| "a".repeat(a));

        mock._inner_called(&3);
    }

    #[test]
    fn behaves_when_always() {
        let mut mock = Mock::<usize, String>::new("a");
        mock.behaves_when(Matcher::Always, |a| "a".repeat(a));

        assert_eq!(mock._inner_called(&3), "aaa".to_string());
    }

    #[test]
    fn assert_called_with() {
        let mut mock = Mock::<usize, String>::new("a");
        mock.behaves_when(Matcher::Always, |a| "a".repeat(a));

        mock._inner_called(&3);

        mock.assert_called_with(3);
    }

    #[test]
    #[should_panic(expected = "a was not called")]
    fn assert_called_with_panics() {
        let mut mock = Mock::<usize, String>::new("a");
        mock.behaves_when(Matcher::Always, |a| "a".repeat(a));

        mock.assert_called_with(3);
    }

    #[test]
    fn assert_called() {
        let mut mock = Mock::<usize, String>::new("a");
        mock.behaves_when(Matcher::Always, |a| "a".repeat(a));

        mock._inner_called(&3);

        mock.assert_called();
    }

    #[test]
    #[should_panic(expected = "a was not called")]
    fn assert_called_panics() {
        let mut mock = Mock::<usize, String>::new("a");
        mock.behaves_when(Matcher::Always, |a| "a".repeat(a));

        mock.assert_called();
    }
}
