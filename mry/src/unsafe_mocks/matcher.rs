#[cfg(test)]
use std::{cell::RefCell, rc::Rc};

use super::mockable::UnsafeMockableArg;

pub trait UnsafeMatch<I> {
    fn matches(&self, input: &I) -> bool;
}

/// An enum describes what arguments are expected
pub struct UnsafeMatcher<I>(Box<dyn UnsafeMatch<I>>);

impl<I> UnsafeMatcher<I> {
    #[cfg(test)]
    pub(crate) fn wrapped(self) -> Rc<RefCell<UnsafeMatcher<I>>> {
        use std::{cell::RefCell, rc::Rc};

        Rc::new(RefCell::new(self))
    }

    pub(crate) fn matches(&self, input: &I) -> bool {
        self.0.matches(input)
    }
}

#[cfg(test)]
impl<I> UnsafeMatcher<I> {
    pub(crate) fn from_match(matcher: impl UnsafeMatch<I> + 'static) -> Self {
        Self(Box::new(matcher))
    }

    pub(crate) fn any() -> Self {
        struct Any;
        impl<I> UnsafeMatch<I> for Any {
            fn matches(&self, _: &I) -> bool {
                true
            }
        }
        Self::from_match(Any)
    }

    pub(crate) fn never() -> Self {
        struct Never;
        impl<I> UnsafeMatch<I> for Never {
            fn matches(&self, _: &I) -> bool {
                false
            }
        }
        Self::from_match(Never)
    }
}

pub enum UnsafeArgMatcher<I> {
    Fn(Box<dyn Fn(&I) -> bool + 'static>),
    Eq {
        value: I,
        partial_eq: fn(&I, &I) -> bool,
    },
    Any,
    Never,
}

impl<I> UnsafeArgMatcher<I> {
    pub(crate) fn new_eq(value: I) -> Self
    where
        I: PartialEq + UnsafeMockableArg,
    {
        UnsafeArgMatcher::Fn(Box::new(move |input| *input == value))
    }

    pub(crate) fn matches(&self, input: &I) -> bool {
        match self {
            UnsafeArgMatcher::Fn(f) => f(input),
            UnsafeArgMatcher::Eq { value, partial_eq } => partial_eq(value, input),
            UnsafeArgMatcher::Any => true,
            UnsafeArgMatcher::Never => false,
        }
    }
}

impl<I: PartialEq + UnsafeMockableArg> From<I> for UnsafeArgMatcher<I> {
    fn from(value: I) -> Self {
        UnsafeArgMatcher::new_eq(value)
    }
}

impl From<&str> for UnsafeArgMatcher<String> {
    fn from(value: &str) -> Self {
        UnsafeArgMatcher::new_eq(value.to_string())
    }
}

impl<'a, O: PartialEq + UnsafeMockableArg, I> From<&'a [I]> for UnsafeArgMatcher<Vec<O>>
where
    I: Into<UnsafeArgMatcher<O>> + Clone,
{
    fn from(value: &'a [I]) -> Self {
        let cloned: Vec<UnsafeArgMatcher<O>> = value
            .iter()
            .map(|elem| -> UnsafeArgMatcher<O> { elem.clone().into() })
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
        UnsafeArgMatcher::Fn(Box::new(check))
    }
}

impl<'a, O: PartialEq + UnsafeMockableArg, I, const N: usize> From<&'a [I; N]>
    for UnsafeArgMatcher<Vec<O>>
where
    I: Into<UnsafeArgMatcher<O>> + Clone,
{
    fn from(value: &'a [I; N]) -> Self {
        <&'a [I]>::into(&value[..])
    }
}

mry_macros::unsafe_create_matchers!();

#[cfg(test)]
mod tests {
    use super::*;

    struct EqMatcher<T>(T);

    impl<T: PartialEq> UnsafeMatch<T> for EqMatcher<T> {
        fn matches(&self, input: &T) -> bool {
            self.0 == *input
        }
    }

    impl<T: PartialEq + Send + 'static> UnsafeMatcher<T> {
        pub(crate) fn new_eq(value: T) -> Self {
            Self(Box::new(EqMatcher(value)))
        }
    }

    #[test]
    fn from_str() {
        let matcher: UnsafeArgMatcher<String> = "A".to_string().into();
        assert!(matcher.matches(&"A".to_string()));
        assert!(!matcher.matches(&"B".to_string()));
    }

    #[test]
    fn matcher_two_values() {
        let matcher: UnsafeMatcher<(u8, u16)> =
            UnsafeMatcher::from_match((3u8.into(), 2u16.into()));
        assert!(matcher.matches(&(3, 2)));
        assert!(!matcher.matches(&(3, 1)));
        assert!(!matcher.matches(&(1, 2)));
        assert!(!matcher.matches(&(1, 1)));
    }
}
