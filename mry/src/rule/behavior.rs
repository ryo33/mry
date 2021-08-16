pub enum Behavior<I, O> {
    Function(Box<dyn FnMut(I) -> O + Send + Sync + 'static>),
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

mry_macros::create_behaviors!();
