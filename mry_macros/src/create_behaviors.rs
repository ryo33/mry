use proc_macro2::{Span, TokenStream};
use quote::quote;
use syn::Ident;

pub fn create() -> TokenStream {
    let alphabet = vec!["A", "B", "C", "D", "E", "F"];
    let items = (0..=alphabet.len()).into_iter().map(|index| {
        let type_names: Vec<_> = alphabet[0..index].iter().cloned().collect();
        let types: Vec<_> = type_names
            .iter()
            .map(|name| Ident::new(name, Span::call_site()))
            .collect();
        let args: Vec<_> = type_names
            .iter()
            .map(|name| Ident::new(&name.to_lowercase(), Span::call_site()))
            .collect();
        let behavior_name = Ident::new(&format!("Behavior{}", index), Span::call_site());
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
    });
    quote! {
        #(#items)*
    }
}
