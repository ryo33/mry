use proc_macro2::{Span, TokenStream};
use quote::{format_ident, quote};
use syn::{
    parse_quote, punctuated::Punctuated, Attribute, FnArg, Ident, Pat, PatIdent, ReturnType,
    Signature, Type, Visibility,
};

#[allow(clippy::too_many_arguments)]
pub(crate) fn transform(
    mocks_tokens: TokenStream,  // `MOCKS.lock()`
    method_prefix: TokenStream, // `Self::`
    method_debug_prefix: &str,  // "Cat::"
    record_call_and_find_mock_output: TokenStream,
    vis: Option<&Visibility>,
    attrs: &[Attribute],
    sig: &Signature,
    body: &TokenStream,
) -> (TokenStream, TokenStream) {
    // Split into receiver and other inputs
    let mut receiver = None;
    let mut mock_receiver = None;
    let mut inputs = sig.inputs.iter().peekable();
    // If receiver exists
    if let Some(FnArg::Receiver(rec)) = inputs.peek() {
        receiver = Some(FnArg::Receiver(rec.clone()));
        mock_receiver = Some(quote![&mut self,]);
        // Skip the receiver
        inputs.next();
    }
    let inputs_without_receiver: Vec<_> = inputs
        .map(|input| {
            if let FnArg::Typed(typed_arg) = input {
                typed_arg.clone()
            } else {
                panic!("multiple receiver?");
            }
        })
        .collect();
    let mut bindings = Vec::new();

    let args_without_receiver: Vec<_> = inputs_without_receiver
        .iter()
        .enumerate()
        .map(|(i, input)| {
            if let Pat::Ident(_) = *input.pat {
                input.clone()
            } else {
                let pat = input.pat.clone();
                let arg_name = Ident::new(&format!("arg{}", i), Span::call_site());
                bindings.push((pat, arg_name.clone()));
                let ident = Pat::Ident(PatIdent {
                    attrs: Default::default(),
                    by_ref: Default::default(),
                    mutability: Default::default(),
                    ident: arg_name,
                    subpat: Default::default(),
                });
                let mut arg_with_type = (*input).clone();
                arg_with_type.pat = Box::new(ident.clone());
                arg_with_type
            }
        })
        .collect();
    let mut is_impl_future = false;
    let static_output_type = match &sig.output {
        ReturnType::Default => quote!(()),
        ReturnType::Type(_, ty) => {
            if let Some(output) = impl_future(ty) {
                is_impl_future = true;
                make_static_type(output)
            } else {
                make_static_type(ty)
            }
        }
    };
    let ident = sig.ident.clone();
    let mock_ident = Ident::new(&format!("mock_{}", ident), Span::call_site());
    let name = format!("{}{}", method_debug_prefix, ident);
    let bindings = bindings.iter().map(|(pat, arg)| quote![let #pat = #arg;]);
    let behavior_name = Ident::new(
        &format!("Behavior{}", inputs_without_receiver.len()),
        Span::call_site(),
    );
    struct Arg {
        org_ty: Type,
        owned_ty: Option<Type>,
        to_owned: TokenStream,
        name: Ident,
    }
    impl Arg {
        fn ty(&self) -> &Type {
            self.owned_ty.as_ref().unwrap_or(&self.org_ty)
        }
    }
    let args: Vec<Arg> = inputs_without_receiver
        .iter()
        .enumerate()
        .map(|(index, input)| {
            let org_ty = input.ty.as_ref().clone();
            let name = if let Pat::Ident(ident) = &*input.pat {
                ident.ident.clone()
            } else {
                format_ident!("arg{}", index)
            };
            let (owned_ty, to_owned) = make_owned_type(&name, &org_ty);
            Arg {
                org_ty,
                owned_ty: Some(owned_ty),
                to_owned,
                name,
            }
        })
        .collect();
    let mock_args = args.iter().map(|arg| {
        let name = &arg.name;
        let ty = arg.ty().clone();
        quote! {
            #name: impl Into<mry::ArgMatcher<#ty>>
        }
    });
    let into_matchers = args.iter().map(|arg| {
        let name = &arg.name;
        quote! {
            #name.into()
        }
    });
    let input_types = args.iter().map(|arg| arg.ty()).collect::<Vec<_>>();
    let owned_args = args.iter().map(|arg| {
        let name = &arg.name;
        let to_owned = &arg.to_owned;
        if arg.owned_ty.is_some() {
            quote![#to_owned]
        } else {
            quote![#name]
        }
    });
    let behavior_type = quote![mry::#behavior_name<(#(#input_types,)*), #static_output_type>];
    let allow_non_snake_case_or_blank = if ident.to_string().starts_with('_') {
        quote!(#[allow(non_snake_case)])
    } else {
        TokenStream::default()
    };
    let key = quote![std::any::Any::type_id(&#method_prefix #ident)];
    let mut sig = sig.clone();
    sig.inputs = Punctuated::from_iter(
        receiver
            .into_iter()
            .chain(args_without_receiver.iter().cloned().map(FnArg::Typed)),
    );
    let return_out = if is_impl_future {
        quote! {
            return async move { out };
        }
    } else {
        quote! {
            return out;
        }
    };
    (
        quote! {
            #(#attrs)*
            #vis #sig {
                #[cfg(debug_assertions)]
                if let Some(out) = #record_call_and_find_mock_output::<_, #static_output_type>(#key, #name, (#(#owned_args,)*)) {
                    #return_out
                }
                #(#bindings)*
                #body
            }
        },
        quote! {
            #[cfg(debug_assertions)]
            #allow_non_snake_case_or_blank
            #[must_use]
            pub fn #mock_ident (#mock_receiver #(#mock_args),*) -> mry::MockLocator<(#(#input_types,)*), #static_output_type, #behavior_type> {
                mry::MockLocator::new(
                    #mocks_tokens,
                    #key,
                    #name,
                    (#(#into_matchers,)*).into(),
                )
            }
        },
    )
}

pub fn make_owned_type(name: &Ident, ty: &Type) -> (Type, TokenStream) {
    if is_str(ty) {
        return (parse_quote!(String), quote![#name.to_string()]);
    }
    let owned = match ty {
        Type::Reference(ty) => {
            match &*ty.elem {
                Type::Slice(inner) => {
                    let elt_type = &inner.elem;
                    let map_ident = Ident::new("elem", Span::call_site());
                    let (inner_owned, inner_clone) = make_owned_type(&map_ident, elt_type);
                    let owned_ty = parse_quote!(Vec<#inner_owned>);
                    let clone = quote![#name.iter().map(|#map_ident: &#elt_type| -> #inner_owned { #inner_clone }).collect::<Vec<_>>()];
                    return (owned_ty, clone)
                },
                _ => ty.elem.as_ref().clone(),
            }
        },
        ty => ty.clone(),
    };
    let cloned = quote![<#owned>::clone(&#name)];
    (owned, cloned)
}

pub fn impl_future(ty: &Type) -> Option<&Type> {
    let syn::Type::ImplTrait(impl_trait) = ty else {
        return None;
    };
    let syn::TypeParamBound::Trait(bound) = &impl_trait.bounds[0] else {
        return None;
    };
    let last = bound.path.segments.last().unwrap();
    if last.ident != "Future" {
        return None;
    }
    let syn::PathArguments::AngleBracketed(args) = &last.arguments else {
        return None;
    };
    let syn::GenericArgument::AssocType(assoc) = &args.args[0] else {
        return None;
    };
    if assoc.ident != "Output" {
        return None;
    }
    Some(&assoc.ty)
}

pub fn make_static_type(ty: &Type) -> TokenStream {
    match &ty {
        Type::Reference(ty) => {
            let ty = &ty.elem;
            quote!(&'static #ty)
        }
        Type::ImplTrait(impl_trait) => {
            let bounds = &impl_trait.bounds;
            quote!(Box<dyn #bounds>)
        }
        ty => quote!(#ty),
    }
}

pub fn is_str(ty: &Type) -> bool {
    match ty {
        Type::Reference(ty) => {
            if let Type::Path(path) = &*ty.elem {
                if let Some(ident) = path.path.get_ident() {
                    return ident == "str";
                }
            }
            false
        }
        _ => false,
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use pretty_assertions::assert_eq;
    use quote::{quote, ToTokens};
    use syn::{parse2, parse_quote, ImplItemFn};

    trait ToString {
        fn to_string(&self) -> String;
    }

    impl ToString for (TokenStream, TokenStream) {
        fn to_string(&self) -> String {
            (self.0.to_string() + " " + &self.1.to_string())
                .to_string()
                .trim()
                .to_string()
        }
    }

    fn t(method: &ImplItemFn) -> (TokenStream, TokenStream) {
        transform(
            quote![self.mry.mocks()],
            quote![Self::],
            "Cat::",
            quote![self.mry.record_call_and_find_mock_output],
            Some(&method.vis),
            &method.attrs,
            &method.sig,
            &method
                .block
                .stmts
                .iter()
                .fold(TokenStream::default(), |mut stream, item| {
                    item.to_tokens(&mut stream);
                    stream
                }),
        )
    }

    fn remove_spaces(s: &str) -> String {
        s.chars().filter(|s| !s.is_whitespace()).collect()
    }

    #[test]
    fn test_make_owned_string() {
        let ident = Ident::new("var", Span::call_site());
        let str_t: Type = syn::parse_str("&str").unwrap();
        let (owned_type, converter) = make_owned_type(&ident, &str_t);
        assert_eq!(owned_type, parse_quote!(String));
        assert_eq!(remove_spaces(&converter.to_string()), "var.to_string()");
    }

    #[test]
    fn test_make_owned_slice() {
        let ident = Ident::new("var", Span::call_site());
        let slice_t: Type = syn::parse_str("&[String]").unwrap();
        let (owned_type, converter) = make_owned_type(&ident, &slice_t);
        dbg!(owned_type.to_token_stream().to_string());
        assert_eq!(owned_type, parse_quote!(Vec<String>));
        assert_eq!(remove_spaces(&converter.to_string()), "var.iter().map(|elem:String|->String{<String>::clone(&elem)}).collect::<Vec<_>>()");
    }

    #[test]
    fn test_make_owned_slice_of_str() {
        let ident = Ident::new("var", Span::call_site());
        let slice_t: Type = syn::parse_str("&[&str]").unwrap();
        let (owned_type, converter) = make_owned_type(&ident, &slice_t);
        dbg!(owned_type.to_token_stream().to_string());
        assert_eq!(owned_type, parse_quote!(Vec<String>));
        assert_eq!(remove_spaces(&converter.to_string()), "var.iter().map(|elem:String|->String{elem.to_string()}).collect::<Vec<_>>()");
    }

    #[test]
    fn adds_mock_function() {
        let input: ImplItemFn = parse2(quote! {
            fn meow(&self, count: usize) -> String {
                "meow".repeat(count)
            }
        })
        .unwrap();

        assert_eq!(
            t(&input).to_string(),
            quote! {
                fn meow(&self, count: usize) -> String {
                    #[cfg(debug_assertions)]
                    if let Some(out) = self.mry.record_call_and_find_mock_output::<_, String>(std::any::Any::type_id(&Self::meow), "Cat::meow", (<usize>::clone(&count),)) {
                        return out;
                    }
                    "meow".repeat(count)
                }

                #[cfg(debug_assertions)]
                #[must_use]
                pub fn mock_meow(&mut self, count: impl Into<mry::ArgMatcher<usize>>) -> mry::MockLocator<(usize,), String, mry::Behavior1<(usize,), String> > {
                    mry::MockLocator::new(
                        self.mry.mocks(),
                        std::any::Any::type_id(&Self::meow),
                        "Cat::meow",
                        (count.into(),).into(),
                    )
                }
            }
            .to_string()
        );
    }

    #[test]
    fn adds_allow_non_snake_case() {
        let input: ImplItemFn = parse2(quote! {
            fn _meow(&self, count: usize) -> String {
                "meow".repeat(count)
            }
        })
        .unwrap();

        assert_eq!(
            t(&input).to_string(),
            quote! {
                fn _meow(&self, count: usize) -> String {
                    #[cfg(debug_assertions)]
                    if let Some(out) = self.mry.record_call_and_find_mock_output::<_, String>(std::any::Any::type_id(&Self::_meow), "Cat::_meow", (<usize>::clone(&count),)) {
                        return out;
                    }
                    "meow".repeat(count)
                }

                #[cfg(debug_assertions)]
                #[allow(non_snake_case)]
                #[must_use]
                pub fn mock__meow(&mut self, count: impl Into<mry::ArgMatcher<usize>>) -> mry::MockLocator<(usize,), String, mry::Behavior1<(usize,), String> > {
                    mry::MockLocator::new(
                        self.mry.mocks(),
                        std::any::Any::type_id(&Self::_meow),
                        "Cat::_meow",
                        (count.into(),).into(),
                    )
                }
            }
            .to_string()
        );
    }

    #[test]
    fn empty_args() {
        let input: ImplItemFn = parse2(quote! {
            fn meow(&self) -> String {
                "meow".into()
            }
        })
        .unwrap();

        assert_eq!(
            t(&input).to_string(),
            quote! {
                fn meow(&self) -> String {
                    #[cfg(debug_assertions)]
                    if let Some(out) = self.mry.record_call_and_find_mock_output::<_, String>(std::any::Any::type_id(&Self::meow), "Cat::meow", ()) {
                        return out;
                    }
                    "meow".into()
                }

                #[cfg(debug_assertions)]
                #[must_use]
                pub fn mock_meow(&mut self,) -> mry::MockLocator<(), String, mry::Behavior0<(), String> > {
                    mry::MockLocator::new(
                        self.mry.mocks(),
                        std::any::Any::type_id(&Self::meow),
                        "Cat::meow",
                        ().into(),
                    )
                }
            }
            .to_string()
        );
    }

    #[test]
    fn multiple_args() {
        let input: ImplItemFn = parse2(quote! {
            fn meow(&self, base: String, count: usize) -> String {
                base.repeat(count)
            }
        })
        .unwrap();

        assert_eq!(
            t(&input).to_string(),
            quote! {
                fn meow(&self, base: String, count: usize) -> String {
                    #[cfg(debug_assertions)]
                    if let Some(out) = self.mry.record_call_and_find_mock_output::<_, String>(std::any::Any::type_id(&Self::meow), "Cat::meow", (<String>::clone(&base), <usize>::clone(&count),)) {
                        return out;
                    }
                    base.repeat(count)
                }

                #[cfg(debug_assertions)]
                #[must_use]
                pub fn mock_meow(&mut self, base: impl Into<mry::ArgMatcher<String>>, count: impl Into<mry::ArgMatcher<usize>>) -> mry::MockLocator<(String, usize,), String, mry::Behavior2<(String, usize,), String> > {
                    mry::MockLocator::new(
                        self.mry.mocks(),
                        std::any::Any::type_id(&Self::meow),
                        "Cat::meow",
                        (base.into(), count.into(),).into(),
                    )
                }
            }
            .to_string()
        );
    }

    #[test]
    fn input_reference_and_str() {
        let input: ImplItemFn = parse2(quote! {
            fn meow(&self, out: &'static mut String, base: &str, count: &usize) {
                *out = base.repeat(count);
            }
        })
        .unwrap();

        assert_eq!(
            t(&input).to_string(),
            quote! {
                fn meow(&self, out: &'static mut String, base: &str, count: &usize) {
                    #[cfg(debug_assertions)]
                    if let Some(out) = self.mry.record_call_and_find_mock_output::<_, ()>(std::any::Any::type_id(&Self::meow), "Cat::meow", (<String>::clone(&out), base.to_string(), <usize>::clone(&count),)) {
                        return out;
                    }
                    *out = base.repeat(count);
                }

                #[cfg(debug_assertions)]
                #[must_use]
                pub fn mock_meow(&mut self, out: impl Into <mry::ArgMatcher<String>>, base: impl Into<mry::ArgMatcher<String>>, count: impl Into<mry::ArgMatcher<usize>>)
                    -> mry::MockLocator<(String, String, usize,), (), mry::Behavior3<(String, String, usize,), ()> > {
                    mry::MockLocator::new(
                        self.mry.mocks(),
                        std::any::Any::type_id(&Self::meow),
                        "Cat::meow",
                        (out.into(), base.into(), count.into(),).into(),
                    )
                }
            }
            .to_string()
        );
    }

    #[test]
    fn supports_async() {
        let input: ImplItemFn = parse2(quote! {
            async fn meow(&self, count: usize) -> String{
                base().await.repeat(count);
            }
        })
        .unwrap();

        assert_eq!(
            t(&input).to_string(),
            quote! {
                async fn meow(&self, count: usize) -> String {
                    #[cfg(debug_assertions)]
                    if let Some(out) = self.mry.record_call_and_find_mock_output::<_, String>(std::any::Any::type_id(&Self::meow), "Cat::meow", (<usize>::clone(&count),)) {
                        return out;
                    }
                    base().await.repeat(count);
                }

                #[cfg(debug_assertions)]
                #[must_use]
                pub fn mock_meow(&mut self, count: impl Into<mry::ArgMatcher<usize>>) -> mry::MockLocator<(usize,), String, mry::Behavior1<(usize,), String> > {
                    mry::MockLocator::new(
                        self.mry.mocks(),
                        std::any::Any::type_id(&Self::meow),
                        "Cat::meow",
                        (count.into(),).into(),
                    )
                }
            }
            .to_string()
        );
    }

    #[test]
    fn support_pattern() {
        let input: ImplItemFn = parse2(quote! {
            fn meow(&self, A { name }: A, count: usize, _: String) -> String {
                name.repeat(count)
            }
        })
        .unwrap();

        assert_eq!(
            t(&input).to_string(),
            quote! {
                fn meow(&self, arg0: A, count: usize, arg2: String) -> String {
                    #[cfg(debug_assertions)]
                    if let Some(out) = self.mry.record_call_and_find_mock_output::<_, String>(std::any::Any::type_id(&Self::meow), "Cat::meow", (<A>::clone(&arg0), <usize>::clone(&count), <String>::clone(&arg2),)) {
                        return out;
                    }
                    let A { name } = arg0;
                    let _ = arg2;
                    name.repeat(count)
                }

                #[cfg(debug_assertions)]
                #[must_use]
                pub fn mock_meow(&mut self, arg0: impl Into<mry::ArgMatcher<A>>, count: impl Into<mry::ArgMatcher<usize>>, arg2: impl Into<mry::ArgMatcher<String>>) -> mry::MockLocator<(A, usize, String,), String, mry::Behavior3<(A, usize, String,), String> > {
                    mry::MockLocator::new(
                        self.mry.mocks(),
                        std::any::Any::type_id(&Self::meow),
                        "Cat::meow",
                        (arg0.into(), count.into(), arg2.into(),).into(),
                    )
                }
            }
            .to_string()
        );
    }

    #[test]
    fn respect_visibility() {
        let input: ImplItemFn = parse2(quote! {
            pub fn meow(&self, count: usize) -> String {
                "meow".repeat(count)
            }
        })
        .unwrap();

        assert_eq!(
            t(&input).0.to_string(),
            quote! {
                pub fn meow(&self, count: usize) -> String {
                    #[cfg(debug_assertions)]
                    if let Some(out) = self.mry.record_call_and_find_mock_output::<_, String>(std::any::Any::type_id(&Self::meow), "Cat::meow", (<usize>::clone(&count),)) {
                        return out;
                    }
                    "meow".repeat(count)
                }
            }
            .to_string()
        );
    }

    #[test]
    fn supports_mut() {
        let input: ImplItemFn = parse2(quote! {
            fn increment(&self, mut count: usize) -> usize {
                count += 1;
                count
            }
        })
        .unwrap();

        assert_eq!(
            t(&input).to_string(),
            quote! {
                fn increment(&self, mut count: usize) -> usize {
                    #[cfg(debug_assertions)]
                    if let Some(out) = self.mry.record_call_and_find_mock_output::<_, usize>(std::any::Any::type_id(&Self::increment), "Cat::increment", (<usize>::clone(&count),)) {
                        return out;
                    }
                    count += 1;
                    count
                }

                #[cfg(debug_assertions)]
                #[must_use]
                pub fn mock_increment(&mut self, count: impl Into<mry::ArgMatcher<usize>>) -> mry::MockLocator<(usize,), usize, mry::Behavior1<(usize,), usize> > {
                    mry::MockLocator::new(
                        self.mry.mocks(),
                        std::any::Any::type_id(&Self::increment),
                        "Cat::increment",
                        (count.into(),).into(),
                    )
                }
            }
            .to_string()
        );
    }

    #[test]
    fn supports_bounds() {
        let input: ImplItemFn = parse2(quote! {
            fn meow<'a, T: Display, const A: usize>(&self, a: usize) -> &'a String {
                todo!()
            }
        })
        .unwrap();

        assert_eq!(
            t(&input).to_string(),
            quote! {
                fn meow<'a, T: Display, const A: usize>(&self, a: usize) -> &'a String {
                    #[cfg(debug_assertions)]
                    if let Some(out) = self.mry.record_call_and_find_mock_output::<_, &'static String>(std::any::Any::type_id(&Self::meow), "Cat::meow", (<usize>::clone(&a),)) {
                        return out;
                    }
                    todo!()
                }

                #[cfg(debug_assertions)]
                #[must_use]
                pub fn mock_meow(&mut self, a: impl Into<mry::ArgMatcher<usize>>) -> mry::MockLocator<(usize,), &'static String, mry::Behavior1<(usize,), &'static String> > {
                    mry::MockLocator::new(
                        self.mry.mocks(),
                        std::any::Any::type_id(&Self::meow),
                        "Cat::meow",
                        (a.into(),).into(),
                    )
                }
            }
            .to_string()
        );
    }

    #[test]
    fn async_fn_in_trait() {
        let input: ImplItemFn = parse2(quote! {
            async fn meow(&self, count: usize) -> String {
                "meow".repeat(count)
            }
        })
        .unwrap();

        assert_eq!(
            t(&input).to_string(),
            quote! {
                async fn meow(&self, count: usize) -> String {
                    #[cfg(debug_assertions)]
                    if let Some(out) = self.mry.record_call_and_find_mock_output::<_, String>(std::any::Any::type_id(&Self::meow), "Cat::meow", (<usize>::clone(&count),)) {
                        return out;
                    }
                    "meow".repeat(count)
                }

                #[cfg(debug_assertions)]
                #[must_use]
                pub fn mock_meow(&mut self, count: impl Into<mry::ArgMatcher<usize>>) -> mry::MockLocator<(usize,), String, mry::Behavior1<(usize,), String> > {
                    mry::MockLocator::new(
                        self.mry.mocks(),
                        std::any::Any::type_id(&Self::meow),
                        "Cat::meow",
                        (count.into(),).into(),
                    )
                }
            }
            .to_string()
        );
    }

    #[test]
    fn return_position_impl_future() {
        let input: ImplItemFn = parse_quote! {
            fn meow(&self, count: usize) -> impl std::future::Future<Output = String> + Send {
                async move {
                    "meow".repeat(count)
                }
            }
        };

        assert_eq!(
                t(&input).to_string(),
                quote! {
                fn meow(&self, count: usize) -> impl std::future::Future<Output = String> + Send {
                    #[cfg(debug_assertions)]
                    if let Some(out) = self.mry.record_call_and_find_mock_output::<_, String>(std::any::Any::type_id(&Self::meow), "Cat::meow", (<usize>::clone(&count),)) {
                        return async move { out };
                    }
                    async move {
                        "meow".repeat(count)
                    }
                }

                #[cfg(debug_assertions)]
                #[must_use]
                pub fn mock_meow(&mut self, count: impl Into<mry::ArgMatcher<usize>>) -> mry::MockLocator<(usize,), String, mry::Behavior1<(usize,), String> > {
                    mry::MockLocator::new(
                        self.mry.mocks(),
                        std::any::Any::type_id(&Self::meow),
                        "Cat::meow",
                        (count.into(),).into(),
                    )
                }
            }.to_string()
        );
    }
}
