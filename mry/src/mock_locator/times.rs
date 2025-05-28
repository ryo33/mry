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

use std::fmt;

impl fmt::Display for Times {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Times::Exact(n) => write!(f, "{n}"),
            Times::Range((start, end)) => {
                let start = match start {
                    Bound::Included(n) => format!("{n}<="),
                    Bound::Excluded(n) => format!("{n}<"),
                    Bound::Unbounded => String::from(""),
                };
                let end = match end {
                    Bound::Included(n) => format!("<={n}"),
                    Bound::Excluded(n) => format!("<{n}"),
                    Bound::Unbounded => String::from(""),
                };
                write!(f, "{start}x{end}")
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn exact() {
        let times = Times::Exact(2);
        assert!(times.contains(&2));
        assert!(!times.contains(&1));
        assert!(!times.contains(&3));
    }

    #[test]
    fn range() {
        let times = Times::Range((Bound::Included(2), Bound::Excluded(4)));
        assert!(!times.contains(&1));
        assert!(times.contains(&2));
        assert!(times.contains(&3));
        assert!(!times.contains(&4));
        assert!(!times.contains(&5));
    }

    #[test]
    fn display() {
        assert_eq!(Times::from(2).to_string(), "2");
        assert_eq!(Times::from(1..2).to_string(), "1<=x<2");
        assert_eq!(Times::from(1..=2).to_string(), "1<=x<=2");
        assert_eq!(Times::from(1..).to_string(), "1<=x");
        assert_eq!(Times::from(..2).to_string(), "x<2");
    }
}
