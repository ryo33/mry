mod method;
use proc_macro2::{Span, TokenStream};
use quote::{quote, ToTokens};
use syn::{FnArg, Ident, ImplItem, ItemImpl, ReturnType, Type};

pub(crate) fn transform(input: ItemImpl) -> TokenStream {
    let struct_type = input.self_ty;
    let members: Vec<_> = input
        .items
        .iter()
        .map(|item| {
            if let ImplItem::Method(method) = item {
                method::transform(method)
            } else {
                item.to_token_stream()
            }
        })
        .collect();

    quote! {
        impl #struct_type {
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
                                .get_mut_or_create::<(usize), String>(&self.mry, "meow")
                                ._inner_called(&(count));
                        }
                        {
                            "meow".repeat(count)
                        }
                    }

                    #[cfg(test)]
                    fn mock_meow<'a>(&'a mut self) -> mry::MockLocator<'a, (usize), String> {
                        if self.mry.is_none() {
                            self.mry = mry::Mry::generate();
                        }
                        mry::MockLocator {
                            id: &self.mry,
                            name: "meow",
                            _phantom: Default::default(),
                        }
                    }
                }
            }
            .to_string()
        );
    }
}
