use proc_macro2::TokenStream;
use quote::{quote, ToTokens};
use syn::ItemFn;

use crate::method;

pub(crate) fn transform(input: ItemFn) -> TokenStream {
    let (original, mock) = method::transform(
        quote![Box::new(mry::STATIC_MOCKS.write())],
        Default::default(),
        "",
        quote![mry::STATIC_MOCKS.write().record_call_and_find_mock_output],
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
					#[cfg(test)]
					if let Some(out) = mry::STATIC_MOCKS.write().record_call_and_find_mock_output(std::any::Any::type_id(&meow), "meow", (count.clone())) {
						return out;
					}
					{
                        "meow".repeat(count)
                    }
				}

				#[cfg(test)]
				pub fn mock_meow<'mry>(arg0: impl Into<mry::Matcher<usize>>) -> mry::MockLocator<'mry, (usize), String, mry::Behavior1<(usize), String> > {
					mry::MockLocator {
						mocks: Box::new(mry::STATIC_MOCKS.write()),
						key: std::any::Any::type_id(&meow),
						name: "meow",
						matcher: Some((arg0.into(),).into()),
						_phantom: Default::default(),
					}
				}
            }
            .to_string()
        );
    }
}
