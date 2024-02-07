use std::sync::{Arc, atomic::AtomicU64};

use parking_lot::Mutex;

use crate::{callback::{Callback, Call}, Matcher};

pub struct Logger<I> {
    matcher: Arc<Mutex<Matcher<I>>>,
    callback: Callback<I, ()>,
}

impl<I> Logger<I> {
    pub(crate) fn new(
        matcher: Arc<Mutex<Matcher<I>>>,
        callback: impl Call<I, ()> + Send + 'static,
    ) -> Self {
        Self {
            matcher,
            callback: Callback::new(callback),
        }
    }
}

impl<I> Logger<I> {
    pub(crate) fn log(&self, input: &I) {
        if self.matcher.lock().matches(input) {
            self.callback.call(input);
        }
    }
}

#[derive(Clone, Default)]
pub(crate) struct Counter(Arc<AtomicU64>);

impl Counter {
    pub(crate) fn new() -> Self {
        Default::default()
    }

    pub(crate) fn get(&self) -> u64 {
        self.0.load(std::sync::atomic::Ordering::SeqCst)
    }
}

impl<I > Call<I, ()> for Counter {
    fn call(&self, _: &I) {
        self.0.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
    }
}
