use proc_macro2::{Span, TokenStream};
use quote::quote;
use syn::Ident;

use crate::MAX_ARGUMENT_COUNT;

pub fn create() -> TokenStream {
    let items = (0..=MAX_ARGUMENT_COUNT).map(|nargs| {
        let (args, types): (Vec<_>, Vec<_>) = (1..=nargs)
            .map(|i| {
                let name = format!("Arg{i}");
                (
                    Ident::new(&name.to_lowercase(), Span::call_site()),
                    Ident::new(&name, Span::call_site()),
                )
            })
            .unzip();
        let behavior_name = Ident::new(&format!("Behavior{}", args.len()), Span::call_site());
        let behavior_name_send_wrapper = Ident::new(&format!("BehaviorSendWrapper{}", args.len()), Span::call_site());
        quote! {
            #[doc(hidden)]
            pub struct #behavior_name<I, O>(Box<dyn FnMut(I) -> O + Send + 'static>);
            #[doc(hidden)]
            pub struct #behavior_name_send_wrapper<I, O>(Box<dyn FnMut(I) -> send_wrapper::SendWrapper<O> + Send + 'static>);

            impl<Fn, O, #(#types),*> From<Fn> for #behavior_name<(#(#types,)*), O>
            where
                Fn: FnMut(#(#types),*) -> O + Send + 'static,
            {
                fn from(mut function: Fn) -> Self {
                    #behavior_name(Box::new(move |(#(#args,)*)| function(#(#args),*)))
                }
            }

            impl<Fn, O, #(#types),*> From<Fn> for #behavior_name_send_wrapper<(#(#types,)*), O>
            where
                Fn: FnMut(#(#types),*) -> O + Send + 'static,
            {
                fn from(mut function: Fn) -> Self {
                    #behavior_name_send_wrapper(Box::new(move |(#(#args,)*)| send_wrapper::SendWrapper::new(function(#(#args),*))))
                }
            }

            impl<I: Clone, O> Into<Behavior<I, O>> for #behavior_name<I, O> {
                fn into(self) -> Behavior<I, O> {
                    Behavior::Function {
                        clone: Clone::clone,
                        call: self.0,
                    }
                }
            }

            impl<I: Clone, O> Into<Behavior<I, send_wrapper::SendWrapper<O>>> for #behavior_name_send_wrapper<I, O> {
                fn into(self) -> Behavior<I, send_wrapper::SendWrapper<O>> {
                    Behavior::Function {
                        clone: Clone::clone,
                        call: self.0,
                    }
                }
            }
        }
    });
    quote![#(#items)*]
}
