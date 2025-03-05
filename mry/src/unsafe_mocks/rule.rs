use std::{cell::RefCell, rc::Rc};

use crate::Output;

use super::{behavior::UnsafeBehavior, matcher::UnsafeMatcher};

pub(crate) struct UnsafeRule<I, O> {
    pub matcher: Rc<RefCell<UnsafeMatcher<I>>>,
    pub behavior: UnsafeBehavior<I, O>,
}

impl<I, O> UnsafeRule<I, O> {
    pub fn matches(&self, input: &I) -> bool {
        self.matcher.borrow().matches(input)
    }
    pub fn call_behavior(&mut self, input: &I) -> Output<O> {
        self.behavior.called(input)
    }
}
