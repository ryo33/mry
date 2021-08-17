use proc_macro2::{Span, TokenStream};
use quote::{quote, ToTokens};
use syn::{Ident, ItemTrait};

use crate::method;

#[derive(Default)]
struct TypeParameterVisitor(Vec<String>);

pub(crate) fn transform(input: &ItemTrait) -> TokenStream {
    let generics = &input.generics;
    let trait_ident = &input.ident;
    let mry_ident = Ident::new(&format!("Mry{}", &input.ident), Span::call_site());
    let vis = &input.vis;
    let panic_message = format!("mock not found for {}", trait_ident);
    let (items, impl_items): (Vec<_>, Vec<_>) = input
        .items
        .iter()
        .map(|item| match item {
            syn::TraitItem::Method(method) => method::transform(
                &trait_ident.to_string(),
                method.to_token_stream(),
                None,
                &method.attrs,
                &method.sig,
                &method
                    .default
                    .as_ref()
                    .map(|default| default.to_token_stream())
                    .unwrap_or(quote![panic!(#panic_message)]),
            ),
            _item => todo!(),
        })
        .unzip();

    quote! {
        #input

        #[cfg(test)]
        #[derive(Default)]
        #vis struct #mry_ident {
            pub mry: mry::Mry,
        }

        #[cfg(test)]
        impl #generics #trait_ident for #mry_ident {
            #(#items)*
        }

        #[cfg(test)]
        impl #mry_ident {
            #(#impl_items)*
        }
    }
}

#[cfg(test)]
mod test {
    use similar_asserts::assert_eq;
    use syn::parse2;

    use super::*;

    #[test]
    fn add_mry_object() {
        let input: ItemTrait = parse2(quote! {
            trait Cat {
                fn meow(&self, count: usize) -> String;
            }
        })
        .unwrap();

        assert_eq!(
            transform(&input).to_string(),
            quote! {
				trait Cat {
					fn meow(&self, count: usize) -> String;
				}

				#[cfg(test)]
				#[derive(Default)]
				struct MryCat {
					pub mry : mry::Mry,
				}

				#[cfg(test)]
                impl Cat for MryCat {
                    fn meow(&self, count: usize) -> String {
                        #[cfg(test)]
                        if self.mry.is_some() {
                            return mry::MOCK_DATA
                                .lock()
                                .get_mut_or_create::<(usize), String>(&self.mry, "Cat::meow")
                                ._inner_called(&(count));
                        }
                        panic!("mock not found for Cat")
                    }
                }

				#[cfg(test)]
                impl MryCat {
                    #[cfg(test)]
                    pub fn mock_meow<'mry>(&'mry mut self) -> mry::MockLocator<'mry, (usize), String, mry::Behavior1<(usize), String> > {
                        if self.mry.is_none() {
                            self.mry = mry::Mry::generate();
                        }
                        mry::MockLocator {
                            id: &self.mry,
                            name: "Cat::meow",
                            _phantom: Default::default(),
                        }
                    }
                }
            }
            .to_string()
        );
    }

    #[test]
    fn respects_attributes_and_visibility() {
        let input: ItemTrait = parse2(quote! {
            pub trait Cat {
                fn meow(&self, count: usize) -> String;
            }
        })
        .unwrap();

        assert_eq!(
            transform(&input).to_string(),
            quote! {
				pub trait Cat {
					fn meow(&self, count: usize) -> String;
				}

				#[cfg(test)]
				#[derive(Default)]
				pub struct MryCat {
					pub mry : mry::Mry,
				}

				#[cfg(test)]
                impl Cat for MryCat {
                    fn meow(&self, count: usize) -> String {
                        #[cfg(test)]
                        if self.mry.is_some() {
                            return mry::MOCK_DATA
                                .lock()
                                .get_mut_or_create::<(usize), String>(&self.mry, "Cat::meow")
                                ._inner_called(&(count));
                        }
                        panic!("mock not found for Cat")
                    }
                }

				#[cfg(test)]
                impl MryCat {
                    #[cfg(test)]
                    pub fn mock_meow<'mry>(&'mry mut self) -> mry::MockLocator<'mry, (usize), String, mry::Behavior1<(usize), String> > {
                        if self.mry.is_none() {
                            self.mry = mry::Mry::generate();
                        }
                        mry::MockLocator {
                            id: &self.mry,
                            name: "Cat::meow",
                            _phantom: Default::default(),
                        }
                    }
                }
            }
            .to_string()
        );
    }
}
