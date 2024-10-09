use proc_macro2::TokenStream;
use quote::{quote, ToTokens};
use syn::ItemFn;

use crate::{method, Options};

pub(crate) fn transform(options: &mut Options, input: ItemFn) -> TokenStream {
    let (original, mock) = method::transform(
        options,
        quote![mry::get_static_mocks()],
        Default::default(),
        "",
        quote![mry::static_record_call_and_find_mock_output],
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
            transform(&mut Default::default(), input).to_string(),
            quote! {
                fn meow(count: usize) -> String {
                    #[cfg(debug_assertions)]
                    if let Some(out) = mry::static_record_call_and_find_mock_output::<_, String>(std::any::Any::type_id(&meow), "meow", (<usize>::clone(&count),)) {
                        return out;
                    }
                    {
                        "meow".repeat(count)
                    }
                }

                #[cfg(debug_assertions)]
                #[must_use]
                pub fn mock_meow(count: impl Into<mry::ArgMatcher<usize>>) -> mry::MockLocator<(usize,), String, mry::Behavior1<(usize,), String> > {
                    mry::MockLocator::new(
                        mry::get_static_mocks(),
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
            transform(&mut Default::default(), input).to_string(),
            quote! {
                fn _meow(count: usize) -> String {
                    #[cfg(debug_assertions)]
                    if let Some(out) = mry::static_record_call_and_find_mock_output::<_, String>(std::any::Any::type_id(&_meow), "_meow", (<usize>::clone(&count),)) {
                        return out;
                    }
                    {
                        "meow".repeat(count)
                    }
                }

                #[cfg(debug_assertions)]
                #[allow(non_snake_case)]
                #[must_use]
                pub fn mock__meow(count: impl Into<mry::ArgMatcher<usize>>) -> mry::MockLocator<(usize,), String, mry::Behavior1<(usize,), String> > {
                    mry::MockLocator::new(
                        mry::get_static_mocks(),
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
