#[derive(Debug, PartialEq)]
pub enum Matcher<I> {
    Always,
    Never,
    Eq(I),
}

impl<I: PartialEq> Matcher<I> {
    pub fn matches(&self, input: &I) -> bool {
        match self {
            Matcher::Always => true,
            Matcher::Never => false,
            Matcher::Eq(value) => value == input,
        }
    }
}

impl<T: PartialEq> From<T> for Matcher<T> {
    fn from(from: T) -> Self {
        Self::Eq(from)
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn into_matcher() {
        let matcher: Matcher<u8> = 1.into();
        assert_eq!(matcher, Matcher::Eq(1));
    }

    #[test]
    fn always_returns_true() {
        let matcher = Matcher::<u8>::Always;
        assert!(matcher.matches(&3));
    }

    #[test]
    fn never_returns_false() {
        let matcher = Matcher::<u8>::Never;
        assert!(!matcher.matches(&3));
    }

    #[test]
    fn eq_returns_false() {
        let matcher = Matcher::<u8>::Eq(2);
        assert!(!matcher.matches(&3));
    }

    #[test]
    fn eq_returns_true() {
        let matcher = Matcher::<u8>::Eq(3);
        assert!(matcher.matches(&3));
    }
}
