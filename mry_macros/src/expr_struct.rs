use proc_macro2::TokenStream;
use quote::quote;
use syn::{ExprStruct, Member};

pub(crate) fn transform(input: ExprStruct) -> TokenStream {
    let attrs = input.attrs.clone();
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
        #[cfg(test)]
        mry_id: Default::default(),
    });
    quote! {
        #ident {
            #(#fields)*
        }
    }
}

#[cfg(test)]
mod test {
    use similar_asserts::assert_eq;
    use syn::parse2;

    use super::*;

    #[test]
    fn adds_mry_id() {
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
                    #[cfg(test)]
                    mry_id: Default::default(),
                }
            }
            .to_string()
        );
    }
}
