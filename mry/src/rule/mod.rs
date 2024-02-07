mod behavior;
mod matcher;

use std::sync::Arc;

pub use behavior::*;
pub use matcher::*;
use parking_lot::Mutex;

pub(crate) struct Rule<I, O> {
    pub matcher: Arc<Mutex<Matcher<I>>>,
    pub behavior: Behavior<I, O>,
}

impl<I, O> Rule<I, O> {
    pub fn matches(&self, input: &I) -> bool {
        self.matcher.lock().matches(input)
    }
    pub fn call_behavior(&mut self, input: I) -> Output<O> {
        self.behavior.called(input)
    }
}
