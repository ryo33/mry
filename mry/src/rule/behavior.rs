pub enum Behavior<I, O> {
    Function(Box<dyn FnMut(I) -> O + Send + Sync + 'static>),
    Const(O),
}

impl<I: Clone, O: Clone> Behavior<I, O> {
    pub fn called(&mut self, input: &I) -> O {
        match self {
            Behavior::Function(function) => function(input.clone()),
            Behavior::Const(cons) => cons.clone(),
        }
    }
}

mry_macros::create_behaviors!();
