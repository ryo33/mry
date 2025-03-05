use std::{cell::RefCell, fmt::Debug};

use crate::Output;

/// Behavior of mock
pub enum UnsafeBehavior<I, O> {
    /// Behaves with a function
    Function {
        clone: fn(&I) -> I,
        call: Box<dyn FnMut(I) -> O + 'static>,
    },
    /// Returns a constant value
    Const(RefCell<Box<dyn Iterator<Item = O> + 'static>>),
    /// Once
    Once(RefCell<Option<O>>),
    /// Calls real implementation instead of mock
    CallsRealImpl,
}

impl<I: Debug, O: Debug> std::fmt::Debug for UnsafeBehavior<I, O> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Function { .. } => f.debug_tuple("Function(_)").finish(),
            Self::Const(cons) => f
                .debug_tuple("Const")
                .field(&cons.borrow_mut().next().unwrap())
                .finish(),
            Self::Once(once) => f
                .debug_tuple("Once")
                .field(&once.borrow().as_ref().unwrap())
                .finish(),
            Self::CallsRealImpl => write!(f, "CallsRealImpl"),
        }
    }
}

impl<I, O> UnsafeBehavior<I, O> {
    pub(crate) fn called(&mut self, input: &I) -> Output<O> {
        match self {
            UnsafeBehavior::Function { clone, call } => Output::Found(call(clone(input))),
            UnsafeBehavior::Const(cons) => Output::Found(cons.get_mut().next().unwrap()),
            UnsafeBehavior::Once(once) => {
                if let Some(ret) = once.take() {
                    Output::Found(ret)
                } else {
                    Output::ErrorCalledOnce
                }
            }
            UnsafeBehavior::CallsRealImpl => Output::CallsRealImpl,
        }
    }
}

mry_macros::unsafe_create_behaviors!();

#[cfg(test)]
mod tests {
    use std::iter::repeat;

    use super::*;

    #[test]
    fn function() {
        assert_eq!(
            UnsafeBehavior::Function {
                call: Box::new(|()| "aaa"),
                clone: Clone::clone
            }
            .called(&()),
            Output::Found("aaa")
        );
    }

    #[test]
    fn const_value() {
        assert_eq!(
            UnsafeBehavior::Const(RefCell::new(Box::new(repeat("aaa")))).called(&()),
            Output::Found("aaa")
        );
    }

    #[test]
    fn calls_real_impl() {
        assert_eq!(
            UnsafeBehavior::<_, ()>::CallsRealImpl.called(&()),
            Output::CallsRealImpl
        );
    }

    #[test]
    fn debug_calls_real_impl() {
        assert_eq!(
            format!("{:?}", UnsafeBehavior::<u8, u8>::CallsRealImpl),
            "CallsRealImpl".to_string()
        )
    }

    #[test]
    fn debug_const() {
        assert_eq!(
            format!(
                "{:?}",
                UnsafeBehavior::<u8, u8>::Const(RefCell::new(Box::new(repeat(3))))
            ),
            "Const(3)".to_string()
        )
    }

    #[test]
    fn debug_function() {
        assert_eq!(
            format!(
                "{:?}",
                UnsafeBehavior::<u8, u8>::Function {
                    clone: Clone::clone,
                    call: Box::new(|a| a)
                }
            ),
            "Function(_)".to_string()
        )
    }
}
