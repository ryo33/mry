use crate::{method, type_name::*};
use proc_macro2::TokenStream;
use quote::{quote, ToTokens};
use syn::visit::Visit;
use syn::visit_mut::VisitMut;
use syn::{parse2, Ident, ImplItem, ItemImpl, Path};

#[derive(Default)]
struct TypeParameterVisitor(Vec<String>);

impl<'ast> Visit<'ast> for TypeParameterVisitor {
    fn visit_path_segment(&mut self, path_seg: &'ast syn::PathSegment) {
        self.visit_path_arguments(&path_seg.arguments);

        self.0.push(path_seg.ident.to_string());
    }
    fn visit_lifetime(&mut self, lifetime: &'ast syn::Lifetime) {
        self.0.push(lifetime.ident.to_string());
    }
}

struct QualifiesAssociatedTypes(Path, Vec<Ident>);
impl VisitMut for QualifiesAssociatedTypes {
    fn visit_type_path_mut(&mut self, type_path: &mut syn::TypePath) {
        type_path
            .path
            .segments
            .iter_mut()
            .for_each(|segment| self.visit_path_segment_mut(segment));
        if let Some(ref mut qself) = &mut type_path.qself {
            self.visit_qself_mut(qself);
        } else {
            let first_and_second: Vec<_> = type_path
                .path
                .segments
                .clone()
                .into_iter()
                .take(2)
                .collect();
            if let (Some(first), Some(second)) = (first_and_second.get(0), first_and_second.get(1))
            {
                let trait_ = &self.0;
                let trailing = type_path.path.segments.iter().skip(1);
                if first.ident.to_string() == "Self" && self.1.contains(&second.ident) {
                    *type_path = parse2(quote![<Self as #trait_>::#(#trailing)::*]).unwrap();
                }
            }
        }
    }
}

pub(crate) fn transform(input: &mut ItemImpl) -> TokenStream {
    if let Some((_, path, _)) = input.trait_.clone() {
        let ty = path.clone();
        let associated_types: Vec<_> = input
            .items
            .iter()
            .filter_map(|item| {
                if let ImplItem::Type(associated_type) = item {
                    Some(associated_type.ident.clone())
                } else {
                    None
                }
            })
            .collect();
        QualifiesAssociatedTypes(ty, associated_types).visit_item_impl_mut(input);
    }
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
                method::transform(
                    &type_name,
                    method.to_token_stream(),
                    Some(&method.vis),
                    &method.attrs,
                    &method.sig,
                    &method.block.to_token_stream(),
                )
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
        let mut input: ItemImpl = parse2(quote! {
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
            transform(&mut input).to_string(),
            quote! {
                impl Cat {
                    #[meow]
                    #[meow]
                    fn meow(#[a] &self, #[b] count: usize) -> String {
                        #[cfg(test)]
                        if self.mry.is_some() {
                            if let Some(out) = mry::MOCK_DATA
                                .lock()
                                .get_mut_or_create::<(usize), String>(&self.mry, "Cat::meow")
                                ._inner_called((count.clone())) {
                                return out;
                            }
                        }
                        {
                            "meow".repeat(count)
                        }
                    }
                }

                impl Cat {
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
    fn support_generics() {
        let mut input: ItemImpl = parse2(quote! {
            impl<'a, A: Clone> Cat<'a, A> {
                fn meow<'a, B>(&'a self, count: usize) -> B {
                    "meow".repeat(count)
                }
            }
        })
        .unwrap();

        assert_eq!(
            transform(&mut input).to_string(),
            quote! {
                impl<'a, A: Clone> Cat<'a, A> {
                    fn meow<'a, B>(&'a self, count: usize) -> B {
                        #[cfg(test)]
                        if self.mry.is_some() {
                            if let Some(out) = mry::MOCK_DATA
                                .lock()
                                .get_mut_or_create::<(usize), B>(&self.mry, "Cat<'a, A>::meow")
                                ._inner_called((count.clone())) {
                                return out;
                            }
                        }
                        {
                            "meow".repeat(count)
                        }
                    }
                }

                impl <'a, A: Clone> Cat<'a, A> {
                    #[cfg(test)]
                    pub fn mock_meow<'mry>(&'mry mut self) -> mry::MockLocator<'mry, (usize), B, mry::Behavior1<(usize), B> > {
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
        let mut input: ItemImpl = parse2(quote! {
            impl<A: Clone> Animal<A> for Cat {
                fn name(&self) -> String {
                    self.name
                }
            }
        })
        .unwrap();

        assert_eq!(
            transform(&mut input).to_string(),
            quote! {
                impl<A: Clone> Animal<A> for Cat {
                    fn name(&self, ) -> String {
                        #[cfg(test)]
                        if self.mry.is_some() {
                            if let Some(out) = mry::MOCK_DATA
                                .lock()
                                .get_mut_or_create::<(), String>(&self.mry, "<Cat as Animal<A>>::name")
                                ._inner_called(()) {
                                return out;
                            }
                        }
                        {
                            self.name
                        }
                    }
                }

                impl Cat {
                    #[cfg(test)]
                    pub fn mock_name<'mry>(&'mry mut self) -> mry::MockLocator<'mry, (), String, mry::Behavior0<(), String> > {
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

    #[test]
    fn support_trait_with_associated_type() {
        let mut input: ItemImpl = parse2(quote! {
            impl Iterator for Cat {
                type Item = String;
                fn next(&self) -> Option<Self::Item> {
                    Some(self.name)
                }
            }
        })
        .unwrap();

        assert_eq!(
            transform(&mut input).to_string(),
            quote! {
                impl Iterator for Cat {
                    type Item = String;
                    fn next(&self, ) -> Option< <Self as Iterator>::Item> {
                        #[cfg(test)]
                        if self.mry.is_some() {
                            if let Some(out) = mry::MOCK_DATA
                                .lock()
                                .get_mut_or_create::<(), Option< <Self as Iterator>::Item> >(&self.mry, "<Cat as Iterator>::next")
                                ._inner_called(()) {
                                return out;
                            }
                        }
                        {
                            Some(self.name)
                        }
                    }
                }

                impl Cat {
                    #[cfg(test)]
                    pub fn mock_next<'mry>(&'mry mut self) -> mry::MockLocator<'mry, (), Option< <Self as Iterator>::Item >, mry::Behavior0<(), Option< <Self as Iterator>::Item> > > {
                        if self.mry.is_none() {
                            self.mry = mry::Mry::generate();
                        }
                        mry::MockLocator {
                            id: &self.mry,
                            name: "<Cat as Iterator>::next",
                            _phantom: Default::default(),
                        }
                    }
                }
            }
            .to_string()
        );
    }
}
