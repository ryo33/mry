mod log;
use std::{iter::repeat, sync::Arc};

pub use log::*;

use parking_lot::Mutex;

use crate::{times::Times, Behavior, Matcher, Output, Rule};

pub struct Mock<I, O> {
    pub name: &'static str,
    pub log: Logs<I>,
    rules: Vec<Rule<I, O>>,
}

impl<I, O> Mock<I, O> {
    pub fn new(name: &'static str) -> Self {
        Self {
            name,
            log: Default::default(),
            rules: Default::default(),
        }
    }
}

impl<I, O> Mock<I, O> {
    pub(crate) fn returns_with(
        &mut self,
        matcher: Arc<Mutex<Matcher<I>>>,
        behavior: Behavior<I, O>,
    ) {
        self.rules.push(Rule { matcher, behavior });
    }

    pub(crate) fn returns_once(&mut self, matcher: Arc<Mutex<Matcher<I>>>, ret: O) {
        self.returns_with(matcher, Behavior::Once(Mutex::new(Some(ret))))
    }

    pub(crate) fn calls_real_impl(&mut self, matcher: Arc<Mutex<Matcher<I>>>) {
        self.rules.push(Rule {
            matcher,
            behavior: Behavior::CallsRealImpl,
        })
    }
}

impl<I: 'static, O> Mock<I, O> {
    pub(crate) fn assert_called(&self, matcher: &Matcher<I>, times: Times) {
        self.log.assert_called(self.name, matcher, times);
    }

    pub(crate) fn record_call(&mut self, input: Arc<Mutex<I>>) {
        self.log.push(input);
    }
}

impl<I, O> Mock<I, O> {
    pub(crate) fn find_mock_output(&mut self, input: &I) -> Option<O> {
        for rule in &mut self.rules {
            if !rule.matches(input) {
                continue;
            }
            return match rule.call_behavior(input) {
                Output::Found(output) => Some(output),
                Output::CallsRealImpl => None,
                Output::ErrorCalledOnce => {
                    panic!("{} was called more than once", self.name)
                }
            };
        }
        panic!("mock not found for {}", self.name)
    }
}

impl<I, O> Mock<I, O>
where
    I: 'static,
    O: Clone + Send + 'static,
{
    pub(crate) fn returns(&mut self, matcher: Arc<Mutex<Matcher<I>>>, ret: O) {
        self.returns_with(matcher, Behavior::Const(Mutex::new(Box::new(repeat(ret)))))
    }
}

impl<I, R> Mock<I, std::pin::Pin<Box<dyn std::future::Future<Output = R> + Send + 'static>>>
where
    I: 'static,
    R: Clone + Send + 'static,
{
    pub(crate) fn returns_ready(&mut self, matcher: Arc<Mutex<Matcher<I>>>, ret: R) {
        self.returns_with(
            matcher,
            Behavior::Const(Mutex::new(Box::new(
                repeat(ret).map(|r| Box::pin(std::future::ready(r)) as _),
            ))),
        )
    }
}

impl<I, R> Mock<I, std::pin::Pin<Box<dyn std::future::Future<Output = R> + Send + 'static>>>
where
    I: 'static,
    R: Send + 'static,
{
    pub(crate) fn returns_ready_once(&mut self, matcher: Arc<Mutex<Matcher<I>>>, ret: R) {
        self.returns_with(
            matcher,
            Behavior::Once(Mutex::new(Some(Box::pin(std::future::ready(ret))))),
        )
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::Behavior1;

    #[test]
    fn returns_with() {
        let mut mock = Mock::<(usize,), String>::new("a");
        mock.returns_with(
            Matcher::any().wrapped(),
            Behavior1::from(|a| "a".repeat(a)).into(),
        );

        assert_eq!(mock.find_mock_output(&(3,)), "aaa".to_string().into());
    }

    #[test]
    fn returns() {
        let mut mock = Mock::<(usize,), String>::new("a");
        mock.returns(Matcher::any().wrapped(), "a".repeat(3));

        assert_eq!(mock.find_mock_output(&(3,)), "aaa".to_string().into());

        // allows called multiple times
        assert_eq!(mock.find_mock_output(&(3,)), "aaa".to_string().into());
    }

    #[test]
    #[should_panic(expected = "mock not found for a")]
    fn returns_with_never() {
        let mut mock = Mock::<(usize,), String>::new("a");
        mock.returns_with(
            Matcher::never().wrapped(),
            Behavior1::from(|a| "a".repeat(a)).into(),
        );

        mock.find_mock_output(&(3,));
    }

    #[test]
    fn returns_with_always() {
        let mut mock = Mock::<(usize,), String>::new("a");
        mock.returns_with(
            Matcher::any().wrapped(),
            Behavior1::from(|a| "a".repeat(a)).into(),
        );

        assert_eq!(mock.find_mock_output(&(3,)), "aaa".to_string().into());
    }

    #[test]
    #[should_panic(expected = "mock not found for a")]
    fn returns_never() {
        let mut mock = Mock::<(usize,), String>::new("a");
        mock.returns(Matcher::never().wrapped(), "a".repeat(3));

        mock.find_mock_output(&(3,));
    }

    #[test]
    fn returns_always() {
        let mut mock = Mock::<(usize,), String>::new("a");
        mock.returns(Matcher::any().wrapped(), "a".repeat(3));

        assert_eq!(mock.find_mock_output(&(3,)), "aaa".to_string().into());
    }

    #[test]
    fn calls_real_impl() {
        let mut mock = Mock::<(usize,), String>::new("a");
        mock.calls_real_impl(Arc::new(Mutex::new(Matcher::new_eq((3,)))));

        assert_eq!(mock.find_mock_output(&(3,)), None);
    }

    #[test]
    #[should_panic(expected = "mock not found for a")]
    fn calls_real_impl_never() {
        let mut mock = Mock::<(usize,), String>::new("a");
        mock.calls_real_impl(Arc::new(Mutex::new(Matcher::new_eq((3,)))));

        mock.find_mock_output(&(2,));
    }

    #[test]
    #[should_panic(expected = "a was called more than once")]
    fn panic_on_once_called_multiple_time() {
        let mut mock = Mock::<(usize,), String>::new("a");
        mock.returns_once(Matcher::any().wrapped(), "a".repeat(3));

        mock.find_mock_output(&(3,));
        mock.find_mock_output(&(3,));
    }
}
