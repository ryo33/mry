mod logs;
mod mock_result;
use std::fmt::Debug;

pub use logs::*;
pub use mock_result::*;

use parking_lot::Mutex;

use crate::{Behavior, Matcher, Rule};

#[derive(PartialEq)]
pub enum CallsRealImpl {
    Yes,
    No,
}

pub struct Mock<I, O> {
    pub name: &'static str,
    calls_real_impl: CallsRealImpl,
    logs: Mutex<Logs<I>>,
    rules: Vec<Rule<I, O>>,
}

impl<I, O> Mock<I, O> {
    pub fn new(name: &'static str) -> Self {
        Self {
            name,
            calls_real_impl: CallsRealImpl::No,
            logs: Default::default(),
            rules: Default::default(),
        }
    }
}

impl<I: Clone + PartialEq + Debug, O: Clone> Mock<I, O> {
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

    pub(crate) fn returns(&mut self, ret: O) {
        self.rules.push(Rule {
            matcher: Matcher::Always,
            behavior: Behavior::Const(ret),
        });
    }

    pub(crate) fn returns_when<M: Into<Matcher<I>>>(&mut self, matcher: M, ret: O) {
        self.rules.push(Rule {
            matcher: matcher.into(),
            behavior: Behavior::Const(ret),
        });
    }

    pub(crate) fn calls_real_impl(&mut self) {
        self.calls_real_impl = CallsRealImpl::Yes;
    }

    pub(crate) fn assert_called_with<T: Into<Matcher<I>>>(&self, matcher: T) -> MockResult<I> {
        let matcher = matcher.into();
        let logs = self.handle_assert_called(&matcher, || {
            panic!("{} was not called with {:?}", self.name, matcher)
        });
        MockResult {
            name: self.name,
            logs,
        }
    }
    pub(crate) fn assert_called(&self) -> MockResult<I> {
        let logs =
            self.handle_assert_called(&Matcher::Always, || panic!("{} was not called", self.name));
        MockResult {
            name: self.name,
            logs,
        }
    }

    fn handle_assert_called(&self, matcher: &Matcher<I>, f: impl FnOnce()) -> Logs<I> {
        let logs = self.logs.lock().filter_matches(matcher);
        if logs.is_empty() {
            f();
        }
        logs
    }

    pub fn _inner_called(&mut self, input: I) -> Option<O> {
        self.logs.lock().push(input.clone());
        for rule in &mut self.rules {
            if let Some(output) = rule.called(&input) {
                return Some(output);
            }
        }
        if self.calls_real_impl == CallsRealImpl::Yes {
            return None;
        }
        panic!("mock not found for {}", self.name);
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::Behavior1;

    #[test]
    fn behaves() {
        let mut mock = Mock::<usize, String>::new("a");
        mock.behaves(Behavior1::from(|a| "a".repeat(a)));

        assert_eq!(mock._inner_called(3), "aaa".to_string().into());
    }

    #[test]
    fn returns() {
        let mut mock = Mock::<usize, String>::new("a");
        mock.returns("a".repeat(3));

        assert_eq!(mock._inner_called(3), "aaa".to_string().into());
    }

    #[test]
    #[should_panic(expected = "mock not found for a")]
    fn behaves_when_never() {
        let mut mock = Mock::<usize, String>::new("a");
        mock.behaves_when(Matcher::Never, Behavior1::from(|a| "a".repeat(a)));

        mock._inner_called(3);
    }

    #[test]
    fn behaves_when_always() {
        let mut mock = Mock::<usize, String>::new("a");
        mock.behaves_when(Matcher::Always, Behavior1::from(|a| "a".repeat(a)));

        assert_eq!(mock._inner_called(3), "aaa".to_string().into());
    }

    #[test]
    #[should_panic(expected = "mock not found for a")]
    fn returns_when_never() {
        let mut mock = Mock::<usize, String>::new("a");
        mock.returns_when(Matcher::Never, "a".repeat(3));

        assert_eq!(mock._inner_called(3), "aaa".to_string().into());
    }

    #[test]
    fn returns_when_always() {
        let mut mock = Mock::<usize, String>::new("a");
        mock.returns_when(Matcher::Always, "a".repeat(3));

        assert_eq!(mock._inner_called(3), "aaa".to_string().into());
    }

    #[test]
    fn calls_real_impl() {
        let mut mock = Mock::<usize, String>::new("a");
        mock.calls_real_impl();

        assert_eq!(mock._inner_called(3), None);
    }

    #[test]
    fn assert_called_with() {
        let mut mock = Mock::<usize, String>::new("a");
        mock.behaves_when(Matcher::Always, Behavior1::from(|a| "a".repeat(a)));

        mock._inner_called(3);

        mock.assert_called_with(3);
    }

    #[test]
    #[should_panic(expected = "a was not called")]
    fn assert_called_with_panics() {
        let mut mock = Mock::<usize, String>::new("a");
        mock.behaves_when(Matcher::Always, Behavior1::from(|a| "a".repeat(a)));

        mock.assert_called_with(3);
    }

    #[test]
    fn assert_called() {
        let mut mock = Mock::<usize, String>::new("a");
        mock.behaves_when(Matcher::Always, Behavior1::from(|a| "a".repeat(a)));

        mock._inner_called(3);

        mock.assert_called();
    }

    #[test]
    #[should_panic(expected = "a was not called")]
    fn assert_called_panics() {
        let mut mock = Mock::<usize, String>::new("a");
        mock.behaves_when(Matcher::Always, Behavior1::from(|a| "a".repeat(a)));

        mock.assert_called();
    }

    #[test]
    fn assert_called_returns_logs() {
        let mut mock = Mock::<usize, String>::new("a");
        mock.behaves_when(Matcher::Always, Behavior1::from(|a| "a".repeat(a)));

        mock._inner_called(3);
        mock._inner_called(3);
        mock._inner_called(2);

        assert_eq!(
            mock.assert_called(),
            MockResult {
                name: "a",
                logs: Logs(vec![3, 3, 2]),
            }
        );
    }

    #[test]
    fn assert_called_with_returns_logs() {
        let mut mock = Mock::<usize, String>::new("a");
        mock.behaves_when(Matcher::Always, Behavior1::from(|a| "a".repeat(a)));

        mock._inner_called(2);
        mock._inner_called(3);
        mock._inner_called(3);
        mock._inner_called(2);

        assert_eq!(
            mock.assert_called_with(2),
            MockResult {
                name: "a",
                logs: Logs(vec![2, 2]),
            }
        );
    }
}
