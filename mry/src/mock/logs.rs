use crate::Matcher;

#[derive(Debug, PartialEq)]
pub struct Logs<I>(Vec<I>);

impl<I: PartialEq + Clone> Logs<I> {
    pub fn push(&mut self, item: I) {
        self.0.push(item);
    }

    pub fn is_empty(&self) -> bool {
        self.0.len() == 0
    }

    pub fn filter_matches(&self, matcher: &Matcher<I>) -> Self {
        Self(
            self.0
                .iter()
                .filter(|log| matcher.matches(log))
                .cloned()
                .collect(),
        )
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
    fn is_empty() {
        assert!(Logs::<u8>(vec![]).is_empty());
        assert!(!Logs::<u8>(vec![1]).is_empty());
    }

    #[test]
    fn filter_matches() {
        let logs = Logs(vec![1, 2, 2, 3, 4, 2]);
        assert_eq!(logs.filter_matches(&Matcher::Eq(2)), Logs(vec![2, 2, 2]));
    }

    #[test]
    fn times() {
        // TODO
    }
}
