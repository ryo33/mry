use proc_macro2::TokenStream;
use quote::quote;
use syn::{Expr, ExprPath, ExprStruct, Member};

pub(crate) fn transform(input: Expr) -> TokenStream {
    match input {
        Expr::Struct(input) => transform_struct(input, quote!(Default::default())),
        Expr::Path(input) => transform_path(input),
        input => unexpected(input),
    }
}

pub(crate) fn transform_unique(input: Expr) -> TokenStream {
    let mry = quote!(mry::Mry::unique());
    match input {
        Expr::Struct(input) => transform_struct(input, mry),
        Expr::Path(input) => {
            let mock_default = transform_path(input);
            quote! {
                {
                    let mut __mry_unique = #mock_default;
                    __mry_unique.mry = #mry;
                    __mry_unique
                }
            }
        }
        input => unexpected(input),
    }
}

fn unexpected(input: Expr) -> TokenStream {
    syn::Error::new_spanned(
        input,
        "expected a struct expression like `Cat { name: \"Tama\".into() }` or a struct path like `Cat`",
    )
    .to_compile_error()
}

fn transform_path(input: ExprPath) -> TokenStream {
    quote! {
        <#input as mry::MockDefault>::mock_default()
    }
}

fn transform_struct(input: ExprStruct, mry: TokenStream) -> TokenStream {
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
        mry: #mry,
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
            transform_struct(input, quote!(Default::default())).to_string(),
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
            transform_struct(input, quote!(Default::default())).to_string(),
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

    #[test]
    fn unique_adds_unique_mry() {
        let input: Expr = parse2(quote! {
            Cat {
                name: "aaa",
            }
        })
        .unwrap();

        assert_eq!(
            transform_unique(input).to_string(),
            quote! {
                Cat {
                    name: "aaa",
                    mry: mry::Mry::unique(),
                }
            }
            .to_string()
        );
    }

    #[test]
    fn unique_path_replaces_mry_of_mock_default() {
        let input: Expr = parse2(quote!(Cat::<A>)).unwrap();

        assert_eq!(
            transform_unique(input).to_string(),
            quote! {
                {
                    let mut __mry_unique = <Cat::<A> as mry::MockDefault>::mock_default();
                    __mry_unique.mry = mry::Mry::unique();
                    __mry_unique
                }
            }
            .to_string()
        );
    }
}
