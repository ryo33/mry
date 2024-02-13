use proc_macro2::{Span, TokenStream};
use quote::quote;
use syn::Ident;

use crate::alphabets::alphabets;

pub fn create() -> TokenStream {
    let items = alphabets(0..6).map(|args| {
        let (args, types): (Vec<_>, Vec<_>) = args
            .iter()
            .map(|name| {
                (
                    Ident::new(&name.to_lowercase(), Span::call_site()),
                    Ident::new(name, Span::call_site()),
                )
            })
            .unzip();
        let behavior_name = Ident::new(&format!("Behavior{}", args.len()), Span::call_site());
        quote! {
            #[doc(hidden)]
            pub struct #behavior_name<I, O>(Box<dyn FnMut(I) -> O + Send + 'static>);

            impl<Fn, O, #(#types),*> From<Fn> for #behavior_name<(#(#types,)*), O>
            where
                Fn: FnMut(#(#types),*) -> O + Send + 'static,
            {
                fn from(mut function: Fn) -> Self {
                    #behavior_name(Box::new(move |(#(#args,)*)| function(#(#args),*)))
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
        }
    });
    quote![#(#items)*]
}
