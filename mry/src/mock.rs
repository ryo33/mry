use std::sync::Mutex;

use crate::Behavior;

#[derive(Default)]
pub struct Mock<I, O> {
    logs: Mutex<Vec<I>>,
    rules: Vec<Behavior<I, O>>,
}

impl<I: Clone, O> Mock<I, O> {
    pub fn behaves<R: Into<Behavior<I, O>>>(&mut self, rule: R) {
        self.rules.push(rule.into());
    }

    pub fn _inner_behave(&self, input: I) -> O {
        self.logs.lock().unwrap().push(input.clone());
        for rule in self.rules.iter() {
            if let Some(output) = rule.called(input.clone()) {
                return output;
            }
        }
        panic!("mock not found");
    }
}

impl<I: PartialEq, O> Mock<I, O> {
    pub fn assert_called_with(&self, input: I) {
        let mut count = 0;
        self.logs.lock().unwrap().iter().for_each(|log| {
            if log == &input {
                count += 1;
            }
        });
        if count == 0 {
            panic!("not called!");
        }
    }
}
