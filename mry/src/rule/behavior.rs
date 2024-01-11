use std::fmt::Debug;

use parking_lot::Mutex;

#[derive(Debug, PartialEq)]
pub(crate) enum Output<O> {
    NotMatches,
    CallsRealImpl,
    /// called once already called
    ErrorCalledOnce,
    Found(O),
}

/// Behavior of mock
pub enum Behavior<I, O> {
    /// Behaves with a function
    Function(Box<dyn FnMut(I) -> O + Send + 'static>),
    /// Returns a constant value
    Const(Mutex<Box<dyn Iterator<Item = O> + Send + 'static>>),
    /// Once
    Once(Mutex<Option<O>>),
    /// Calls real implementation instead of mock
    CallsRealImpl,
}

impl<I: Debug, O: Debug> std::fmt::Debug for Behavior<I, O> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Function(_) => f.debug_tuple("Function(_)").finish(),
            Self::Const(cons) => f
                .debug_tuple("Const")
                .field(&cons.lock().next().unwrap())
                .finish(),
            Self::Once(once) => f
                .debug_tuple("Once")
                .field(&once.lock().as_ref().unwrap())
                .finish(),
            Self::CallsRealImpl => write!(f, "CallsRealImpl"),
        }
    }
}

impl<I: Clone, O> Behavior<I, O> {
    pub(crate) fn called(&mut self, input: &I) -> Output<O> {
        match self {
            Behavior::Function(function) => Output::Found(function(input.clone())),
            Behavior::Const(cons) => Output::Found(cons.get_mut().next().unwrap()),
            Behavior::Once(once) => {
                if let Some(ret) = once.lock().take() {
                    Output::Found(ret)
                } else {
                    Output::ErrorCalledOnce
                }
            }
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
            Behavior::Const(Mutex::new(Box::new(repeat("aaa")))).called(&()),
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

    #[test]
    fn debug_calls_real_impl() {
        assert_eq!(
            format!("{:?}", Behavior::<u8, u8>::CallsRealImpl),
            "CallsRealImpl".to_string()
        )
    }

    #[test]
    fn debug_const() {
        assert_eq!(
            format!(
                "{:?}",
                Behavior::<u8, u8>::Const(Mutex::new(Box::new(repeat(3))))
            ),
            "Const(3)".to_string()
        )
    }

    #[test]
    fn debug_function() {
        assert_eq!(
            format!("{:?}", Behavior::<u8, u8>::Function(Box::new(|_| 42))),
            "Function(_)".to_string()
        )
    }
}
