mod logs;
use std::fmt::Debug;
use std::iter::repeat;

pub use logs::*;

use parking_lot::{Mutex, RwLock};

use crate::{times::Times, Behavior, Matcher, Output, Rule};

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

impl<I: Clone + PartialEq + Debug, O: Debug> Mock<I, O> {
    pub(crate) fn returns_with(&mut self, matcher: Matcher<I>, behavior: Behavior<I, O>) {
        self.rules.push(Rule {
            matcher,
            behavior: behavior,
        });
    }

    pub(crate) fn calls_real_impl(&mut self, matcher: Matcher<I>) {
        self.rules.push(Rule {
            matcher,
            behavior: Behavior::CallsRealImpl,
        })
    }

    pub(crate) fn assert_called(&self, matcher: Matcher<I>, times: Times) -> Logs<I> {
        let logs = self.logs.lock().filter_matches(&matcher);
        if !times.contains(&dbg!(logs.0.len())) {
            panic!("{} was not called\n{:?}", self.name, *self.logs.lock())
        }
        logs
    }

    pub(crate) fn record_call_and_find_mock_output(&mut self, input: I) -> Option<O> {
        self.logs.lock().push(input.clone());
        for rule in &mut self.rules {
            match rule.called(&input) {
                Output::Found(output) => return Some(output),
                Output::NotMatches => {}
                Output::CallsRealImpl => return None,
            };
        }
        panic!("mock not found for {}\n{:?}", self.name, self.rules)
    }
}

impl<I, O> Mock<I, O>
where
    I: Clone + PartialEq + Debug + Send + Sync + 'static,
    O: Clone + Debug + Send + Sync + 'static,
{
    pub(crate) fn returns(&mut self, matcher: Matcher<I>, ret: O) {
        self.returns_with(matcher, Behavior::Const(RwLock::new(Box::new(repeat(ret)))))
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::Behavior1;

    #[test]
    fn returns_with() {
        let mut mock = Mock::<usize, String>::new("a");
        mock.returns_with(Matcher::Any, Behavior1::from(|a| "a".repeat(a)).into());

        assert_eq!(
            mock.record_call_and_find_mock_output(3),
            "aaa".to_string().into()
        );
    }

    #[test]
    fn returns() {
        let mut mock = Mock::<usize, String>::new("a");
        mock.returns(Matcher::Any, "a".repeat(3));

        assert_eq!(
            mock.record_call_and_find_mock_output(3),
            "aaa".to_string().into()
        );

        // allows called multiple times
        assert_eq!(
            mock.record_call_and_find_mock_output(3),
            "aaa".to_string().into()
        );
    }

    #[test]
    #[should_panic(expected = "mock not found for a")]
    fn returns_with_never() {
        let mut mock = Mock::<usize, String>::new("a");
        mock.returns_with(Matcher::Never, Behavior1::from(|a| "a".repeat(a)).into());

        mock.record_call_and_find_mock_output(3);
    }

    #[test]
    fn returns_with_always() {
        let mut mock = Mock::<usize, String>::new("a");
        mock.returns_with(Matcher::Any, Behavior1::from(|a| "a".repeat(a)).into());

        assert_eq!(
            mock.record_call_and_find_mock_output(3),
            "aaa".to_string().into()
        );
    }

    #[test]
    #[should_panic(expected = "mock not found for a")]
    fn returns_never() {
        let mut mock = Mock::<usize, String>::new("a");
        mock.returns(Matcher::Never, "a".repeat(3));

        mock.record_call_and_find_mock_output(3);
    }

    #[test]
    fn returns_always() {
        let mut mock = Mock::<usize, String>::new("a");
        mock.returns(Matcher::Any, "a".repeat(3));

        assert_eq!(
            mock.record_call_and_find_mock_output(3),
            "aaa".to_string().into()
        );
    }

    #[test]
    fn calls_real_impl() {
        let mut mock = Mock::<usize, String>::new("a");
        mock.calls_real_impl(Matcher::Eq(3));

        assert_eq!(mock.record_call_and_find_mock_output(3), None);
    }

    #[test]
    #[should_panic(expected = "mock not found for a")]
    fn calls_real_impl_never() {
        let mut mock = Mock::<usize, String>::new("a");
        mock.calls_real_impl(Matcher::Eq(3));

        mock.record_call_and_find_mock_output(2);
    }

    #[test]
    fn assert_called_with() {
        let mut mock = Mock::<usize, String>::new("a");
        mock.returns_with(Matcher::Any, Behavior1::from(|a| "a".repeat(a)).into());

        mock.record_call_and_find_mock_output(3);

        mock.assert_called(Matcher::Eq(3), Times::Exact(1));
    }

    #[test]
    #[should_panic(expected = "a was not called")]
    fn assert_called_with_not_eq() {
        let mut mock = Mock::<usize, String>::new("a");
        mock.returns_with(Matcher::Any, Behavior1::from(|a| "a".repeat(a)).into());

        mock.record_call_and_find_mock_output(3);

        mock.assert_called(Matcher::Eq(2), Times::Exact(1));
    }

    #[test]
    #[should_panic(expected = "a was not called")]
    fn assert_called_with_panics() {
        let mut mock = Mock::<usize, String>::new("a");
        mock.returns_with(Matcher::Any, Behavior1::from(|a| "a".repeat(a)).into());

        mock.assert_called(Matcher::Eq(3), Times::Exact(1));
    }

    #[test]
    #[should_panic(expected = "[1, 2, 2]")]
    fn assert_called_with_log() {
        let mut mock = Mock::<usize, String>::new("a");
        mock.returns_with(Matcher::Any, Behavior1::from(|a| "a".repeat(a)).into());
        mock.record_call_and_find_mock_output(1);
        mock.record_call_and_find_mock_output(2);
        mock.record_call_and_find_mock_output(2);

        mock.assert_called(Matcher::Eq(3), Times::Exact(1));
    }

    #[test]
    fn assert_called_returns_logs() {
        let mut mock = Mock::<usize, String>::new("a");
        mock.returns_with(Matcher::Any, Behavior1::from(|a| "a".repeat(a)).into());

        mock.record_call_and_find_mock_output(3);
        mock.record_call_and_find_mock_output(3);
        mock.record_call_and_find_mock_output(2);

        assert_eq!(
            mock.assert_called(Matcher::Any, Times::Exact(3)),
            Logs(vec![3, 3, 2]),
        );
    }

    #[test]
    fn assert_called_returns_logs_matching() {
        let mut mock = Mock::<usize, String>::new("a");
        mock.returns_with(Matcher::Any, Behavior1::from(|a| "a".repeat(a)).into());

        mock.record_call_and_find_mock_output(2);
        mock.record_call_and_find_mock_output(3);
        mock.record_call_and_find_mock_output(3);
        mock.record_call_and_find_mock_output(2);

        assert_eq!(
            mock.assert_called(Matcher::Eq(2), Times::Exact(2)),
            Logs(vec![2, 2]),
        );
    }

    #[test]
    #[should_panic(
        expected = "[Rule { matcher: Eq(3), behavior: Function(_) }, Rule { matcher: Eq(3), behavior: CallsRealImpl }]"
    )]
    fn mock_not_found_with_rules() {
        let mut mock = Mock::<usize, String>::new("a");
        mock.returns_with(
            Matcher::Eq(3),
            Behavior::Function(Box::new(|_| "42".to_string())),
        );
        mock.calls_real_impl(Matcher::Eq(3));

        mock.record_call_and_find_mock_output(2);
    }
}
