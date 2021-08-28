use std::fmt::Debug;

#[derive(Debug)]
pub enum Matcher<I> {
    Any,
    Never,
    Eq(I),
    Composite(Box<dyn CompositeMatcher<I> + Send + Sync>),
}

pub trait CompositeMatcher<I>: Debug {
    fn matches(&self, input: &I) -> bool;
}

impl<I: PartialEq> Matcher<I> {
    pub fn matches(&self, input: &I) -> bool {
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

impl<I: PartialEq> Into<Matcher<I>> for (Matcher<I>,) {
    fn into(self) -> Matcher<I> {
        self.0
    }
}

mry_macros::create_matchers!();

#[cfg(test)]
mod tests {
    use super::*;

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
