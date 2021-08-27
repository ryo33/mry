#[derive(Debug, PartialEq)]
pub enum Matcher<I> {
    Any,
    Never,
    Eq(I),
}

impl<I: PartialEq> Matcher<I> {
    pub fn matches(&self, input: &I) -> bool {
        match self {
            Matcher::Any => true,
            Matcher::Never => false,
            Matcher::Eq(value) => value == input,
            _ => todo!(),
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
        todo!()
    }
}

impl<T: ToOwned> From<&T> for Matcher<T> {
    fn from(from: &T) -> Self {
        todo!()
    }
}

pub struct Matcher0();

impl<I> Into<Matcher<I>> for Matcher0 {
    fn into(self) -> Matcher<I> {
        todo!()
    }
}

impl<A> Into<Matcher<(A)>> for (Matcher<A>,) {
    fn into(self) -> Matcher<(A)> {
        todo!()
    }
}

impl<A, B> Into<Matcher<(A, B)>> for (Matcher<A>, Matcher<B>) {
    fn into(self) -> Matcher<(A, B)> {
        todo!()
    }
}

impl<A, B, C> Into<Matcher<(A, B, C)>> for (Matcher<A>, Matcher<B>, Matcher<C>) {
    fn into(self) -> Matcher<(A, B, C)> {
        todo!()
    }
}

pub struct Any;

impl<T> From<Any> for Matcher<T> {
    fn from(_: Any) -> Self {
        todo!()
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
        let matcher = Matcher::<u8>::Any;
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
