use crate::Matcher;

pub enum Behavior<I, O> {
    Function(Box<dyn for<'a> FnMut(I) -> O + Send + Sync + 'static>),
}

impl<I: Clone, O> Behavior<I, O> {
    pub fn called(&mut self, input: &I) -> O {
        match self {
            Behavior::Function(function) => function(input.clone()),
            _ => {
                todo!()
            }
        }
    }
}

impl<F, I, O> From<F> for Behavior<I, O>
where
    F: for<'a> FnMut(I) -> O + Send + Sync + 'static,
{
    fn from(function: F) -> Self {
        Behavior::Function(Box::new(function))
    }
}
