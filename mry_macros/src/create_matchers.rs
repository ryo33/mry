use proc_macro2::{Span, TokenStream};
use quote::quote;
use syn::Ident;

use crate::create_behaviors::for_each_alphabet;

pub(crate) fn create() -> TokenStream {
    for_each_alphabet(|args| {
        if args.len() == 0 {
            return Default::default();
        }
        let (args, types): (Vec<_>, Vec<_>) = args
            .iter()
            .map(|name| {
                (
                    Ident::new(&name.to_lowercase(), Span::call_site()),
                    Ident::new(name, Span::call_site()),
                )
            })
            .unzip();
        let matchers: Vec<_> = types.iter().map(|ty| quote![Matcher<#ty>]).collect();
        let args = quote![#(#args,)*];
        let a = quote! {
            impl<#(#types),*> From<(#(#matchers,)*)> for Matcher<(#(#types),*)> {
                fn from((#args): &(#(#matchers,)*)) -> Self {
                    let matchers = vec![#args];
                    Matcher::Fn(Box::new(move |&(#args)| matchers.iter().zip(vec![#args].iter()).all(|(matcher, arg)| matcher.matches(arg))))
                }
            }
        };
        dbg!(a.to_string());
        a
    })
}
