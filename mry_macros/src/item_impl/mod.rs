mod method;
mod type_name;
use proc_macro2::{Span, TokenStream};
use quote::{quote, ToTokens};
use syn::{FnArg, Ident, ImplItem, ItemImpl, ReturnType, Type};
use type_name::*;

pub(crate) fn transform(input: ItemImpl) -> TokenStream {
    let impl_generics = &input.generics;
    let struct_type = &input.self_ty;
    let struct_name = type_name(&*input.self_ty);
    let members: Vec<_> = input
        .items
        .iter()
        .map(|item| {
            if let ImplItem::Method(method) = item {
                method::transform(&struct_name, method)
            } else {
                item.to_token_stream()
            }
        })
        .collect();

    quote! {
        impl #impl_generics #struct_type {
            #(#members)*
        }
    }
}

#[cfg(test)]
mod test {
    use similar_asserts::assert_eq;
    use syn::parse2;

    use super::*;

    #[test]
    fn keeps_attributes() {
        let input: ItemImpl = parse2(quote! {
            impl Cat {
                #[meow]
                #[meow]
                fn meow(#[a] &self, #[b] count: usize) -> String {
                    "meow".repeat(count)
                }
            }
        })
        .unwrap();

        assert_eq!(
            transform(input).to_string(),
            quote! {
                impl Cat {
                    #[meow]
                    #[meow]
                    fn meow(#[a] &self, #[b] count: usize) -> String {
                        #[cfg(test)]
                        if self.mry.is_some() {
                            return mry::MOCK_DATA
                                .lock()
                                .get_mut_or_create::<(usize), String>(&self.mry, "Cat::meow")
                                ._inner_called(&(count));
                        }
                        {
                            "meow".repeat(count)
                        }
                    }

                    #[cfg(test)]
                    fn mock_meow<'a>(&'a mut self) -> mry::MockLocator<'a, (usize), String, mry::Behavior1<(usize), String> > {
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
    fn support_generics() {
        let input: ItemImpl = parse2(quote! {
            impl<A: Clone> Cat<A> {
                fn meow<'a, B>(&'a self, count: usize) -> B {
                    "meow".repeat(count)
                }
            }
        })
        .unwrap();

        assert_eq!(
            transform(input).to_string(),
            quote! {
                impl<A: Clone> Cat<A> {
                    fn meow<'a, B>(&'a self, count: usize) -> B {
                        #[cfg(test)]
                        if self.mry.is_some() {
                            return mry::MOCK_DATA
                                .lock()
                                .get_mut_or_create::<(usize), B>(&self.mry, "Cat<A>::meow")
                                ._inner_called(&(count));
                        }
                        {
                            "meow".repeat(count)
                        }
                    }

                    #[cfg(test)]
                    fn mock_meow<'a>(&'a mut self) -> mry::MockLocator<'a, (usize), B, mry::Behavior1<(usize), B> > {
                        if self.mry.is_none() {
                            self.mry = mry::Mry::generate();
                        }
                        mry::MockLocator {
                            id: &self.mry,
                            name: "Cat<A>::meow",
                            _phantom: Default::default(),
                        }
                    }
                }
            }
            .to_string()
        );
    }
}
