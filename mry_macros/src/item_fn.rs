use proc_macro2::TokenStream;
use quote::{quote, ToTokens};
use syn::ItemFn;

use crate::method;

pub(crate) fn transform(input: ItemFn) -> TokenStream {
    let (original, mock) = method::transform(
        quote![mry::STATIC_MOCKS.clone()],
        Default::default(),
        "",
        quote![mry::STATIC_MOCKS.lock().record_call_and_find_mock_output],
        Some(&input.vis),
        &input.attrs,
        &input.sig,
        &input.block.to_token_stream(),
    );

    quote! {
        #original
        #mock
    }
}

#[cfg(test)]
mod test {
    use pretty_assertions::assert_eq;
    use syn::parse2;

    use super::*;

    #[test]
    fn add_mry_object() {
        let input: ItemFn = parse2(quote! {
            fn meow(count: usize) -> String {
                "meow".repeat(count)
            }
        })
        .unwrap();

        assert_eq!(
            transform(input).to_string(),
            quote! {
                fn meow(count: usize) -> String {
                    #[cfg(debug_assertions)]
                    if let Some(out) = mry::STATIC_MOCKS.lock().record_call_and_find_mock_output::<_, String>(std::any::Any::type_id(&meow), "meow", (count.clone())) {
                        return out;
                    }
                    {
                        "meow".repeat(count)
                    }
                }

                #[cfg(debug_assertions)]
                #[must_use]
                pub fn mock_meow(count: impl Into<mry::Matcher<usize>>) -> mry::MockLocator<(usize), String, mry::Behavior1<(usize), String> > {
                    mry::MockLocator::new(
                        mry::STATIC_MOCKS.clone(),
                        std::any::Any::type_id(&meow),
                        "meow",
                        (count.into(),).into(),
                    )
                }
            }
            .to_string()
        );
    }

    #[test]
    fn add_allow_non_snake_case() {
        let input: ItemFn = parse2(quote! {
            fn _meow(count: usize) -> String {
                "meow".repeat(count)
            }
        })
        .unwrap();

        assert_eq!(
            transform(input).to_string(),
            quote! {
                fn _meow(count: usize) -> String {
                    #[cfg(debug_assertions)]
                    if let Some(out) = mry::STATIC_MOCKS.lock().record_call_and_find_mock_output::<_, String>(std::any::Any::type_id(&_meow), "_meow", (count.clone())) {
                        return out;
                    }
                    {
                        "meow".repeat(count)
                    }
                }

                #[cfg(debug_assertions)]
                #[allow(non_snake_case)]
                #[must_use]
                pub fn mock__meow(count: impl Into<mry::Matcher<usize>>) -> mry::MockLocator<(usize), String, mry::Behavior1<(usize), String> > {
                    mry::MockLocator::new(
                        mry::STATIC_MOCKS.clone(),
                        std::any::Any::type_id(&_meow),
                        "_meow",
                        (count.into(),).into(),
                    )
                }
            }
            .to_string()
        );
    }
}
