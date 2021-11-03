mod mock;
mod mock_key;
mod mock_locator;
mod mocks;
mod mry;
mod rule;

pub use crate::mry::*;
use mock::*;
pub use mock_locator::*;
pub use mocks::*;
pub use mry_macros::*;
use once_cell::sync::Lazy;
use parking_lot::Mutex;
pub use rule::*;
pub use Matcher::Any;

pub static STATIC_MOCKS: Lazy<Mutex<Mocks>> = Lazy::new(|| Mutex::new(Mocks::default()));
