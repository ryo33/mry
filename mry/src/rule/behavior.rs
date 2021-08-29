#[derive(Debug, PartialEq)]
pub enum Output<O> {
    NotMatches,
    CallsRealImpl,
    Found(O),
}

pub enum Behavior<I, O> {
    Function(Box<dyn FnMut(I) -> O + Send + Sync + 'static>),
    Const(Box<dyn Iterator<Item = O> + Send + Sync + 'static>),
    CallsRealImpl,
}

impl<I: Clone, O> Behavior<I, O> {
    pub fn called(&mut self, input: &I) -> Output<O> {
        match self {
            Behavior::Function(function) => Output::Found(function(input.clone())),
            Behavior::Const(cons) => Output::Found(cons.next().unwrap()),
            Behavior::CallsRealImpl => Output::CallsRealImpl,
        }
    }
}

mry_macros::create_behaviors!();

#[cfg(test)]
mod tests {
    use std::iter::repeat;

    use super::*;

    #[test]
    fn function() {
        assert_eq!(
            Behavior::Function(Box::new(|()| "aaa")).called(&()),
            Output::Found("aaa")
        );
    }

    #[test]
    fn const_value() {
        assert_eq!(
            Behavior::Const(Box::new(repeat("aaa"))).called(&()),
            Output::Found("aaa")
        );
    }

    #[test]
    fn calls_real_impl() {
        assert_eq!(
            Behavior::<_, ()>::CallsRealImpl.called(&()),
            Output::CallsRealImpl
        );
    }
}
