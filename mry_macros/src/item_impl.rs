use crate::{method, MryAttr};
use proc_macro2::TokenStream;
use quote::{quote, ToTokens};
use syn::visit_mut::VisitMut;
use syn::{parse2, FnArg, Ident, ImplItem, ItemImpl, Path};

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
            if let (Some(first), Some(second)) = (first_and_second.first(), first_and_second.get(1))
            {
                let trait_ = &self.0;
                let trailing = type_path.path.segments.iter().skip(1);
                if first.ident == "Self" && self.1.contains(&second.ident) {
                    *type_path = parse2(quote![<Self as #trait_>::#(#trailing)::*]).unwrap();
                }
            }
        }
    }
}

pub(crate) fn transform(mry_attr: &MryAttr, mut input: ItemImpl) -> TokenStream {
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
        QualifiesAssociatedTypes(ty, associated_types).visit_item_impl_mut(&mut input);
    }
    let generics = &input.generics;
    let struct_type = &input.self_ty;
    let mut trait_name = None;
    let trait_ = match &input.trait_ {
        Some((bang, path, for_)) => {
            trait_name = Some(path);
            quote! {
                #bang #path #for_
            }
        }
        None => TokenStream::default(),
    };

    struct LifetimeAnonymizer;
    impl VisitMut for LifetimeAnonymizer {
        fn visit_lifetime_mut(&mut self, lifetime: &mut syn::Lifetime) {
            lifetime.ident = Ident::new("_", lifetime.ident.span());
        }
    }

    let mut anonimized_struct = struct_type.clone();
    LifetimeAnonymizer.visit_type_mut(&mut anonimized_struct);

    let type_name;
    let qualified_type = if let Some(trait_name) = trait_name {
        let mut trait_name = trait_name.clone();
        LifetimeAnonymizer.visit_path_mut(&mut trait_name);
        let tokens = quote![<#anonimized_struct as #trait_name>];
        type_name = tokens.to_string();
        tokens
    } else {
        type_name = struct_type.to_token_stream().to_string();
        quote![<#anonimized_struct>]
    };

    // Pretty print type name
    let type_name = type_name
        .replace(" ,", ",")
        .replace(" >", ">")
        .replace(" <", "<")
        .replace("< ", "<");

    let (members, impl_members): (Vec<_>, Vec<_>) = input
        .items
        .iter()
        .map(|item| {
            if let ImplItem::Fn(method) = item {
                if mry_attr.should_skip_method(&method.sig.ident) {
                    return (item.to_token_stream(), TokenStream::default());
                }
                if let Some(FnArg::Receiver(_)) = method.sig.inputs.first() {
                    method::transform(
                        mry_attr,
                        quote![self.mry.mocks()],
                        quote![#qualified_type::],
                        &(type_name.clone() + "::"),
                        quote![self.mry.record_call_and_find_mock_output],
                        Some(&method.vis),
                        &method.attrs,
                        &method.sig,
                        &method.block.stmts.iter().fold(
                            TokenStream::default(),
                            |mut stream, item| {
                                item.to_tokens(&mut stream);
                                stream
                            },
                        ),
                        false,
                    )
                } else {
                    method::transform(
                        mry_attr,
                        quote![mry::get_static_mocks()],
                        quote![#qualified_type::],
                        &(type_name.clone() + "::"),
                        quote![mry::static_record_call_and_find_mock_output],
                        Some(&method.vis),
                        &method.attrs,
                        &method.sig,
                        &method.block.stmts.iter().fold(
                            TokenStream::default(),
                            |mut stream, item| {
                                item.to_tokens(&mut stream);
                                stream
                            },
                        ),
                        false,
                    )
                }
            } else {
                (item.to_token_stream(), TokenStream::default())
            }
        })
        .unzip();

    let where_clause = &generics.where_clause;

    quote! {
        impl #generics #trait_ #struct_type #where_clause {
            #(#members)*
        }

        impl #generics #struct_type #where_clause {
            #(#impl_members)*
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
            transform(&MryAttr::default(), input).to_string(),
            quote! {
                impl Cat {
                    #[meow]
                    #[meow]
                    #[cfg_attr(debug_assertions, track_caller)]
                    fn meow(#[a] &self, #[b] count: usize) -> String {
                        #[cfg(debug_assertions)]
                        if let Some(out) = self.mry.record_call_and_find_mock_output::<_, String>(std::any::Any::type_id(&<Cat>::meow), "Cat::meow", (<usize>::clone(&count),)) {
                            return out;
                        }
                        (move || {
                            "meow".repeat(count)
                        })()
                    }
                }

                impl Cat {
                    #[cfg(debug_assertions)]
                    #[must_use]
                    pub fn mock_meow(&mut self, count: impl Into<mry::ArgMatcher<usize>>) -> mry::MockLocator<(usize,), String, String, mry::Behavior1<(usize,), String> > {
                        mry::MockLocator::new(
                            self.mry.mocks(),
                            std::any::Any::type_id(&<Cat>::meow),
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
            transform(&MryAttr::default(), input).to_string(),
            quote! {
                impl<'a, A: Clone> Cat<'a, A> {
                    #[cfg_attr(debug_assertions, track_caller)]
                    fn meow<'a, B>(&'a self, count: usize) -> B {
                        #[cfg(debug_assertions)]
                        if let Some(out) = self.mry.record_call_and_find_mock_output::<_, B>(std::any::Any::type_id(&<Cat<'_, A> >::meow::<B>), "Cat<'a, A>::meow", (<usize>::clone(&count),)) {
                            return out;
                        }
                        (move || {
                            "meow".repeat(count)
                        })()
                    }
                }

                impl <'a, A: Clone> Cat<'a, A> {
                    #[cfg(debug_assertions)]
                    #[must_use]
                    pub fn mock_meow<'a, B>(&mut self, count: impl Into<mry::ArgMatcher<usize>>) -> mry::MockLocator<(usize,), B, B, mry::Behavior1<(usize,), B> > {
                        mry::MockLocator::new(
                            self.mry.mocks(),
                            std::any::Any::type_id(&<Cat<'_, A> >::meow::<B>),
                            "Cat<'a, A>::meow",
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
            transform(&MryAttr::default(), input).to_string(),
            quote! {
                impl<A: Clone> Animal<A> for Cat {
                    #[cfg_attr(debug_assertions, track_caller)]
                    fn name(&self) -> String {
                        #[cfg(debug_assertions)]
                        if let Some(out) = self.mry.record_call_and_find_mock_output::<_, String>(std::any::Any::type_id(&<Cat as Animal<A> >::name), "<Cat as Animal<A>>::name", ()) {
                            return out;
                        }
                        (move || {
                            self.name
                        })()
                    }
                }

                impl<A: Clone> Cat {
                    #[cfg(debug_assertions)]
                    #[must_use]
                    pub fn mock_name(&mut self,) -> mry::MockLocator<(), String, String, mry::Behavior0<(), String> > {
                        mry::MockLocator::new(
                            self.mry.mocks(),
                            std::any::Any::type_id(&< Cat as Animal < A > >::name),
                            "<Cat as Animal<A>>::name",
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
    fn support_trait_with_associated_type() {
        let input: ItemImpl = parse2(quote! {
            impl Iterator for Cat {
                type Item = String;
                fn next(&self) -> Option<Self::Item> {
                    Some(self.name)
                }
            }
        })
        .unwrap();

        assert_eq!(
            transform(&MryAttr::default(), input).to_string(),
            quote! {
                impl Iterator for Cat {
                    type Item = String;
                    #[cfg_attr(debug_assertions, track_caller)]
                    fn next(&self) -> Option< <Self as Iterator>::Item> {
                        #[cfg(debug_assertions)]
                        if let Some(out) = self.mry.record_call_and_find_mock_output::<_, Option< <Self as Iterator>::Item> >(std::any::Any::type_id(&<Cat as Iterator>::next), "<Cat as Iterator>::next", ()) {
                            return out;
                        }
                        (move || {
                            Some(self.name)
                        })()
                    }
                }

                impl Cat {
                    #[cfg(debug_assertions)]
                    #[must_use]
                    pub fn mock_next(&mut self,) -> mry::MockLocator<(), Option< <Self as Iterator>::Item >, Option< <Self as Iterator>::Item >, mry::Behavior0<(), Option< <Self as Iterator>::Item> > > {
                        mry::MockLocator::new(
                            self.mry.mocks(),
                            std::any::Any::type_id(&<Cat as Iterator>::next),
                            "<Cat as Iterator>::next",
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
    fn support_associated_functions() {
        let input: ItemImpl = parse2(quote! {
            impl Cat {
                fn meow(count: usize) -> String {
                    "meow".repeat(count)
                }
            }
        })
        .unwrap();

        assert_eq!(
            transform(&MryAttr::default(), input).to_string(),
            quote! {
                impl Cat {
                    #[cfg_attr(debug_assertions, track_caller)]
                    fn meow(count: usize) -> String {
                        #[cfg(debug_assertions)]
                        if let Some(out) = mry::static_record_call_and_find_mock_output::<_, String>(std::any::Any::type_id(&<Cat>::meow), "Cat::meow", (<usize>::clone(&count),)) {
                            return out;
                        }
                        (move || {
                            "meow".repeat(count)
                        })()
                    }
                }

                impl Cat {
                    #[cfg(debug_assertions)]
                    #[must_use]
                    pub fn mock_meow(count: impl Into<mry::ArgMatcher<usize>>) -> mry::MockLocator<(usize,), String, String, mry::Behavior1<(usize,), String> > {
                        mry::MockLocator::new(
                            mry::get_static_mocks(),
                            std::any::Any::type_id(&<Cat>::meow),
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
    fn test_skip_in_impl() {
        let input: ItemImpl = parse2(quote! {
            impl Cat {
                fn skipped(&self, rc: Rc<String>) -> String {
                    format!("skipped {} meows", self.name)
                }
            }
        })
        .unwrap();

        let attr = MryAttr::from_meta(&parse_quote! {
            mry(skip_fns(skipped))
        })
        .unwrap();

        assert_eq!(
            transform(&attr, input).to_string(),
            quote! {
                impl Cat {
                    fn skipped(&self, rc: Rc<String>) -> String {
                        format!("skipped {} meows", self.name)
                    }
                }

                impl Cat {
                }
            }
            .to_string()
        );
    }

    #[test]
    fn test_skip_in_impl_with_trait() {
        let attr = MryAttr::from_meta(&parse_quote! {
            mry(skip_fns(skipped))
        })
        .unwrap();

        let input: ItemImpl = parse2(quote! {
            impl Cat for MockCat {
                fn skipped(&self, rc: Rc<String>) -> String {
                    format!("skipped {} meows", self.name)
                }
            }
        })
        .unwrap();

        assert_eq!(
            transform(&attr, input).to_string(),
            quote! {
                impl Cat for MockCat {
                    fn skipped(&self, rc: Rc<String>) -> String {
                        format!("skipped {} meows", self.name)
                    }
                }

                impl MockCat {
                }
            }
            .to_string()
        );
    }

    #[test]
    fn preserves_where_clause_for_impl() {
        let input: ItemImpl = parse2(quote! {
            impl<T> Cat<T>
            where
                T: Clone + Send,
            {
                fn meow(&self, value: T) -> String {
                    format!("meow: {:?}", value)
                }
            }
        })
        .unwrap();

        assert_eq!(
            transform(&MryAttr::default(), input).to_string(),
            quote! {
                impl<T> Cat<T>
                where
                    T: Clone + Send,
                {
                    #[cfg_attr(debug_assertions, track_caller)]
                    fn meow(&self, value: T) -> String {
                        #[cfg(debug_assertions)]
                        if let Some(out) = self.mry.record_call_and_find_mock_output::<_, String>(std::any::Any::type_id(&<Cat<T> >::meow), "Cat<T>::meow", (<T>::clone(&value),)) {
                            return out;
                        }
                        (move || {
                            format!("meow: {:?}", value)
                        })()
                    }
                }

                impl<T> Cat<T>
                where
                    T: Clone + Send,
                {
                    #[cfg(debug_assertions)]
                    #[must_use]
                    pub fn mock_meow(&mut self, value: impl Into<mry::ArgMatcher<T>>) -> mry::MockLocator<(T,), String, String, mry::Behavior1<(T,), String> > {
                        mry::MockLocator::new(
                            self.mry.mocks(),
                            std::any::Any::type_id(&<Cat<T> >::meow),
                            "Cat<T>::meow",
                            (value.into(),).into(),
                            std::convert::identity,
                        )
                    }
                }
            }
            .to_string()
        );
    }

    #[test]
    fn preserves_where_clause_for_impl_trait() {
        let input: ItemImpl = parse2(quote! {
            impl<T> Animal<T> for Cat<T>
            where
                T: Clone + Send + 'static,
            {
                fn name(&self, prefix: T) -> String {
                    format!("{:?}: {}", prefix, self.name)
                }
            }
        })
        .unwrap();

        assert_eq!(
            transform(&MryAttr::default(), input).to_string(),
            quote! {
                impl<T> Animal<T> for Cat<T>
                where
                    T: Clone + Send + 'static,
                {
                    #[cfg_attr(debug_assertions, track_caller)]
                    fn name(&self, prefix: T) -> String {
                        #[cfg(debug_assertions)]
                        if let Some(out) = self.mry.record_call_and_find_mock_output::<_, String>(std::any::Any::type_id(&<Cat<T> as Animal<T>  >::name), "<Cat<T> as Animal<T>>::name", (<T>::clone(&prefix),)) {
                            return out;
                        }
                        (move || {
                            format!("{:?}: {}", prefix, self.name)
                        })()
                    }
                }

                impl<T> Cat<T>
                where
                    T: Clone + Send + 'static,
                {
                    #[cfg(debug_assertions)]
                    #[must_use]
                    pub fn mock_name(&mut self, prefix: impl Into<mry::ArgMatcher<T>>) -> mry::MockLocator<(T,), String, String, mry::Behavior1<(T,), String> > {
                        mry::MockLocator::new(
                            self.mry.mocks(),
                            std::any::Any::type_id(&< Cat < T > as Animal < T > >::name),
                            "<Cat<T> as Animal<T>>::name",
                            (prefix.into(),).into(),
                            std::convert::identity,
                        )
                    }
                }
            }
            .to_string()
        );
    }
}
