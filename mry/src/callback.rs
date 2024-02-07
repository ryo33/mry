pub struct Callback<I, O>(Box<dyn Call<I, O> + Send>);

impl<I, O> Callback<I, O> {
    pub fn new(call: impl Call<I, O> + Send + 'static) -> Self {
        Self(Box::new(call))
    }

    pub fn call(&self, input: &I) -> O {
        self.0.call(input)
    }
}

pub trait Call<I, O> {
    fn call(&self, input: &I) -> O;
}
