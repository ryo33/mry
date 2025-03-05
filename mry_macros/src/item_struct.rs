use proc_macro2::TokenStream;
use proc_macro2::TokenTree;
use quote::quote;
use quote::ToTokens;
use syn::ItemStruct;

pub(crate) fn transform(input: ItemStruct) -> TokenStream {
    let vis = &input.vis;
    let struct_name = &input.ident;

    let serde_skip_or_blank = if input.attrs.iter().any(|attr| {
        if !attr.path().is_ident("derive") {
            return false;
        }
        attr.meta
            .require_list()
            .unwrap()
            .tokens
            .to_token_stream()
            .into_iter()
            .any(|token| {
                if let TokenTree::Ident(ident) = token {
                    ident == "Serialize" || ident == "Deserialize"
                } else {
                    false
                }
            })
    }) {
        quote!(#[serde(skip)])
    } else {
        TokenStream::default()
    };

    let attrs = &input.attrs;
    let struct_fields = input
        .fields
        .iter()
        .map(|field| {
            let attrs = field.attrs.clone();
            let name = field.ident.as_ref().unwrap();
            let ty = &field.ty;
            let vis = &field.vis;
            quote! {
                #(#attrs)*
                #vis #name: #ty
            }
        })
        .collect::<Vec<_>>();
    let struct_field_names = input
        .fields
        .iter()
        .map(|field| &field.ident)
        .collect::<Vec<_>>();
    let generics = &input.generics;
    let comma_for_fields = if struct_field_names.is_empty() {
        None
    } else {
        Some(quote![,])
    };

    quote! {
        #(#attrs)*
        #vis struct #struct_name #generics {
            #(#struct_fields),*#comma_for_fields
            #serde_skip_or_blank
            pub mry: mry::Mry,
        }
    }
}

pub(crate) fn unsafe_transform(input: ItemStruct) -> TokenStream {
    let vis = &input.vis;
    let struct_name = &input.ident;

    let serde_skip_or_blank = if input.attrs.iter().any(|attr| {
        if !attr.path().is_ident("derive") {
            return false;
        }
        attr.meta
            .require_list()
            .unwrap()
            .tokens
            .to_token_stream()
            .into_iter()
            .any(|token| {
                if let TokenTree::Ident(ident) = token {
                    ident == "Serialize" || ident == "Deserialize"
                } else {
                    false
                }
            })
    }) {
        quote!(#[serde(skip)])
    } else {
        TokenStream::default()
    };

    let attrs = &input.attrs;
    let struct_fields = input
        .fields
        .iter()
        .map(|field| {
            let attrs = field.attrs.clone();
            let name = field.ident.as_ref().unwrap();
            let ty = &field.ty;
            let vis = &field.vis;
            quote! {
                #(#attrs)*
                #vis #name: #ty
            }
        })
        .collect::<Vec<_>>();
    let struct_field_names = input
        .fields
        .iter()
        .map(|field| &field.ident)
        .collect::<Vec<_>>();
    let generics = &input.generics;
    let comma_for_fields = if struct_field_names.is_empty() {
        None
    } else {
        Some(quote![,])
    };

    quote! {
        #(#attrs)*
        #vis struct #struct_name #generics {
            #(#struct_fields),*#comma_for_fields
            #serde_skip_or_blank
            pub mry: mry::unsafe_mocks::UnsafeMry,
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
        let input: ItemStruct = parse2(quote! {
            struct Cat {
                name: String,
            }
        })
        .unwrap();

        assert_eq!(
            transform(input).to_string(),
            quote! {
                struct Cat {
                    name: String,
                    pub mry : mry::Mry,
                }
            }
            .to_string()
        );
    }

    #[test]
    fn keep_attributes() {
        let input: ItemStruct = parse2(quote! {
            #[derive(Clone, Default)]
            struct Cat {
                #[name]
                name: String,
            }
        })
        .unwrap();

        assert_eq!(
            transform(input).to_string(),
            quote! {
                #[derive(Clone, Default)]
                struct Cat {
                    #[name]
                    name: String,
                    pub mry : mry::Mry,
                }
            }
            .to_string()
        );
    }

    #[test]
    fn keep_publicity() {
        let input: ItemStruct = parse2(quote! {
            pub struct Cat {
                pub name: String,
            }
        })
        .unwrap();

        assert_eq!(
            transform(input).to_string(),
            quote! {
                pub struct Cat {
                    pub name: String,
                    pub mry : mry::Mry,
                }
            }
            .to_string()
        );
    }

    #[test]
    fn support_generics() {
        let input: ItemStruct = parse2(quote! {
            pub struct Cat<'a, A> {
                pub name: &'a A,
            }
        })
        .unwrap();

        assert_eq!(
            transform(input).to_string(),
            quote! {
                pub struct Cat<'a, A> {
                    pub name: &'a A,
                    pub mry : mry::Mry,
                }
            }
            .to_string()
        );
    }

    #[test]
    fn support_blank_struct() {
        let input: ItemStruct = parse2(quote! {
            struct Cat {
            }
        })
        .unwrap();

        assert_eq!(
            transform(input).to_string(),
            quote! {
                struct Cat {
                    pub mry : mry::Mry,
                }
            }
            .to_string()
        );
    }

    #[test]
    fn skip_serde() {
        let input: ItemStruct = parse2(quote! {
            #[derive(Debug, Clone, PartialEq, Serialize)]
            struct Cat {
                pub name: String
            }
        })
        .unwrap();

        assert_eq!(
            transform(input).to_string(),
            quote! {
                #[derive(Debug, Clone, PartialEq, Serialize)]
                struct Cat {
                    pub name: String,
                    #[serde(skip)]
                    pub mry : mry::Mry,
                }
            }
            .to_string()
        );
    }

    #[test]
    fn skip_serde_with_path() {
        let input: ItemStruct = parse2(quote! {
            #[derive(Debug, Clone, PartialEq, serde::Deserialize)]
            struct Cat {
                pub name: String
            }
        })
        .unwrap();

        assert_eq!(
            transform(input).to_string(),
            quote! {
                #[derive(Debug, Clone, PartialEq, serde::Deserialize)]
                struct Cat {
                    pub name: String,
                    #[serde(skip)]
                    pub mry : mry::Mry,
                }
            }
            .to_string()
        );
    }
}
