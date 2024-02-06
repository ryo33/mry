use crate::{times::Times, Matcher};

#[derive(Debug, PartialEq)]
pub struct Logs<I>(pub Vec<I>);

impl<I: PartialEq + Clone> Logs<I> {
    pub(crate) fn push(&mut self, item: I) {
        self.0.push(item);
    }

    pub(crate) fn filter_matches(&self, matcher: &Matcher<I>) -> Self {
        Self(
            self.0
                .iter()
                .filter(|log| matcher.matches(log))
                .cloned()
                .collect(),
        )
    }

    pub(crate) fn assert_called(&self, name: &str, matcher: &Matcher<I>, times: Times) -> Logs<I> {
        let logs = self.filter_matches(matcher);
        if !times.contains(&logs.0.len()) {
            panic!("{} was not called", name)
        }
        logs
    }
}

impl<I> Default for Logs<I> {
    fn default() -> Self {
        Self(Default::default())
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn filter_matches() {
        let logs = Logs(vec![1, 2, 2, 3, 4, 2]);
        assert_eq!(logs.filter_matches(&Matcher::Eq(2)), Logs(vec![2, 2, 2]));
    }
}
