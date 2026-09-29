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
    let mock_default = mock_default(&input);

    quote! {
        #(#attrs)*
        #vis struct #struct_name #generics {
            #(#struct_fields),*#comma_for_fields
            #serde_skip_or_blank
            pub mry: mry::Mry,
        }

        #mock_default
    }
}

fn mock_default(input: &ItemStruct) -> TokenStream {
    let struct_name = &input.ident;
    let (impl_generics, ty_generics, where_clause) = input.generics.split_for_impl();
    let predicates = where_clause
        .map(|where_clause| where_clause.predicates.iter().collect::<Vec<_>>())
        .unwrap_or_default();
    let field_names = input.fields.iter().map(|field| &field.ident);
    let field_types = input.fields.iter().map(|field| &field.ty);

    // The otherwise-unused HRTB intentionally prevents rustc from eagerly rejecting
    // a concrete unsatisfied trivial bound like `NotDefault: Default`.
    // This lets #[mry::mry] be applied to any struct while making `MockDefault`
    // available only when all fields implement `Default`.
    // See rust-lang/rust#48214 (`trivial_bounds`).
    quote! {
        impl #impl_generics mry::MockDefault for #struct_name #ty_generics
        where
            #(#predicates,)*
            #(for<'__mry> #field_types: Default,)*
        {
            fn mock_default() -> Self {
                Self {
                    #(#field_names: Default::default(),)*
                    mry: Default::default(),
                }
            }
        }
    }
}

#[cfg(test)]
mod test {
    use pretty_assertions::assert_eq;
    use syn::parse2;

    use super::*;

    fn assert_struct(input: ItemStruct, expected: TokenStream) {
        let mock_default = mock_default(&input);
        assert_eq!(
            transform(input).to_string(),
            quote! {
                #expected
                #mock_default
            }
            .to_string()
        );
    }

    #[test]
    fn mock_default_requires_default_for_each_field() {
        let input: ItemStruct = parse2(quote! {
            struct Cat {
                name: String,
                age: u8,
            }
        })
        .unwrap();

        assert_eq!(
            mock_default(&input).to_string(),
            quote! {
                impl mry::MockDefault for Cat
                where
                    for<'__mry> String: Default,
                    for<'__mry> u8: Default,
                {
                    fn mock_default() -> Self {
                        Self {
                            name: Default::default(),
                            age: Default::default(),
                            mry: Default::default(),
                        }
                    }
                }
            }
            .to_string()
        );
    }

    #[test]
    fn mock_default_keeps_generics_and_where_clause() {
        let input: ItemStruct = parse2(quote! {
            struct Cat<'a, A: Clone>
            where
                A: Send,
            {
                name: &'a A,
            }
        })
        .unwrap();

        assert_eq!(
            mock_default(&input).to_string(),
            quote! {
                impl<'a, A: Clone> mry::MockDefault for Cat<'a, A>
                where
                    A: Send,
                    for<'__mry> &'a A: Default,
                {
                    fn mock_default() -> Self {
                        Self {
                            name: Default::default(),
                            mry: Default::default(),
                        }
                    }
                }
            }
            .to_string()
        );
    }

    #[test]
    fn adds_mry() {
        let input: ItemStruct = parse2(quote! {
            struct Cat {
                name: String,
            }
        })
        .unwrap();

        assert_struct(
            input,
            quote! {
                struct Cat {
                    name: String,
                    pub mry : mry::Mry,
                }
            },
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

        assert_struct(
            input,
            quote! {
                #[derive(Clone, Default)]
                struct Cat {
                    #[name]
                    name: String,
                    pub mry : mry::Mry,
                }
            },
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

        assert_struct(
            input,
            quote! {
                pub struct Cat {
                    pub name: String,
                    pub mry : mry::Mry,
                }
            },
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

        assert_struct(
            input,
            quote! {
                pub struct Cat<'a, A> {
                    pub name: &'a A,
                    pub mry : mry::Mry,
                }
            },
        );
    }

    #[test]
    fn support_blank_struct() {
        let input: ItemStruct = parse2(quote! {
            struct Cat {
            }
        })
        .unwrap();

        assert_struct(
            input,
            quote! {
                struct Cat {
                    pub mry : mry::Mry,
                }
            },
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

        assert_struct(
            input,
            quote! {
                #[derive(Debug, Clone, PartialEq, Serialize)]
                struct Cat {
                    pub name: String,
                    #[serde(skip)]
                    pub mry : mry::Mry,
                }
            },
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

        assert_struct(
            input,
            quote! {
                #[derive(Debug, Clone, PartialEq, serde::Deserialize)]
                struct Cat {
                    pub name: String,
                    #[serde(skip)]
                    pub mry : mry::Mry,
                }
            },
        );
    }
}
