use proc_macro2::{Span, TokenStream};
use quote::{quote, ToTokens};
use syn::{Ident, ItemTrait};

use crate::method;

pub(crate) fn transform(input: ItemTrait) -> TokenStream {
    let async_trait_or_blank = if input.attrs.iter().any(|attr| {
        attr.path()
            .segments
            .iter()
            .any(|segment| segment.ident == "async_trait")
    }) {
        quote!(#[async_trait::async_trait])
    } else {
        TokenStream::default()
    };

    let generics = &input.generics;
    let trait_ident = &input.ident;
    let mry_ident = Ident::new(&format!("Mock{}", &input.ident), Span::call_site());
    let vis = &input.vis;
    let panic_message = format!("mock not found for {}", trait_ident);
    let (items, impl_items): (Vec<_>, Vec<_>) = input
        .items
        .iter()
        .map(|item| match item {
            syn::TraitItem::Fn(method) => {
                let method_prefix = quote![<#mry_ident as #trait_ident>::];
                let body = &method
                    .default
                    .as_ref()
                    .map(|default| default.to_token_stream())
                    .unwrap_or(quote![panic!(#panic_message)]);
                if method.sig.receiver().is_none() {
                    method::transform(
                        quote![mry::get_static_mocks()],
                        method_prefix,
                        &format!("<{} as {}>::", mry_ident, trait_ident),
                        quote![mry::static_record_call_and_find_mock_output],
                        None,
                        &method.attrs,
                        &method.sig,
                        body,
                    )
                } else {
                    method::transform(
                        quote![self.mry.mocks()],
                        method_prefix,
                        &(trait_ident.to_string() + "::"),
                        quote![self.mry.record_call_and_find_mock_output],
                        None,
                        &method.attrs,
                        &method.sig,
                        body,
                    )
                }
            }
            _item => todo!(),
        })
        .unzip();

    quote! {
        #input

        // This cfg(debug_assertions) is needed because `panic!` with return position impl
        // trait is not supported yet in rustc. It is problem with using
        // `trait_variant::make` macro that desugars `async fn`.
        // See https://github.com/rust-lang/rust/issues/35121
        #[cfg(debug_assertions)]
        #[derive(Default, Clone, Debug)]
        #vis struct #mry_ident {
            pub mry: mry::Mry,
        }
        #[cfg(debug_assertions)]
        #async_trait_or_blank
        impl #generics #trait_ident for #mry_ident {
            #(#items)*
        }

        #[cfg(debug_assertions)]
        impl #mry_ident {
            #(#impl_items)*
        }
    }
}

#[cfg(test)]
mod test {
    use pretty_assertions::assert_eq;
    use syn::{parse2, parse_quote};

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
            transform(input).to_string(),
            quote! {
                trait Cat {
                    fn meow(&self, count: usize) -> String;
                }

                #[cfg(debug_assertions)]
                #[derive(Default, Clone, Debug)]
                struct MockCat {
                    pub mry : mry::Mry,
                }

                #[cfg(debug_assertions)]
                impl Cat for MockCat {
                    fn meow(&self, count: usize) -> String {
                        #[cfg(debug_assertions)]
                        if let Some(out) = self.mry.record_call_and_find_mock_output::<_, String>(std::any::Any::type_id(&<MockCat as Cat>::meow), "Cat::meow", (<usize>::clone(&count),)) {
                            return out;
                        }
                        panic!("mock not found for Cat")
                    }
                }

                #[cfg(debug_assertions)]
                impl MockCat {
                    #[cfg(debug_assertions)]
                    #[must_use]
                    pub fn mock_meow(&mut self, count: impl Into<mry::ArgMatcher<usize>>) -> mry::MockLocator<(usize,), String, mry::Behavior1<(usize,), String> > {
                        mry::MockLocator::new(
                            self.mry.mocks(),
                            std::any::Any::type_id(&<MockCat as Cat>::meow),
                            "Cat::meow",
                            (count.into(),).into(),
                        )
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
            transform(input).to_string(),
            quote! {
                pub trait Cat {
                    fn meow(&self, count: usize) -> String;
                }

                #[cfg(debug_assertions)]
                #[derive(Default, Clone, Debug)]
                pub struct MockCat {
                    pub mry : mry::Mry,
                }

                #[cfg(debug_assertions)]
                impl Cat for MockCat {
                    fn meow(&self, count: usize) -> String {
                        #[cfg(debug_assertions)]
                        if let Some(out) = self.mry.record_call_and_find_mock_output::<_, String>(std::any::Any::type_id(&<MockCat as Cat>::meow), "Cat::meow", (<usize>::clone(&count),)) {
                            return out;
                        }
                        panic!("mock not found for Cat")
                    }
                }

                #[cfg(debug_assertions)]
                impl MockCat {
                    #[cfg(debug_assertions)]
                    #[must_use]
                    pub fn mock_meow(&mut self, count: impl Into<mry::ArgMatcher<usize>>) -> mry::MockLocator<(usize,), String, mry::Behavior1<(usize,), String> > {
                        mry::MockLocator::new(
                            self.mry.mocks(),
                            std::any::Any::type_id(&<MockCat as Cat>::meow),
                            "Cat::meow",
                            (count.into(),).into(),
                        )
                    }
                }
            }
            .to_string()
        );
    }

    #[test]
    fn supports_async_trait() {
        let input: ItemTrait = parse2(quote! {
            #[async_trait::async_trait]
            trait Cat {
                async fn meow(&self, count: usize) -> String;
            }
        })
        .unwrap();

        assert_eq!(
            transform(input).to_string(),
            quote! {
                #[async_trait::async_trait]
                trait Cat {
                    async fn meow(&self, count: usize) -> String;
                }

                #[cfg(debug_assertions)]
                #[derive(Default, Clone, Debug)]
                struct MockCat {
                    pub mry : mry::Mry,
                }

                #[cfg(debug_assertions)]
                #[async_trait::async_trait]
                impl Cat for MockCat {
                    async fn meow(&self, count: usize) -> String {
                        #[cfg(debug_assertions)]
                        if let Some(out) = self.mry.record_call_and_find_mock_output::<_, String>(std::any::Any::type_id(&<MockCat as Cat>::meow), "Cat::meow", (<usize>::clone(&count),)) {
                            return out;
                        }
                        panic!("mock not found for Cat")
                    }
                }

                #[cfg(debug_assertions)]
                impl MockCat {
                    #[cfg(debug_assertions)]
                    #[must_use]
                    pub fn mock_meow(&mut self, count: impl Into<mry::ArgMatcher<usize>>) -> mry::MockLocator<(usize,), String, mry::Behavior1<(usize,), String> > {
                        mry::MockLocator::new(
                            self.mry.mocks(),
                            std::any::Any::type_id(&<MockCat as Cat>::meow),
                            "Cat::meow",
                            (count.into(),).into(),
                        )
                    }
                }
            }
            .to_string()
        );
    }

    #[test]
    fn add_allow_non_snake_case() {
        let input: ItemTrait = parse2(quote! {
            trait Cat {
                fn _meow(&self, count: usize) -> String;
            }
        })
        .unwrap();

        assert_eq!(
            transform(input).to_string(),
            quote! {
                trait Cat {
                    fn _meow(&self, count: usize) -> String;
                }

                #[cfg(debug_assertions)]
                #[derive(Default, Clone, Debug)]
                struct MockCat {
                    pub mry : mry::Mry,
                }

                #[cfg(debug_assertions)]
                impl Cat for MockCat {
                    fn _meow(&self, count: usize) -> String {
                        #[cfg(debug_assertions)]
                        if let Some(out) = self.mry.record_call_and_find_mock_output::<_, String>(std::any::Any::type_id(&<MockCat as Cat>::_meow), "Cat::_meow", (<usize>::clone(&count),)) {
                            return out;
                        }
                        panic!("mock not found for Cat")
                    }
                }

                #[cfg(debug_assertions)]
                impl MockCat {
                    #[cfg(debug_assertions)]
                    #[allow(non_snake_case)]
                    #[must_use]
                    pub fn mock__meow(&mut self, count: impl Into<mry::ArgMatcher<usize>>) -> mry::MockLocator<(usize,), String, mry::Behavior1<(usize,), String> > {
                        mry::MockLocator::new(
                            self.mry.mocks(),
                            std::any::Any::type_id(&<MockCat as Cat>::_meow),
                            "Cat::_meow",
                            (count.into(),).into(),
                        )
                    }
                }
            }
            .to_string()
        );
    }

    #[test]
    fn async_fn_in_trait() {
        let input: ItemTrait = parse2(quote! {
            trait Cat {
                async fn meow(&self, count: usize) -> String;
            }
        })
        .unwrap();

        assert_eq!(
            transform(input).to_string(),
            quote! {
                trait Cat {
                    async fn meow(&self, count: usize) -> String;
                }

                #[cfg(debug_assertions)]
                #[derive(Default, Clone, Debug)]
                struct MockCat {
                    pub mry : mry::Mry,
                }

                #[cfg(debug_assertions)]
                impl Cat for MockCat {
                    async fn meow(&self, count: usize) -> String {
                        #[cfg(debug_assertions)]
                        if let Some(out) = self.mry.record_call_and_find_mock_output::<_, String>(std::any::Any::type_id(&<MockCat as Cat>::meow), "Cat::meow", (<usize>::clone(&count),)) {
                            return out;
                        }
                        panic!("mock not found for Cat")
                    }
                }

                #[cfg(debug_assertions)]
                impl MockCat {
                    #[cfg(debug_assertions)]
                    #[must_use]
                    pub fn mock_meow(&mut self, count: impl Into<mry::ArgMatcher<usize>>) -> mry::MockLocator<(usize,), String, mry::Behavior1<(usize,), String> > {
                        mry::MockLocator::new(
                            self.mry.mocks(),
                            std::any::Any::type_id(&<MockCat as Cat>::meow),
                            "Cat::meow",
                            (count.into(),).into(),
                        )
                    }
                }
            }
            .to_string()
        );
    }

    #[test]
    fn new_method() {
        let input: ItemTrait = parse_quote! {
            trait Cat {
                fn new(name: String) -> Self;
            }
        };

        assert_eq!(transform(input).to_string(), quote! {
            trait Cat {
                fn new(name: String) -> Self;
            }

            #[cfg(debug_assertions)]
            #[derive(Default, Clone, Debug)]
            struct MockCat {
                pub mry : mry::Mry,
            }

            #[cfg(debug_assertions)]
            impl Cat for MockCat {
                fn new(name: String) -> Self {
                    #[cfg(debug_assertions)]
                    if let Some(out) = mry::static_record_call_and_find_mock_output::<_, Self>(std::any::Any::type_id(&<MockCat as Cat>::new), "<MockCat as Cat>::new", (<String>::clone(&name),)) {
                        return out;
                    }
                    panic!("mock not found for Cat")
                }
            }

            #[cfg(debug_assertions)]
            impl MockCat {
                #[cfg(debug_assertions)]
                #[must_use]
                pub fn mock_new(name: impl Into<mry::ArgMatcher<String>>) -> mry::MockLocator<(String,), Self, mry::Behavior1<(String,), Self> > {
                    mry::MockLocator::new(
                        mry::get_static_mocks(),
                        std::any::Any::type_id(&<MockCat as Cat>::new),
                        "<MockCat as Cat>::new",
                        (name.into(),).into(),
                    )
                }
            }
        }.to_string());
    }
}
