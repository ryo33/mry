use std::ops::RangeBounds;

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
            panic!("{} was not called {} time(s)", self.name, times);
        }
    }

    /// Assert that the mock is called times within the range
    pub fn times_within<T: RangeBounds<usize>>(&self, range: T) {
        let len = self.logs.0.len();
        if !range.contains(&len) {
            panic!("{} was called {} time(s)", self.name, len);
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
    #[should_panic(expected = "Cat was not called 3 time(s)")]
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
    #[should_panic(expected = "Cat was called 2 time(s)")]
    fn times_within_panics() {
        let result = MockResult {
            name: "Cat",
            logs: Logs(vec![1, 2]),
        };
        result.times_within(3..5);
    }
}
