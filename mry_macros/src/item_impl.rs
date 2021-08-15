use proc_macro2::{Span, TokenStream};
use quote::quote;
use syn::{FnArg, Ident, ImplItem, ItemImpl, ReturnType};

pub(crate) fn transform(input: ItemImpl) -> TokenStream {
    let struct_type = input.self_ty;
    let methods: Vec<_> = input
        .items
        .iter()
        .filter_map(|item| {
            if let ImplItem::Method(method) = item {
                Some(method)
            } else {
                None
            }
        })
        .map(|method| {
			let attrs = method.attrs.clone();
            let ident = method.sig.ident.clone();
			let mock_ident = Ident::new(&format!("mock_{}", ident), Span::call_site());
            let name = ident.to_string();
            let inputs = method.sig.inputs.clone();
            let input_type_tuple: Vec<_> = method
                .sig
                .inputs
                .iter()
                .filter_map(|fn_arg| {
                    if let FnArg::Typed(typed_arg) = fn_arg {
                        let ty = &typed_arg.ty;
                        Some(quote!(#ty))
                    } else {
                        None
                    }
                })
                .collect();
            let input_tuple: Vec<_> = method
                .sig
                .inputs
                .iter()
                .filter_map(|fn_arg| {
                    if let FnArg::Typed(typed_arg) = fn_arg {
                        let pat = &typed_arg.pat;
                        Some(quote!(&#pat))
                    } else {
                        None
                    }
                })
                .collect();
            let output_type = match &method.sig.output {
                ReturnType::Default => quote!(()),
                ReturnType::Type(_, ty) => quote!(#ty),
            };
			let input_type_tuple = quote!((#(#input_type_tuple),*));
			let mock_type = quote!(mry::Mock<#input_type_tuple, #output_type>);
			let input_tuple = quote!((#(#input_tuple)*));
            quote! {
				#(#attrs)*
                fn #ident(#inputs) -> #output_type {
					#[cfg(test)]
					if self.mry_id.is_some() {
						return mry::MOCK_DATA
							.lock()
							.get_mut_or_create::<#input_type_tuple, #output_type>(&self.mry_id, #name)
							._inner_called(#input_tuple);
                    }
                    "meow".repeat(count)
                }

				#[cfg(test)]
				fn #mock_ident<'a>(&'a mut self) -> mry::MockLocator<'a, #input_type_tuple, #output_type> {
					if self.mry_id.is_none() {
						self.mry_id = mry::MryId::generate();
					}
					mry::MockLocator {
						id: &self.mry_id,
						name: #name,
						_phantom: Default::default(),
					}
				}
            }
        })
        .collect();

    quote! {
        impl #struct_type {
            #(#methods)*
        }
    }
}

#[cfg(test)]
mod test {
    use similar_asserts::assert_eq;
    use syn::parse2;

    use super::*;

    #[test]
    fn adds_mry_id() {
        let input: ItemImpl = parse2(quote! {
            impl Cat {
                fn meow(&self, count: usize) -> String {
                    "meow".repeat(count)
                }
            }
        })
        .unwrap();

        assert_eq!(
            transform(input).to_string(),
            quote! {
                impl Cat {
                    fn meow(&self, count: usize) -> String {
                        #[cfg(test)]
                        if self.mry_id.is_some() {
                            return mry::MOCK_DATA
                                .lock()
                                .get_mut_or_create::<(usize), String>(&self.mry_id, "meow")
                                ._inner_called((&count));
                        }
                        "meow".repeat(count)
                    }

                    #[cfg(test)]
                    fn mock_meow<'a>(&'a mut self) -> mry::MockLocator<'a, (usize), String> {
                        if self.mry_id.is_none() {
                            self.mry_id = mry::MryId::generate();
                        }
                        mry::MockLocator {
                            id: &self.mry_id,
                            name: "meow",
                            _phantom: Default::default(),
                        }
                    }
                }
            }
            .to_string()
        );
    }

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
                        if self.mry_id.is_some() {
                            return mry::MOCK_DATA
                                .lock()
                                .get_mut_or_create::<(usize), String>(&self.mry_id, "meow")
                                ._inner_called((&count));
                        }
                        "meow".repeat(count)
                    }

                    #[cfg(test)]
                    fn mock_meow<'a>(&'a mut self) -> mry::MockLocator<'a, (usize), String> {
                        if self.mry_id.is_none() {
                            self.mry_id = mry::MryId::generate();
                        }
                        mry::MockLocator {
                            id: &self.mry_id,
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
