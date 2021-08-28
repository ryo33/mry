use proc_macro2::{Span, TokenStream};
use quote::quote;
use syn::{Ident, Index};

use crate::alphabets::alphabets;

pub(crate) fn create() -> TokenStream {
    let items = alphabets(2..6).map(|args| {
        let (args, types): (Vec<_>, Vec<_>) = args
            .iter()
            .map(|name| {
                (
                    Ident::new(&name.to_lowercase(), Span::call_site()),
                    Ident::new(name, Span::call_site()),
                )
            })
            .unzip();
        let matcher_name = Ident::new(&format!("Matcher{}", args.len()), Span::call_site());
        let matchers: Vec<_> = types.iter().map(|ty| quote![Matcher<#ty>]).collect();
        let trait_bounds: Vec<_> = types
            .iter()
            .map(|ty| quote![#ty: PartialEq + Debug + Send + Sync + 'static])
            .collect();
        let types = quote![#(#types),*];
        let matchers = quote![#(#matchers,)*];
        let matches = args.iter().enumerate().map(|(index, arg)| {
            let index = Index::from(index);
            quote![self.#index.matches(#arg)]
        });
        let args = quote![#(#args),*];
        let a = quote! {
            #[derive(Debug)]
            struct #matcher_name<#(#trait_bounds),*>(#matchers);

            impl<#(#trait_bounds),*> CompositeMatcher<(#types)> for #matcher_name<#types> {
                fn matches(&self, (#args): &(#types)) -> bool {
                    #(#matches)&&*
                }
            }

            impl<#(#trait_bounds),*> From<(#matchers)> for Matcher<(#types)> {
                fn from((#args): (#matchers)) -> Self {
                    Matcher::Composite(Box::new(#matcher_name(#args)))
                }
            }
        };
        println!("{}", a.to_string());
        a
    });
    quote![#(#items)*]
}
