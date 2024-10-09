#[cfg(test)]
use parking_lot::Mutex;
#[cfg(test)]
use std::sync::Arc;

use crate::mockable::MockableArg;

/// An enum describes what arguments are expected
pub struct Matcher<I>(Box<dyn Match<I> + Send>);

impl<I> Matcher<I> {
    #[cfg(test)]
    pub(crate) fn wrapped(self) -> Arc<Mutex<Matcher<I>>> {
        Arc::new(Mutex::new(self))
    }

    pub(crate) fn matches(&self, input: &I) -> bool {
        self.0.matches(input)
    }
}

#[cfg(test)]
impl<I> Matcher<I> {
    pub(crate) fn from_match(matcher: impl Match<I> + Send + 'static) -> Self {
        Self(Box::new(matcher))
    }

    pub(crate) fn any() -> Self {
        struct Any;
        impl<I> Match<I> for Any {
            fn matches(&self, _: &I) -> bool {
                true
            }
        }
        Self::from_match(Any)
    }

    pub(crate) fn never() -> Self {
        struct Never;
        impl<I> Match<I> for Never {
            fn matches(&self, _: &I) -> bool {
                false
            }
        }
        Self::from_match(Never)
    }
}

pub trait Match<I> {
    fn matches(&self, input: &I) -> bool;
}

pub enum ArgMatcher<I> {
    Fn(Box<dyn Fn(&I) -> bool + Send + 'static>),
    Eq {
        value: I,
        partial_eq: fn(&I, &I) -> bool,
    },
    Any,
    Never,
}

impl<I> ArgMatcher<I> {
    pub(crate) fn new_eq(value: I) -> Self
    where
        I: PartialEq + MockableArg,
    {
        ArgMatcher::Fn(Box::new(move |input| *input == value))
    }

    pub(crate) fn matches(&self, input: &I) -> bool {
        match self {
            ArgMatcher::Fn(f) => f(input),
            ArgMatcher::Eq { value, partial_eq } => partial_eq(value, input),
            ArgMatcher::Any => true,
            ArgMatcher::Never => false,
        }
    }
}

impl<I: PartialEq + MockableArg> From<I> for ArgMatcher<I> {
    fn from(value: I) -> Self {
        ArgMatcher::new_eq(value)
    }
}

impl From<&str> for ArgMatcher<String> {
    fn from(value: &str) -> Self {
        ArgMatcher::new_eq(value.to_string())
    }
}

impl<'a, O: PartialEq + MockableArg, I> From<&'a [I]> for ArgMatcher<Vec<O>>
where
    I: Into<ArgMatcher<O>> + Clone,
{
    fn from(value: &'a [I]) -> Self {
        let cloned: Vec<ArgMatcher<O>> = value
            .iter()
            .map(|elem| -> ArgMatcher<O> { elem.clone().into() })
            .collect();
        let check = move |actual: &Vec<O>| {
            if actual.len() != cloned.len() {
                return false;
            }
            for (cloned_item, actual_item) in cloned.iter().zip(actual.iter()) {
                if !cloned_item.matches(actual_item) {
                    return false;
                }
            }
            true
        };
        ArgMatcher::Fn(Box::new(check))
    }
}

impl<'a, O: PartialEq + MockableArg, I, const N: usize> From<&'a [I; N]> for ArgMatcher<Vec<O>>
where
    I: Into<ArgMatcher<O>> + Clone,
{
    fn from(value: &'a [I; N]) -> Self {
        <&'a [I]>::into(&value[..])
    }
}

mry_macros::create_matchers!();

#[cfg(test)]
mod tests {
    use super::*;

    struct EqMatcher<T>(T);

    impl<T: PartialEq> Match<T> for EqMatcher<T> {
        fn matches(&self, input: &T) -> bool {
            self.0 == *input
        }
    }

    impl<T: PartialEq + Send + 'static> Matcher<T> {
        pub(crate) fn new_eq(value: T) -> Self {
            Self(Box::new(EqMatcher(value)))
        }
    }

    #[test]
    fn from_str() {
        let matcher: ArgMatcher<String> = "A".to_string().into();
        assert!(matcher.matches(&"A".to_string()));
        assert!(!matcher.matches(&"B".to_string()));
    }

    #[test]
    fn matcher_two_values() {
        let matcher: Matcher<(u8, u16)> = Matcher::from_match((3u8.into(), 2u16.into()));
        assert!(matcher.matches(&(3, 2)));
        assert!(!matcher.matches(&(3, 1)));
        assert!(!matcher.matches(&(1, 2)));
        assert!(!matcher.matches(&(1, 1)));
    }
}
