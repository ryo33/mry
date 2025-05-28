use proc_macro2::{Span, TokenStream};
use quote::{quote, ToTokens};
use syn::{parse_quote, Ident, ItemTrait};

use crate::{attrs::MryAttr, method};

pub(crate) fn transform(mry_attr: &MryAttr, mut input: ItemTrait) -> TokenStream {
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
    let panic_message = format!("mock not found for {trait_ident}");
    let (def, generated): (Vec<_>, Vec<_>) = input
        .items
        .iter()
        .map(|item| match item {
            syn::TraitItem::Fn(method) => {
                if mry_attr.should_skip_method(&method.sig.ident) {
                    let mut method = method.clone();
                    method.default = Some(parse_quote!({
                        panic!("this method is skipped with `#[mry::mry(skip_fns(...))]` attribute")
                    }));
                    method.attrs.push(parse_quote!(#[allow(unused_variables)]));
                    return (
                        item.clone(),
                        (
                            syn::TraitItem::Fn(method).to_token_stream(),
                            Default::default(),
                        ),
                    );
                }
                let method_prefix = quote![<#mry_ident as #trait_ident>::];
                let body = &method
                    .default
                    .as_ref()
                    .map(|default| {
                        default
                            .stmts
                            .iter()
                            .fold(TokenStream::new(), |mut tokens, stmt| {
                                stmt.to_tokens(&mut tokens);
                                tokens
                            })
                    })
                    .unwrap_or(quote![panic!(#panic_message)]);
                if method.sig.receiver().is_none() {
                    (
                        item.clone(),
                        method::transform(
                            mry_attr,
                            quote![mry::get_static_mocks()],
                            method_prefix,
                            &format!("<{mry_ident} as {trait_ident}>::"),
                            quote![mry::static_record_call_and_find_mock_output],
                            None,
                            &method.attrs,
                            &method.sig,
                            body,
                            method.default.is_none(),
                        ),
                    )
                } else {
                    (
                        item.clone(),
                        method::transform(
                            mry_attr,
                            quote![self.mry.mocks()],
                            method_prefix,
                            &(trait_ident.to_string() + "::"),
                            quote![self.mry.record_call_and_find_mock_output],
                            None,
                            &method.attrs,
                            &method.sig,
                            body,
                            method.default.is_none(),
                        ),
                    )
                }
            }
            item => (item.clone(), Default::default()),
        })
        .unzip();
    input.items = def;
    let items = generated.iter().map(|item| &item.0);
    let impl_items = generated.iter().map(|item| &item.1);

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
    use darling::FromMeta as _;
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
            transform(&MryAttr::default(), input).to_string(),
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
                    #[cfg_attr(debug_assertions, track_caller)]
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
                    pub fn mock_meow(&mut self, count: impl Into<mry::ArgMatcher<usize>>) -> mry::MockLocator<(usize,), String, String, mry::Behavior1<(usize,), String> > {
                        mry::MockLocator::new(
                            self.mry.mocks(),
                            std::any::Any::type_id(&<MockCat as Cat>::meow),
                            "Cat::meow",
                            (count.into(),).into(),
                            std::convert::identity,
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
            transform(&MryAttr::default(), input).to_string(),
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
                    #[cfg_attr(debug_assertions, track_caller)]
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
                    pub fn mock_meow(&mut self, count: impl Into<mry::ArgMatcher<usize>>) -> mry::MockLocator<(usize,), String, String, mry::Behavior1<(usize,), String> > {
                        mry::MockLocator::new(
                            self.mry.mocks(),
                            std::any::Any::type_id(&<MockCat as Cat>::meow),
                            "Cat::meow",
                            (count.into(),).into(),
                            std::convert::identity,
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
            transform(&MryAttr::default(), input).to_string(),
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
                    #[cfg_attr(debug_assertions, track_caller)]
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
                    pub fn mock_meow(&mut self, count: impl Into<mry::ArgMatcher<usize>>) -> mry::MockLocator<(usize,), String, String, mry::Behavior1<(usize,), String> > {
                        mry::MockLocator::new(
                            self.mry.mocks(),
                            std::any::Any::type_id(&<MockCat as Cat>::meow),
                            "Cat::meow",
                            (count.into(),).into(),
                            std::convert::identity,
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
            transform(&MryAttr::default(), input).to_string(),
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
                    #[cfg_attr(debug_assertions, track_caller)]
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
                    pub fn mock__meow(&mut self, count: impl Into<mry::ArgMatcher<usize>>) -> mry::MockLocator<(usize,), String, String, mry::Behavior1<(usize,), String> > {
                        mry::MockLocator::new(
                            self.mry.mocks(),
                            std::any::Any::type_id(&<MockCat as Cat>::_meow),
                            "Cat::_meow",
                            (count.into(),).into(),
                            std::convert::identity,
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
            transform(&MryAttr::default(), input).to_string(),
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
                    #[cfg_attr(debug_assertions, track_caller)]
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
                    pub fn mock_meow(&mut self, count: impl Into<mry::ArgMatcher<usize>>) -> mry::MockLocator<(usize,), String, String, mry::Behavior1<(usize,), String> > {
                        mry::MockLocator::new(
                            self.mry.mocks(),
                            std::any::Any::type_id(&<MockCat as Cat>::meow),
                            "Cat::meow",
                            (count.into(),).into(),
                            std::convert::identity,
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

        assert_eq!(transform(&MryAttr::default(), input).to_string(), quote! {
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
                #[cfg_attr(debug_assertions, track_caller)]
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
                pub fn mock_new(name: impl Into<mry::ArgMatcher<String>>) -> mry::MockLocator<(String,), Self, Self, mry::Behavior1<(String,), Self> > {
                    mry::MockLocator::new(
                        mry::get_static_mocks(),
                        std::any::Any::type_id(&<MockCat as Cat>::new),
                        "<MockCat as Cat>::new",
                        (name.into(),).into(),
                        std::convert::identity,
                    )
                }
            }
        }.to_string());
    }

    #[test]
    fn test_skip_in_trait() {
        let attr = MryAttr::from_meta(&parse_quote! {
            mry(skip_fns(skipped))
        })
        .unwrap();

        let input: ItemTrait = parse_quote! {
            trait Cat {
                fn not_skipped(&self) -> String;
                fn skipped(&self, rc: Rc<String>) -> String;
            }
        };

        assert_eq!(
            transform(&attr, input).to_string(),
            quote! {
                trait Cat {
                    fn not_skipped(&self) -> String;
                    fn skipped(&self, rc: Rc<String>) -> String;
                }

                #[cfg(debug_assertions)]
                #[derive(Default, Clone, Debug)]
                struct MockCat {
                    pub mry : mry::Mry,
                }

                #[cfg(debug_assertions)]
                impl Cat for MockCat {
                    #[cfg_attr(debug_assertions, track_caller)]
                    fn not_skipped(&self) -> String {
                        #[cfg(debug_assertions)]
                        if let Some(out) = self.mry.record_call_and_find_mock_output::<_, String>(std::any::Any::type_id(&<MockCat as Cat>::not_skipped), "Cat::not_skipped", ()) {
                            return out;
                        }
                        panic!("mock not found for Cat")
                    }
                    #[allow(unused_variables)]
                    fn skipped(&self, rc: Rc<String>) -> String {
                        panic!("this method is skipped with `#[mry::mry(skip_fns(...))]` attribute")
                    }
                }

                #[cfg(debug_assertions)]
                impl MockCat {
                    #[cfg(debug_assertions)]
                    #[must_use]
                    pub fn mock_not_skipped(&mut self,) -> mry::MockLocator<(), String, String, mry::Behavior0<(), String> > {
                        mry::MockLocator::new(
                            self.mry.mocks(),
                            std::any::Any::type_id(&<MockCat as Cat>::not_skipped),
                            "Cat::not_skipped",
                            ().into(),
                            std::convert::identity,
                        )
                    }
                }
            }
            .to_string()
        );
    }

    #[test]
    fn trait_with_default_implementation() {
        let input: ItemTrait = parse2(quote! {
            trait Cat {
                fn meow(&self, count: usize) -> String {
                    "default meow".to_string()
                }
            }
        })
        .unwrap();

        assert_eq!(
            transform(&MryAttr::default(), input).to_string(),
            quote! {
                trait Cat {
                    fn meow(&self, count: usize) -> String {
                        "default meow".to_string()
                    }
                }

                #[cfg(debug_assertions)]
                #[derive(Default, Clone, Debug)]
                struct MockCat {
                    pub mry : mry::Mry,
                }

                #[cfg(debug_assertions)]
                impl Cat for MockCat {
                    #[cfg_attr(debug_assertions, track_caller)]
                    fn meow(&self, count: usize) -> String {
                        #[cfg(debug_assertions)]
                        if let Some(out) = self.mry.record_call_and_find_mock_output::<_, String>(std::any::Any::type_id(&<MockCat as Cat>::meow), "Cat::meow", (<usize>::clone(&count),)) {
                            return out;
                        }
                        (move || {
                            "default meow".to_string()
                        })()
                    }
                }

                #[cfg(debug_assertions)]
                impl MockCat {
                    #[cfg(debug_assertions)]
                    #[must_use]
                    pub fn mock_meow(&mut self, count: impl Into<mry::ArgMatcher<usize>>) -> mry::MockLocator<(usize,), String, String, mry::Behavior1<(usize,), String> > {
                        mry::MockLocator::new(
                            self.mry.mocks(),
                            std::any::Any::type_id(&<MockCat as Cat>::meow),
                            "Cat::meow",
                            (count.into(),).into(),
                            std::convert::identity,
                        )
                    }
                }
            }
            .to_string()
        );
    }
}
