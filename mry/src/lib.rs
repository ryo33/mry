mod mock;
mod mock_locator;
mod mockable;
mod mocks;
mod mry;
mod rule;
mod static_mocks;

pub use crate::mry::*;
pub use mock_locator::*;
pub use mocks::*;
pub use mry_macros::{lock, m, mry, new};
pub use rule::*;
pub use static_mocks::*;

pub use rule::ArgMatcher::Any;

pub use mockable::*;

#[cfg(feature = "send_wrapper")]
pub mod send_wrapper {
    use std::ops::{Deref, DerefMut};

    #[derive(Debug, Clone)]
    pub struct SendWrapper<T>(send_wrapper::SendWrapper<T>);

    impl<T> SendWrapper<T> {
        pub fn new(value: T) -> Self {
            Self(send_wrapper::SendWrapper::new(value))
        }

        pub fn take(self) -> T {
            self.0.take()
        }
    }

    impl<T> PartialEq for SendWrapper<T>
    where
        T: PartialEq,
    {
        fn eq(&self, other: &Self) -> bool {
            *self.0 == *other.0
        }
    }

    impl<T> Deref for SendWrapper<T> {
        type Target = T;

        fn deref(&self) -> &Self::Target {
            &self.0
        }
    }

    impl<T> DerefMut for SendWrapper<T> {
        fn deref_mut(&mut self) -> &mut Self::Target {
            &mut self.0
        }
    }
}
