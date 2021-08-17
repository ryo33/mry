mod method;
mod type_name;
use proc_macro2::TokenStream;
use quote::{quote, ToTokens};
use syn::visit::Visit;
use syn::{ImplItem, ItemImpl};
use type_name::*;

#[derive(Default)]
struct TypeParameterVisitor(Vec<String>);

// TODO does not work
impl<'mryst> Visit<'mryst> for TypeParameterVisitor {
    fn visit_path_segment(&mut self, path_seg: &'mryst syn::PathSegment) {
        self.visit_path_arguments(&path_seg.arguments);

        self.0.push(path_seg.ident.to_string());
    }
    fn visit_lifetime(&mut self, lifetime: &'mryst syn::Lifetime) {
        self.0.push(lifetime.ident.to_string());
    }
}

pub(crate) fn transform(input: ItemImpl) -> TokenStream {
    let generics = &input.generics;
    let mut type_params = TypeParameterVisitor::default();
    type_params.visit_type(&input.self_ty);
    let impl_generics: Vec<_> = input
        .generics
        .params
        .iter()
        .filter(|param| {
            let ident = match param {
                syn::GenericParam::Type(ty) => &ty.ident,
                syn::GenericParam::Lifetime(lifetime) => &lifetime.lifetime.ident,
                syn::GenericParam::Const(cons) => &cons.ident,
            };
            type_params.0.contains(&ident.to_string())
        })
        .collect();
    let struct_type = &input.self_ty;
    let mut trait_name = None;
    let trait_ = match &input.trait_ {
        Some((bang, path, for_)) => {
            trait_name = Some(path_name(path));
            quote! {
                #bang #path #for_
            }
        }
        None => TokenStream::default(),
    };
    let struct_name = type_name(&*input.self_ty);
    let type_name = match trait_name {
        Some(trait_name) => format!("<{} as {}>", struct_name, trait_name),
        None => struct_name,
    };
    let (members, impl_members): (Vec<_>, Vec<_>) = input
        .items
        .iter()
        .map(|item| {
            if let ImplItem::Method(method) = item {
                method::transform(&type_name, method)
            } else {
                (item.to_token_stream(), TokenStream::default())
            }
        })
        .unzip();

    let impl_generics = if impl_generics.is_empty() {
        TokenStream::default()
    } else {
        quote!( <#(#impl_generics),*>)
    };

    quote! {
        impl #generics #trait_ #struct_type {
            #(#members)*
        }

        impl #impl_generics #struct_type {
            #(#impl_members)*
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
                }

                impl Cat {
                    #[cfg(test)]
                    fn mock_meow<'mry>(&'mry mut self) -> mry::MockLocator<'mry, (usize), String, mry::Behavior1<(usize), String> > {
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
            impl<'a, A: Clone> Cat<'a, A> {
                fn meow<'a, B>(&'a self, count: usize) -> B {
                    "meow".repeat(count)
                }
            }
        })
        .unwrap();

        assert_eq!(
            transform(input).to_string(),
            quote! {
                impl<'a, A: Clone> Cat<'a, A> {
                    fn meow<'a, B>(&'a self, count: usize) -> B {
                        #[cfg(test)]
                        if self.mry.is_some() {
                            return mry::MOCK_DATA
                                .lock()
                                .get_mut_or_create::<(usize), B>(&self.mry, "Cat<'a, A>::meow")
                                ._inner_called(&(count));
                        }
                        {
                            "meow".repeat(count)
                        }
                    }
                }

                impl <'a, A: Clone> Cat<'a, A> {
                    #[cfg(test)]
                    fn mock_meow<'mry>(&'mry mut self) -> mry::MockLocator<'mry, (usize), B, mry::Behavior1<(usize), B> > {
                        if self.mry.is_none() {
                            self.mry = mry::Mry::generate();
                        }
                        mry::MockLocator {
                            id: &self.mry,
                            name: "Cat<'a, A>::meow",
                            _phantom: Default::default(),
                        }
                    }
                }
            }
            .to_string()
        );
    }

    #[test]
    fn support_trait() {
        let input: ItemImpl = parse2(quote! {
            impl<A: Clone> Animal<A> for Cat {
                fn name(&self) -> String {
                    self.name
                }
            }
        })
        .unwrap();

        assert_eq!(
            transform(input).to_string(),
            quote! {
                impl<A: Clone> Animal<A> for Cat {
                    fn name(&self, ) -> String {
                        #[cfg(test)]
                        if self.mry.is_some() {
                            return mry::MOCK_DATA
                                .lock()
                                .get_mut_or_create::<(), String>(&self.mry, "<Cat as Animal<A>>::name")
                                ._inner_called(&());
                        }
                        {
                            self.name
                        }
                    }
                }

                impl Cat {
                    #[cfg(test)]
                    fn mock_name<'mry>(&'mry mut self) -> mry::MockLocator<'mry, (), String, mry::Behavior0<(), String> > {
                        if self.mry.is_none() {
                            self.mry = mry::Mry::generate();
                        }
                        mry::MockLocator {
                            id: &self.mry,
                            name: "<Cat as Animal<A>>::name",
                            _phantom: Default::default(),
                        }
                    }
                }
            }
            .to_string()
        );
    }
}
