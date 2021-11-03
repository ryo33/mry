use proc_macro2::TokenStream;
use quote::quote;
use syn::ItemStruct;

pub(crate) fn transform(input: ItemStruct) -> TokenStream {
    let vis = &input.vis;
    let struct_name = &input.ident;
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
            #[cfg(test)]
            pub mry: mry::Mry,
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
                    #[cfg(test)]
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
                    #[cfg(test)]
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
                    #[cfg(test)]
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
                    #[cfg(test)]
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
                    #[cfg(test)]
                    pub mry : mry::Mry,
                }
            }
            .to_string()
        );
    }
}
