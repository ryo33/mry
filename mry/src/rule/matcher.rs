use std::fmt::Debug;

#[derive(Debug)]
/// An enum shows what arguments are expected
pub enum Matcher<I> {
    /// Any value
    Any,
    /// Never matches
    Never,
    /// Equal to the value
    Eq(I),
    /// Composite matcher
    Composite(Box<dyn CompositeMatcher<I> + Send>),
}

#[doc(hidden)]
pub trait CompositeMatcher<I>: Debug {
    fn matches(&self, input: &I) -> bool;
}

impl<I: PartialEq> Matcher<I> {
    pub(crate) fn matches(&self, input: &I) -> bool {
        match self {
            Matcher::Any => true,
            Matcher::Never => false,
            Matcher::Eq(value) => value == input,
            Matcher::Composite(matcher) => matcher.matches(input),
        }
    }
}

impl<T: PartialEq> From<T> for Matcher<T> {
    fn from(from: T) -> Self {
        Self::Eq(from)
    }
}

impl From<&str> for Matcher<String> {
    fn from(from: &str) -> Self {
        Matcher::Eq(from.to_string())
    }
}

impl<I: PartialEq> From<(Matcher<I>,)> for Matcher<I> {
    fn from(val: (Matcher<I>,)) -> Self {
        val.0
    }
}

mry_macros::create_matchers!();

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use parking_lot::Mutex;

    use super::*;

    impl<I> Matcher<I> {
        pub(crate) fn wrapped(self) -> Arc<Mutex<Matcher<I>>> {
            Arc::new(Mutex::new(self))
        }
    }

    #[test]
    fn from_str() {
        assert_eq!(
            format!("{:?}", Matcher::<String>::from("A")),
            format!("{:?}", Matcher::Eq("A".to_string()))
        );
    }

    #[test]
    fn to_owned() {
        assert_eq!(
            format!("{:?}", Matcher::<String>::from("A")),
            format!("{:?}", Matcher::Eq("A".to_string()))
        );
    }

    #[test]
    fn matcher_two_values() {
        let matcher: Matcher<(u8, u16)> = (Matcher::Eq(3u8), Matcher::Eq(2u16)).into();
        assert!(matcher.matches(&(3, 2)));
        assert!(!matcher.matches(&(3, 1)));
        assert!(!matcher.matches(&(1, 2)));
        assert!(!matcher.matches(&(1, 1)));
    }
}
