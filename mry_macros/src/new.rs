use proc_macro2::TokenStream;
use quote::quote;
use syn::{Expr, ExprPath, ExprStruct, Member};

pub(crate) fn transform(input: Expr) -> TokenStream {
    match input {
        Expr::Struct(input) => transform_struct(input),
        Expr::Path(input) => transform_path(input),
        input => syn::Error::new_spanned(
            input,
            "expected a struct expression like `Cat { name: \"Tama\".into() }` or a struct path like `Cat`",
        )
        .to_compile_error(),
    }
}

fn transform_path(input: ExprPath) -> TokenStream {
    quote! {
        <#input as mry::MockDefault>::mock_default()
    }
}

fn transform_struct(input: ExprStruct) -> TokenStream {
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
            transform_struct(input).to_string(),
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
            transform_struct(input).to_string(),
            quote! {
                Cat::<A> {
                    name: "aaa",
                    mry: Default::default(),
                }
            }
            .to_string()
        );
    }

    #[test]
    fn path_uses_mock_default() {
        let input: Expr = parse2(quote!(Cat)).unwrap();

        assert_eq!(
            transform(input).to_string(),
            quote! {
                <Cat as mry::MockDefault>::mock_default()
            }
            .to_string()
        );
    }

    #[test]
    fn path_supports_generics() {
        let input: Expr = parse2(quote!(Cat::<A>)).unwrap();

        assert_eq!(
            transform(input).to_string(),
            quote! {
                <Cat::<A> as mry::MockDefault>::mock_default()
            }
            .to_string()
        );
    }
}
