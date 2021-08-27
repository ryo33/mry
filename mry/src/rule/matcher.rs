pub enum Matcher<I> {
    Any,
    Never,
    Eq(I),
    Fn(Box<dyn Fn(&I) -> bool + Send + Sync>),
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

mry_macros::create_matchers!();

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn matcher_two_values() {
        let matcher: Matcher<(u8, u16)> = (Matcher::Eq(3u8), Matcher::Eq(2u16)).into();
        assert!(matcher.matches(&(3, 2)));
        assert!(!matcher.matches(&(3, 1)));
        assert!(!matcher.matches(&(1, 2)));
        assert!(!matcher.matches(&(1, 1)));
    }
}
