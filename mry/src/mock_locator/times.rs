use std::ops::{Bound, Range, RangeBounds, RangeFrom, RangeInclusive, RangeTo};

#[doc(hidden)]
#[derive(Debug)]
pub enum Times {
    Exact(usize),
    Range((Bound<usize>, Bound<usize>)),
}

impl Times {
    pub(crate) fn contains(&self, count: &usize) -> bool {
        match self {
            Times::Exact(n) => count == n,
            Times::Range(range) => range.contains(count),
        }
    }
}

impl From<usize> for Times {
    fn from(times: usize) -> Self {
        Times::Exact(times)
    }
}

impl From<Range<usize>> for Times {
    fn from(range: Range<usize>) -> Self {
        Times::Range((range.start_bound().cloned(), range.end_bound().cloned()))
    }
}

impl From<RangeFrom<usize>> for Times {
    fn from(range: RangeFrom<usize>) -> Self {
        Times::Range((range.start_bound().cloned(), range.end_bound().cloned()))
    }
}

impl From<RangeTo<usize>> for Times {
    fn from(range: RangeTo<usize>) -> Self {
        Times::Range((range.start_bound().cloned(), range.end_bound().cloned()))
    }
}

impl From<RangeInclusive<usize>> for Times {
    fn from(range: RangeInclusive<usize>) -> Self {
        Times::Range((range.start_bound().cloned(), range.end_bound().cloned()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn exact() {
        let times = Times::Exact(2);
        assert_eq!(times.contains(&2), true);
        assert_eq!(times.contains(&1), false);
        assert_eq!(times.contains(&3), false);
    }

    #[test]
    fn range() {
        let times = Times::Range((Bound::Included(2), Bound::Excluded(4)));
        assert_eq!(times.contains(&1), false);
        assert_eq!(times.contains(&2), true);
        assert_eq!(times.contains(&3), true);
        assert_eq!(times.contains(&4), false);
        assert_eq!(times.contains(&5), false);
    }
}
