use proc_macro2::{Span, TokenStream};
use quote::quote;
use syn::{Ident, ItemStruct};

pub(crate) fn transform(input: &ItemStruct) -> TokenStream {
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
    let mry_struct_name = Ident::new(&format!("Mry{}", struct_name), Span::call_site());
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

        #(#attrs)*
        #vis struct #mry_struct_name #generics {
            #(#struct_fields),*#comma_for_fields
        }
        impl #generics #mry_struct_name #generics {
            pub fn mry(self) -> #struct_name #generics {
                self.into()
            }
        }

        impl #generics From<#mry_struct_name #generics> for #struct_name #generics {
            fn from(#mry_struct_name {#(#struct_field_names),*}: #mry_struct_name #generics) -> Self {
                #struct_name {
                    #(#struct_field_names),*#comma_for_fields
                    #[cfg(test)]
                    mry: Default::default(),
                }
            }
        }
    }
}

#[cfg(test)]
mod test {
    use similar_asserts::assert_eq;
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
            transform(&input).to_string(),
            quote! {
                struct Cat {
                    name: String,
                    #[cfg(test)]
                    pub mry : mry::Mry,
                }

                struct MryCat {
                    name: String,
                }

                impl MryCat {
                    pub fn mry(self) -> Cat {
                        self.into()
                    }
                }

                impl From<MryCat> for Cat {
                    fn from (MryCat { name }: MryCat) -> Self {
                        Cat {
                            name,
                            #[cfg(test)] mry: Default::default(),
                        }
                    }
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
            transform(&input).to_string(),
            quote! {
                #[derive(Clone, Default)]
                struct Cat {
                    #[name]
                    name: String,
                    #[cfg(test)]
                    pub mry : mry::Mry,
                }

                #[derive(Clone, Default)]
                struct MryCat {
                    #[name]
                    name: String,
                }

                impl MryCat {
                    pub fn mry(self) -> Cat {
                        self.into()
                    }
                }

                impl From<MryCat> for Cat {
                    fn from (MryCat { name }: MryCat) -> Self {
                        Cat {
                            name,
                            #[cfg(test)] mry: Default::default(),
                        }
                    }
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
            transform(&input).to_string(),
            quote! {
                pub struct Cat {
                    pub name: String,
                    #[cfg(test)]
                    pub mry : mry::Mry,
                }

                pub struct MryCat {
                    pub name: String,
                }

                impl MryCat {
                    pub fn mry(self) -> Cat {
                        self.into()
                    }
                }

                impl From<MryCat> for Cat {
                    fn from (MryCat { name }: MryCat) -> Self {
                        Cat {
                            name,
                            #[cfg(test)] mry: Default::default(),
                        }
                    }
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
            transform(&input).to_string(),
            quote! {
                pub struct Cat<'a, A> {
                    pub name: &'a A,
                    #[cfg(test)]
                    pub mry : mry::Mry,
                }

                pub struct MryCat<'a, A> {
                    pub name: &'a A,
                }

                impl<'a, A> MryCat<'a, A> {
                    pub fn mry(self) -> Cat<'a, A> {
                        self.into()
                    }
                }

                impl<'a, A> From<MryCat<'a, A> > for Cat<'a, A> {
                    fn from (MryCat { name }: MryCat<'a, A>) -> Self {
                        Cat {
                            name,
                            #[cfg(test)] mry: Default::default(),
                        }
                    }
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
            transform(&input).to_string(),
            quote! {
                struct Cat {
                    #[cfg(test)]
                    pub mry : mry::Mry,
                }

                struct MryCat {
                }

                impl MryCat {
                    pub fn mry(self) -> Cat {
                        self.into()
                    }
                }

                impl From<MryCat> for Cat {
                    fn from (MryCat { }: MryCat) -> Self {
                        Cat {
                            #[cfg(test)] mry: Default::default(),
                        }
                    }
                }
            }
            .to_string()
        );
    }
}
