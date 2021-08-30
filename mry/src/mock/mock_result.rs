use std::{fmt::Debug, ops::RangeBounds};

use crate::Logs;

#[derive(Debug, PartialEq)]
pub struct MockResult<I> {
    pub(crate) name: &'static str,
    pub(crate) logs: Logs<I>,
}

impl<I: Clone> MockResult<I> {
    /// Assert that the mock is called exact times.
    pub fn times(&self, times: usize) {
        if self.logs.0.len() != times {
            panic!(
                "{} was called {} times not {} times",
                self.name,
                self.logs.0.len(),
                times
            );
        }
    }

    /// Assert that the mock is called times within the range
    pub fn times_within<T: RangeBounds<usize> + Debug>(&self, range: T) {
        let len = self.logs.0.len();
        if !range.contains(&len) {
            panic!(
                "{} was called {} times and is out of {:?}",
                self.name, len, range
            );
        }
    }

    pub fn logs(&self) -> Vec<I> {
        self.logs.0.clone()
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn times() {
        let result = MockResult {
            name: "Cat",
            logs: Logs(vec![1, 2, 3]),
        };
        result.times(3);
    }

    #[test]
    #[should_panic(expected = "Cat was called 2 times not 3 times")]
    fn times_panics() {
        let result = MockResult {
            name: "Cat",
            logs: Logs(vec![1, 2]),
        };
        result.times(3);
    }

    #[test]
    fn times_within() {
        let result = MockResult {
            name: "Cat",
            logs: Logs(vec![1, 2]),
        };
        result.times_within(0..=2);
        result.times_within(2..=4);
    }

    #[test]
    #[should_panic(expected = "Cat was called 2 times and is out of 3..5")]
    fn times_within_panics() {
        let result = MockResult {
            name: "Cat",
            logs: Logs(vec![1, 2]),
        };
        result.times_within(3..5);
    }
}
