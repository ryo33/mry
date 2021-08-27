use proc_macro2::{Span, TokenStream};
use quote::quote;
use syn::Ident;

pub fn for_each_alphabet(function: impl Fn(Vec<&str>) -> TokenStream) -> TokenStream {
    let alphabet = vec!["A", "B", "C", "D", "E", "F"];
    let item = (0..=alphabet.len())
        .into_iter()
        .map(|index| function(alphabet[0..index].iter().cloned().collect()));
    quote![#(#item)*]
}

pub fn create() -> TokenStream {
    for_each_alphabet(|args| {
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
            pub struct #behavior_name<I, O>(Box<dyn FnMut(I) -> O + Send + Sync + 'static>);

            impl<Fn, O, #(#types),*> From<Fn> for #behavior_name<(#(#types),*), O>
            where
                Fn: FnMut(#(#types),*) -> O + Send + Sync + 'static,
            {
                fn from(mut function: Fn) -> Self {
                    #behavior_name(Box::new(move |(#(#args),*)| function(#(#args),*)))
                }
            }

            impl<I, O> Into<Behavior<I, O>> for #behavior_name<I, O> {
                fn into(self) -> Behavior<I, O> {
                    Behavior::Function(self.0)
                }
            }
        }
    })
}
