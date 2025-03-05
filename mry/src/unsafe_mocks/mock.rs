use std::{cell::RefCell, iter::repeat, rc::Rc};

use crate::{times::Times, Output};

use super::{
    behavior::UnsafeBehavior, log::UnsafeLogs, matcher::UnsafeMatcher, mockable::UnsafeMockableRet,
    rule::UnsafeRule,
};

pub struct UnsafeMock<I, O> {
    pub name: &'static str,
    pub log: UnsafeLogs<I>,
    rules: Vec<UnsafeRule<I, O>>,
}

impl<I, O> UnsafeMock<I, O> {
    pub fn new(name: &'static str) -> Self {
        Self {
            name,
            log: Default::default(),
            rules: Default::default(),
        }
    }
}

impl<I, O> UnsafeMock<I, O> {
    pub(crate) fn returns_with(
        &mut self,
        matcher: Rc<RefCell<UnsafeMatcher<I>>>,
        behavior: UnsafeBehavior<I, O>,
    ) {
        self.rules.push(UnsafeRule { matcher, behavior });
    }

    pub(crate) fn returns_once(&mut self, matcher: Rc<RefCell<UnsafeMatcher<I>>>, ret: O) {
        self.returns_with(matcher, UnsafeBehavior::Once(RefCell::new(Some(ret))))
    }

    pub(crate) fn calls_real_impl(&mut self, matcher: Rc<RefCell<UnsafeMatcher<I>>>) {
        self.rules.push(UnsafeRule {
            matcher,
            behavior: UnsafeBehavior::CallsRealImpl,
        })
    }
}

impl<I: 'static, O> UnsafeMock<I, O> {
    pub(crate) fn assert_called(&self, matcher: &UnsafeMatcher<I>, times: Times) {
        self.log.assert_called(self.name, matcher, times);
    }

    pub(crate) fn record_call(&mut self, input: Rc<RefCell<I>>) {
        self.log.push(input);
    }
}

impl<I, O> UnsafeMock<I, O> {
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

impl<I, O> UnsafeMock<I, O>
where
    I: 'static,
    O: Clone + UnsafeMockableRet,
{
    pub(crate) fn returns(&mut self, matcher: Rc<RefCell<UnsafeMatcher<I>>>, ret: O) {
        self.returns_with(
            matcher,
            UnsafeBehavior::Const(RefCell::new(Box::new(repeat(ret)))),
        )
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::unsafe_mocks::behavior::UnsafeBehavior1;

    #[test]
    fn returns_with() {
        let mut mock = UnsafeMock::<(usize,), String>::new("a");
        mock.returns_with(
            UnsafeMatcher::any().wrapped(),
            UnsafeBehavior1::from(|a| "a".repeat(a)).into(),
        );

        assert_eq!(mock.find_mock_output(&(3,)), "aaa".to_string().into());
    }

    #[test]
    fn returns() {
        let mut mock = UnsafeMock::<(usize,), String>::new("a");
        mock.returns(UnsafeMatcher::any().wrapped(), "a".repeat(3));

        assert_eq!(mock.find_mock_output(&(3,)), "aaa".to_string().into());

        // allows called multiple times
        assert_eq!(mock.find_mock_output(&(3,)), "aaa".to_string().into());
    }

    #[test]
    #[should_panic(expected = "mock not found for a")]
    fn returns_with_never() {
        let mut mock = UnsafeMock::<(usize,), String>::new("a");
        mock.returns_with(
            UnsafeMatcher::never().wrapped(),
            UnsafeBehavior1::from(|a| "a".repeat(a)).into(),
        );

        mock.find_mock_output(&(3,));
    }

    #[test]
    fn returns_with_always() {
        let mut mock = UnsafeMock::<(usize,), String>::new("a");
        mock.returns_with(
            UnsafeMatcher::any().wrapped(),
            UnsafeBehavior1::from(|a| "a".repeat(a)).into(),
        );

        assert_eq!(mock.find_mock_output(&(3,)), "aaa".to_string().into());
    }

    #[test]
    #[should_panic(expected = "mock not found for a")]
    fn returns_never() {
        let mut mock = UnsafeMock::<(usize,), String>::new("a");
        mock.returns(UnsafeMatcher::never().wrapped(), "a".repeat(3));

        mock.find_mock_output(&(3,));
    }

    #[test]
    fn returns_always() {
        let mut mock = UnsafeMock::<(usize,), String>::new("a");
        mock.returns(UnsafeMatcher::any().wrapped(), "a".repeat(3));

        assert_eq!(mock.find_mock_output(&(3,)), "aaa".to_string().into());
    }

    #[test]
    fn calls_real_impl() {
        let mut mock = UnsafeMock::<(usize,), String>::new("a");
        mock.calls_real_impl(Rc::new(RefCell::new(UnsafeMatcher::new_eq((3,)))));

        assert_eq!(mock.find_mock_output(&(3,)), None);
    }

    #[test]
    #[should_panic(expected = "mock not found for a")]
    fn calls_real_impl_never() {
        let mut mock = UnsafeMock::<(usize,), String>::new("a");
        mock.calls_real_impl(Rc::new(RefCell::new(UnsafeMatcher::new_eq((3,)))));

        mock.find_mock_output(&(2,));
    }

    #[test]
    #[should_panic(expected = "a was called more than once")]
    fn panic_on_once_called_multiple_time() {
        let mut mock = UnsafeMock::<(usize,), String>::new("a");
        mock.returns_once(UnsafeMatcher::any().wrapped(), "a".repeat(3));

        mock.find_mock_output(&(3,));
        mock.find_mock_output(&(3,));
    }
}
