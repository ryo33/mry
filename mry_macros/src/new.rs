use proc_macro2::TokenStream;
use quote::quote;
use syn::{ExprStruct, Member};

pub(crate) fn transform(input: ExprStruct) -> TokenStream {
    let ident = input.path.clone();
    let mut fields: Vec<_> = input
        .fields
        .iter()
        .map(|field| {
            if let Member::Named(ident) = &field.member {
                let expr = &field.expr;
                quote! {
                    #ident: #expr,
                }
            } else {
                quote!(compile_error!("mry does not support tuple structs yet."))
            }
        })
        .collect();
    fields.push(quote! {
        mry: Default::default(),
    });
    quote! {
        #ident {
            #(#fields)*
        }
    }
}

#[cfg(test)]
mod test {
    use pretty_assertions::assert_eq;
    use syn::parse2;

    use super::*;

    #[test]
    fn adds_mry() {
        let input: ExprStruct = parse2(quote! {
            Cat {
                name: "aaa",
            }
        })
        .unwrap();

        assert_eq!(
            transform(input).to_string(),
            quote! {
                Cat {
                    name: "aaa",
                    mry: Default::default(),
                }
            }
            .to_string()
        );
    }

    #[test]
    fn support_generics() {
        let input: ExprStruct = parse2(quote! {
            Cat::<A> {
                name: "aaa",
            }
        })
        .unwrap();

        assert_eq!(
            transform(input).to_string(),
            quote! {
                Cat::<A> {
                    name: "aaa",
                    mry: Default::default(),
                }
            }
            .to_string()
        );
    }
}
