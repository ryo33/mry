use proc_macro2::{Span, TokenStream};
use quote::{format_ident, quote};
use syn::{
    parse_quote, punctuated::Punctuated, Attribute, FnArg, Ident, Pat, PatIdent, ReturnType,
    Signature, Type, TypeArray, TypeSlice, Visibility,
};

use crate::attrs::MryAttr;

fn has_track_caller_attr(attrs: &[Attribute]) -> bool {
    attrs.iter().any(|attr| {
        if attr.path().is_ident("track_caller") {
            return true;
        }
        if attr.path().is_ident("cfg_attr") {
            if let Ok(meta) = attr.meta.require_list() {
                let tokens = &meta.tokens;
                let token_str = tokens.to_string();
                return token_str.contains("track_caller");
            }
        }
        false
    })
}

#[allow(clippy::too_many_arguments)]
pub(crate) fn transform(
    mry_attr: &MryAttr,
    mocks_tokens: TokenStream,  // `MOCKS.lock()`
    method_prefix: TokenStream, // `Self::`
    method_debug_prefix: &str,  // "Cat::"
    record_call_and_find_mock_output: TokenStream,
    vis: Option<&Visibility>,
    attrs: &[Attribute],
    sig: &Signature,
    body: &TokenStream,
    // The body must not be wrapped in a closure even if the method has no `#[track_caller]`
    force_location_tracking: bool,
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
                (typed_arg.clone(), mry_attr.test_skip_args(&typed_arg.ty))
            } else {
                panic!("multiple receiver?");
            }
        })
        .collect();
    let mut bindings = Vec::new();

    let args_without_receiver: Vec<_> = inputs_without_receiver
        .iter()
        .enumerate()
        .map(|(i, (input, skip))| {
            if *skip {
                input.clone()
            } else if let Pat::Ident(_) = *input.pat {
                input.clone()
            } else {
                let pat = input.pat.clone();
                let arg_name = Ident::new(&format!("arg{i}"), Span::call_site());
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
    let OutputType {
        static_type: static_output_type,
        behavior_type: behavior_output_type,
        send_wrapper: out_is_send_wrapper,
    } = match &sig.output {
        ReturnType::Default => OutputType {
            static_type: quote!(()),
            behavior_type: quote!(()),
            send_wrapper: false,
        },
        ReturnType::Type(_, ty) => {
            if let Some(output) = impl_future(ty) {
                is_impl_future = true;
                make_output_type(mry_attr, output)
            } else {
                make_output_type(mry_attr, ty)
            }
        }
    };
    let ident = sig.ident.clone();
    let mock_ident = Ident::new(&format!("mock_{ident}"), Span::call_site());
    let name = format!("{method_debug_prefix}{ident}");
    let bindings = bindings
        .iter()
        .map(|(pat, arg)| quote![let #pat = #arg;])
        .collect::<Vec<_>>();
    let behavior_name = Ident::new(
        &format!(
            "Behavior{}{}",
            if out_is_send_wrapper {
                "SendWrapper"
            } else {
                ""
            },
            inputs_without_receiver
                .iter()
                .filter(|(_, skip)| !*skip)
                .count()
        ),
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
        .filter_map(|(index, (input, skip))| {
            if *skip {
                return None;
            }
            let org_ty = input.ty.as_ref().clone();
            let name = if let Pat::Ident(ident) = &*input.pat {
                ident.ident.clone()
            } else {
                format_ident!("arg{}", index)
            };
            let (owned_ty, to_owned) = make_owned_type(mry_attr, &name, &org_ty);
            Some(Arg {
                org_ty,
                owned_ty: Some(owned_ty),
                to_owned,
                name,
            })
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
    let behavior_type = quote![mry::#behavior_name<(#(#input_types,)*), #behavior_output_type>];
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

    let out = if out_is_send_wrapper {
        quote!(mry::send_wrapper::SendWrapper::take(out))
    } else {
        quote!(out)
    };

    let return_out = if is_impl_future {
        quote!(return async move { #out };)
    } else {
        quote!(return #out;)
    };

    let ret_to_out = if out_is_send_wrapper {
        quote!(|out| mry::send_wrapper::SendWrapper::new(out))
    } else {
        quote!(std::convert::identity)
    };

    let has_track_caller_attr = has_track_caller_attr(attrs);
    let track_caller_attr = if has_track_caller_attr {
        quote!()
    } else {
        quote!(#[cfg_attr(debug_assertions, track_caller)])
    };

    // This ensures that the panic within the body is located at the correct line even if the method itself is not marked with `#[track_caller]`
    let body = if has_track_caller_attr || force_location_tracking {
        // If the method itself is marked with `#[track_caller]`, we can just use the body as is
        body.clone()
    } else if sig.asyncness.is_some() {
        quote! {
            (move || async move { #body })().await
        }
    } else {
        quote! {
            (move || { #body })()
        }
    };

    (
        quote! {
            #(#attrs)*
            #track_caller_attr
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
            pub fn #mock_ident (#mock_receiver #(#mock_args),*) -> mry::MockLocator<(#(#input_types,)*), #static_output_type, #behavior_output_type, #behavior_type> {
                mry::MockLocator::new(
                    #mocks_tokens,
                    #key,
                    #name,
                    (#(#into_matchers,)*).into(),
                    #ret_to_out,
                )
            }
        },
    )
}

pub fn make_owned_type(mry_attr: &MryAttr, name: &Ident, ty: &Type) -> (Type, TokenStream) {
    if is_str(ty) {
        return (parse_quote!(String), quote![#name.to_string()]);
    }
    fn make_owned_type_slice(mry_attr: &MryAttr, name: &Ident, elem: &Type) -> (Type, TokenStream) {
        let map_ident = Ident::new("elem", Span::call_site());
        let (inner_owned, inner_clone) = make_owned_type(mry_attr, &map_ident, elem);
        let owned_ty = parse_quote!(Vec<#inner_owned>);
        let clone = quote![#name.iter().map(|#map_ident: &#elem| -> #inner_owned { #inner_clone }).collect::<Vec<_>>()];
        (owned_ty, clone)
    }
    let owned = match ty {
        Type::Array(TypeArray { elem, .. }) => {
            return make_owned_type_slice(mry_attr, name, elem);
        }
        Type::Reference(ty) => match &*ty.elem {
            Type::Slice(TypeSlice { elem, .. }) | Type::Array(TypeArray { elem, .. }) => {
                return make_owned_type_slice(mry_attr, name, elem)
            }
            _ => ty.elem.as_ref(),
        },
        Type::Ptr(_) => {
            let owned_ty = parse_quote!(mry::send_wrapper::SendWrapper<#ty>);
            let clone = quote![mry::send_wrapper::SendWrapper::new(<#ty>::clone(&#name))];
            return (owned_ty, clone);
        }
        ty => ty,
    };
    let cloned = quote![<#owned>::clone(&#name)];
    if mry_attr.test_non_send(owned) {
        let owned_ty = parse_quote!(mry::send_wrapper::SendWrapper<#owned>);
        let clone = quote![mry::send_wrapper::SendWrapper::new(#cloned)];
        return (owned_ty, clone);
    }
    (owned.clone(), cloned)
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

struct OutputType {
    static_type: TokenStream,
    behavior_type: TokenStream,
    send_wrapper: bool,
}

fn make_output_type(mry_attr: &MryAttr, ty: &Type) -> OutputType {
    let mut send_wrapper = mry_attr.test_non_send(ty);
    let ty = match ty {
        Type::Reference(ty) => {
            let ty = &ty.elem;
            quote!(&'static #ty)
        }
        Type::ImplTrait(impl_trait) => {
            let bounds = &impl_trait.bounds;
            quote!(Box<dyn #bounds>)
        }
        Type::Ptr(_) => {
            send_wrapper = true;
            quote!(#ty)
        }
        ty => quote!(#ty),
    };
    let static_type = if send_wrapper {
        quote!(mry::send_wrapper::SendWrapper<#ty>)
    } else {
        quote!(#ty)
    };
    OutputType {
        static_type,
        behavior_type: quote!(#ty),
        send_wrapper,
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
    use crate::attrs::NotSend;

    use super::*;
    use darling::FromMeta as _;
    use pretty_assertions::assert_eq;
    use quote::{quote, ToTokens};
    use syn::{parse2, parse_quote, ImplItemFn, Meta};

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
        t_with_attr(parse_quote!(mry()), method)
    }

    fn t_with_attr(attr: Meta, method: &ImplItemFn) -> (TokenStream, TokenStream) {
        let attr = MryAttr::from_meta(&attr).unwrap();
        transform(
            &attr,
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
            false,
        )
    }

    fn remove_spaces(s: &str) -> String {
        s.chars().filter(|s| !s.is_whitespace()).collect()
    }

    #[test]
    fn test_make_owned_string() {
        let ident = Ident::new("var", Span::call_site());
        let str_t: Type = syn::parse_str("&str").unwrap();
        let (owned_type, converter) = make_owned_type(&MryAttr::default(), &ident, &str_t);
        assert_eq!(owned_type, parse_quote!(String));
        assert_eq!(remove_spaces(&converter.to_string()), "var.to_string()");
    }

    #[test]
    fn test_make_owned_slice() {
        let ident = Ident::new("var", Span::call_site());
        let slice_t: Type = syn::parse_str("&[String]").unwrap();
        let (owned_type, converter) = make_owned_type(&MryAttr::default(), &ident, &slice_t);
        dbg!(owned_type.to_token_stream().to_string());
        assert_eq!(owned_type, parse_quote!(Vec<String>));
        assert_eq!(
            remove_spaces(&converter.to_string()),
            "var.iter().map(|elem:&String|->String{<String>::clone(&elem)}).collect::<Vec<_>>()"
        );
    }

    #[test]
    fn test_make_owned_slice_of_str() {
        let ident = Ident::new("var", Span::call_site());
        let slice_t: Type = syn::parse_str("&[&str]").unwrap();
        let (owned_type, converter) = make_owned_type(&MryAttr::default(), &ident, &slice_t);
        dbg!(owned_type.to_token_stream().to_string());
        assert_eq!(owned_type, parse_quote!(Vec<String>));
        assert_eq!(
            remove_spaces(&converter.to_string()),
            "var.iter().map(|elem:&&str|->String{elem.to_string()}).collect::<Vec<_>>()"
        );
    }

    #[test]
    fn test_make_owned_raw_pointer() {
        let ident = Ident::new("var", Span::call_site());
        let ptr_t: Type = syn::parse_str("*mut String").unwrap();
        let (owned_type, converter) = make_owned_type(&MryAttr::default(), &ident, &ptr_t);

        assert_eq!(
            owned_type,
            parse_quote!(mry::send_wrapper::SendWrapper<*mut String>)
        );
        assert_eq!(
            remove_spaces(&converter.to_string()),
            "mry::send_wrapper::SendWrapper::new(<*mutString>::clone(&var))"
        );
    }

    #[test]
    fn test_make_owned_with_non_send() {
        let attr = MryAttr {
            non_send: Some(NotSend(vec![parse_quote!(A::B), parse_quote!(C)])),
            ..Default::default()
        };
        let ident = Ident::new("var", Span::call_site());
        let a_b: Type = syn::parse_str("A::B").unwrap();
        let c_d: Type = syn::parse_str("C::D").unwrap();
        let (owned_type, converter) = make_owned_type(&attr, &ident, &a_b);
        assert_eq!(
            owned_type,
            parse_quote!(mry::send_wrapper::SendWrapper<A::B>)
        );
        assert_eq!(
            remove_spaces(&converter.to_string()),
            "mry::send_wrapper::SendWrapper::new(<A::B>::clone(&var))"
        );
        let (owned_type, converter) = make_owned_type(&attr, &ident, &c_d);
        assert_eq!(
            owned_type,
            parse_quote!(mry::send_wrapper::SendWrapper<C::D>)
        );
        assert_eq!(
            remove_spaces(&converter.to_string()),
            "mry::send_wrapper::SendWrapper::new(<C::D>::clone(&var))"
        );
    }

    #[test]
    fn test_make_owned_slice_of_non_send() {
        let attr = MryAttr {
            non_send: Some(NotSend(vec![parse_quote!(A)])),
            ..Default::default()
        };
        let ident = Ident::new("var", Span::call_site());
        let a: Type = syn::parse_str("&[A]").unwrap();
        let (owned_type, converter) = make_owned_type(&attr, &ident, &a);
        assert_eq!(
            owned_type,
            parse_quote!(Vec<mry::send_wrapper::SendWrapper<A>>)
        );
        assert_eq!(
            remove_spaces(&converter.to_string()),
            "var.iter().map(|elem:&A|->mry::send_wrapper::SendWrapper<A>{mry::send_wrapper::SendWrapper::new(<A>::clone(&elem))}).collect::<Vec<_>>()"
        );
    }

    #[test]
    fn test_make_owned_slice_of_ptr() {
        let ident = Ident::new("var", Span::call_site());
        let a: Type = syn::parse_str("&[*mut String]").unwrap();
        let (owned_type, converter) = make_owned_type(&MryAttr::default(), &ident, &a);
        assert_eq!(
            owned_type,
            parse_quote!(Vec<mry::send_wrapper::SendWrapper<*mut String>>)
        );
        assert_eq!(
            remove_spaces(&converter.to_string()),
            "var.iter().map(|elem:&*mutString|->mry::send_wrapper::SendWrapper<*mutString>{mry::send_wrapper::SendWrapper::new(<*mutString>::clone(&elem))}).collect::<Vec<_>>()"
        );
    }

    #[test]
    fn test_make_owned_array_of_ptr() {
        let ident = Ident::new("var", Span::call_site());
        let a: Type = syn::parse_str("[*mut String; 3]").unwrap();
        let (owned_type, converter) = make_owned_type(&MryAttr::default(), &ident, &a);
        assert_eq!(
            owned_type,
            parse_quote!(Vec<mry::send_wrapper::SendWrapper<*mut String>>)
        );
        assert_eq!(
            remove_spaces(&converter.to_string()),
            "var.iter().map(|elem:&*mutString|->mry::send_wrapper::SendWrapper<*mutString>{mry::send_wrapper::SendWrapper::new(<*mutString>::clone(&elem))}).collect::<Vec<_>>()"
        );
    }

    #[test]
    fn test_make_static_raw_pointer() {
        let ptr_t: Type = syn::parse_str("*const String").unwrap();
        let output = make_output_type(&MryAttr::default(), &ptr_t);

        assert_eq!(
            output.static_type.to_string(),
            "mry :: send_wrapper :: SendWrapper < * const String >"
        );
        assert_eq!(output.behavior_type.to_string(), "* const String");

        let ptr_t: Type = syn::parse_str("*mut String").unwrap();
        let output = make_output_type(&MryAttr::default(), &ptr_t);

        assert_eq!(
            output.static_type.to_string(),
            "mry :: send_wrapper :: SendWrapper < * mut String >"
        );
        assert_eq!(output.behavior_type.to_string(), "* mut String");
    }

    #[test]
    fn test_make_static_with_non_send() {
        let attr = MryAttr {
            non_send: Some(NotSend(vec![parse_quote!(A::B), parse_quote!(C)])),
            ..Default::default()
        };
        let a: Type = parse_quote!(A);
        let a_b: Type = parse_quote!(A::B);
        let c_d: Type = parse_quote!(C::D);
        let a_type = make_output_type(&attr, &a);
        let a_b_type = make_output_type(&attr, &a_b);
        let c_d_type = make_output_type(&attr, &c_d);

        assert_eq!(a_type.static_type.to_string(), "A");
        assert_eq!(
            remove_spaces(&a_b_type.static_type.to_string()),
            "mry::send_wrapper::SendWrapper<A::B>"
        );
        assert_eq!(
            remove_spaces(&c_d_type.static_type.to_string()),
            "mry::send_wrapper::SendWrapper<C::D>"
        );
        assert_eq!(a_type.behavior_type.to_string(), "A");
        assert_eq!(remove_spaces(&a_b_type.behavior_type.to_string()), "A::B");
        assert_eq!(remove_spaces(&c_d_type.behavior_type.to_string()), "C::D");
    }

    #[test]
    fn test_non_send_ref() {
        let attr = MryAttr {
            non_send: Some(NotSend(vec![parse_quote!(A)])),
            ..Default::default()
        };
        let a: Type = parse_quote!(&A);
        let a_type = make_output_type(&attr, &a);
        assert_eq!(
            remove_spaces(&a_type.static_type.to_string()),
            "mry::send_wrapper::SendWrapper<&'staticA>"
        );
        assert_eq!(&a_type.behavior_type.to_string(), "& 'static A");
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
                #[cfg_attr(debug_assertions, track_caller)]
                fn meow(&self, count: usize) -> String {
                    #[cfg(debug_assertions)]
                    if let Some(out) = self.mry.record_call_and_find_mock_output::<_, String>(std::any::Any::type_id(&Self::meow), "Cat::meow", (<usize>::clone(&count),)) {
                        return out;
                    }
                    (move || {
                        "meow".repeat(count)
                    })()
                }

                #[cfg(debug_assertions)]
                #[must_use]
                pub fn mock_meow(&mut self, count: impl Into<mry::ArgMatcher<usize>>) -> mry::MockLocator<(usize,), String, String, mry::Behavior1<(usize,), String> > {
                    mry::MockLocator::new(
                        self.mry.mocks(),
                        std::any::Any::type_id(&Self::meow),
                        "Cat::meow",
                        (count.into(),).into(),
                        std::convert::identity,
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
                #[cfg_attr(debug_assertions, track_caller)]
                fn _meow(&self, count: usize) -> String {
                    #[cfg(debug_assertions)]
                    if let Some(out) = self.mry.record_call_and_find_mock_output::<_, String>(std::any::Any::type_id(&Self::_meow), "Cat::_meow", (<usize>::clone(&count),)) {
                        return out;
                    }
                    (move || {
                        "meow".repeat(count)
                    })()
                }

                #[cfg(debug_assertions)]
                #[allow(non_snake_case)]
                #[must_use]
                pub fn mock__meow(&mut self, count: impl Into<mry::ArgMatcher<usize>>) -> mry::MockLocator<(usize,), String, String, mry::Behavior1<(usize,), String> > {
                    mry::MockLocator::new(
                        self.mry.mocks(),
                        std::any::Any::type_id(&Self::_meow),
                        "Cat::_meow",
                        (count.into(),).into(),
                        std::convert::identity,
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
                #[cfg_attr(debug_assertions, track_caller)]
                fn meow(&self) -> String {
                    #[cfg(debug_assertions)]
                    if let Some(out) = self.mry.record_call_and_find_mock_output::<_, String>(std::any::Any::type_id(&Self::meow), "Cat::meow", ()) {
                        return out;
                    }
                    (move || {
                        "meow".into()
                    })()
                }

                #[cfg(debug_assertions)]
                #[must_use]
                pub fn mock_meow(&mut self,) -> mry::MockLocator<(), String, String, mry::Behavior0<(), String> > {
                    mry::MockLocator::new(
                        self.mry.mocks(),
                        std::any::Any::type_id(&Self::meow),
                        "Cat::meow",
                        ().into(),
                        std::convert::identity,
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
                #[cfg_attr(debug_assertions, track_caller)]
                fn meow(&self, base: String, count: usize) -> String {
                    #[cfg(debug_assertions)]
                    if let Some(out) = self.mry.record_call_and_find_mock_output::<_, String>(std::any::Any::type_id(&Self::meow), "Cat::meow", (<String>::clone(&base), <usize>::clone(&count),)) {
                        return out;
                    }
                    (move || {
                        base.repeat(count)
                    })()
                }

                #[cfg(debug_assertions)]
                #[must_use]
                pub fn mock_meow(&mut self, base: impl Into<mry::ArgMatcher<String>>, count: impl Into<mry::ArgMatcher<usize>>) -> mry::MockLocator<(String, usize,), String, String, mry::Behavior2<(String, usize,), String> > {
                    mry::MockLocator::new(
                        self.mry.mocks(),
                        std::any::Any::type_id(&Self::meow),
                        "Cat::meow",
                        (base.into(), count.into(),).into(),
                        std::convert::identity,
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
                #[cfg_attr(debug_assertions, track_caller)]
                fn meow(&self, out: &'static mut String, base: &str, count: &usize) {
                    #[cfg(debug_assertions)]
                    if let Some(out) = self.mry.record_call_and_find_mock_output::<_, ()>(std::any::Any::type_id(&Self::meow), "Cat::meow", (<String>::clone(&out), base.to_string(), <usize>::clone(&count),)) {
                        return out;
                    }
                    (move || {
                        *out = base.repeat(count);
                    })()
                }

                #[cfg(debug_assertions)]
                #[must_use]
                pub fn mock_meow(&mut self, out: impl Into <mry::ArgMatcher<String>>, base: impl Into<mry::ArgMatcher<String>>, count: impl Into<mry::ArgMatcher<usize>>)
                    -> mry::MockLocator<(String, String, usize,), (), (), mry::Behavior3<(String, String, usize,), ()> > {
                    mry::MockLocator::new(
                        self.mry.mocks(),
                        std::any::Any::type_id(&Self::meow),
                        "Cat::meow",
                        (out.into(), base.into(), count.into(),).into(),
                        std::convert::identity,
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
                #[cfg_attr(debug_assertions, track_caller)]
                async fn meow(&self, count: usize) -> String {
                    #[cfg(debug_assertions)]
                    if let Some(out) = self.mry.record_call_and_find_mock_output::<_, String>(std::any::Any::type_id(&Self::meow), "Cat::meow", (<usize>::clone(&count),)) {
                        return out;
                    }
                    (move || async move {
                        base().await.repeat(count);
                    })().await
                }

                #[cfg(debug_assertions)]
                #[must_use]
                pub fn mock_meow(&mut self, count: impl Into<mry::ArgMatcher<usize>>) -> mry::MockLocator<(usize,), String, String, mry::Behavior1<(usize,), String> > {
                    mry::MockLocator::new(
                        self.mry.mocks(),
                        std::any::Any::type_id(&Self::meow),
                        "Cat::meow",
                        (count.into(),).into(),
                        std::convert::identity,
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
                #[cfg_attr(debug_assertions, track_caller)]
                fn meow(&self, arg0: A, count: usize, arg2: String) -> String {
                    #[cfg(debug_assertions)]
                    if let Some(out) = self.mry.record_call_and_find_mock_output::<_, String>(std::any::Any::type_id(&Self::meow), "Cat::meow", (<A>::clone(&arg0), <usize>::clone(&count), <String>::clone(&arg2),)) {
                        return out;
                    }
                    let A { name } = arg0;
                    let _ = arg2;
                    (move || {
                        name.repeat(count)
                    })()
                }

                #[cfg(debug_assertions)]
                #[must_use]
                pub fn mock_meow(&mut self, arg0: impl Into<mry::ArgMatcher<A>>, count: impl Into<mry::ArgMatcher<usize>>, arg2: impl Into<mry::ArgMatcher<String>>) -> mry::MockLocator<(A, usize, String,), String, String, mry::Behavior3<(A, usize, String,), String> > {
                    mry::MockLocator::new(
                        self.mry.mocks(),
                        std::any::Any::type_id(&Self::meow),
                        "Cat::meow",
                        (arg0.into(), count.into(), arg2.into(),).into(),
                        std::convert::identity,
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
                #[cfg_attr(debug_assertions, track_caller)]
                pub fn meow(&self, count: usize) -> String {
                    #[cfg(debug_assertions)]
                    if let Some(out) = self.mry.record_call_and_find_mock_output::<_, String>(std::any::Any::type_id(&Self::meow), "Cat::meow", (<usize>::clone(&count),)) {
                        return out;
                    }
                    (move || {
                        "meow".repeat(count)
                    })()
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
                #[cfg_attr(debug_assertions, track_caller)]
                fn increment(&self, mut count: usize) -> usize {
                    #[cfg(debug_assertions)]
                    if let Some(out) = self.mry.record_call_and_find_mock_output::<_, usize>(std::any::Any::type_id(&Self::increment), "Cat::increment", (<usize>::clone(&count),)) {
                        return out;
                    }
                    (move || {
                        count += 1;
                        count
                    })()
                }

                #[cfg(debug_assertions)]
                #[must_use]
                pub fn mock_increment(&mut self, count: impl Into<mry::ArgMatcher<usize>>) -> mry::MockLocator<(usize,), usize, usize, mry::Behavior1<(usize,), usize> > {
                    mry::MockLocator::new(
                        self.mry.mocks(),
                        std::any::Any::type_id(&Self::increment),
                        "Cat::increment",
                        (count.into(),).into(),
                        std::convert::identity,
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
                #[cfg_attr(debug_assertions, track_caller)]
                fn meow<'a, T: Display, const A: usize>(&self, a: usize) -> &'a String {
                    #[cfg(debug_assertions)]
                    if let Some(out) = self.mry.record_call_and_find_mock_output::<_, &'static String>(std::any::Any::type_id(&Self::meow), "Cat::meow", (<usize>::clone(&a),)) {
                        return out;
                    }
                    (move || {
                        todo!()
                    })()
                }

                #[cfg(debug_assertions)]
                #[must_use]
                pub fn mock_meow(&mut self, a: impl Into<mry::ArgMatcher<usize>>) -> mry::MockLocator<(usize,), &'static String, &'static String, mry::Behavior1<(usize,), &'static String> > {
                    mry::MockLocator::new(
                        self.mry.mocks(),
                        std::any::Any::type_id(&Self::meow),
                        "Cat::meow",
                        (a.into(),).into(),
                        std::convert::identity,
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
                #[cfg_attr(debug_assertions, track_caller)]
                async fn meow(&self, count: usize) -> String {
                    #[cfg(debug_assertions)]
                    if let Some(out) = self.mry.record_call_and_find_mock_output::<_, String>(std::any::Any::type_id(&Self::meow), "Cat::meow", (<usize>::clone(&count),)) {
                        return out;
                    }
                    (move || async move {
                        "meow".repeat(count)
                    })().await
                }

                #[cfg(debug_assertions)]
                #[must_use]
                pub fn mock_meow(&mut self, count: impl Into<mry::ArgMatcher<usize>>) -> mry::MockLocator<(usize,), String, String, mry::Behavior1<(usize,), String> > {
                    mry::MockLocator::new(
                        self.mry.mocks(),
                        std::any::Any::type_id(&Self::meow),
                        "Cat::meow",
                        (count.into(),).into(),
                        std::convert::identity,
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
                #[cfg_attr(debug_assertions, track_caller)]
                fn meow(&self, count: usize) -> impl std::future::Future<Output = String> + Send {
                    #[cfg(debug_assertions)]
                    if let Some(out) = self.mry.record_call_and_find_mock_output::<_, String>(std::any::Any::type_id(&Self::meow), "Cat::meow", (<usize>::clone(&count),)) {
                        return async move { out };
                    }
                    (move || {
                        async move {
                            "meow".repeat(count)
                        }
                    })()
                }

                #[cfg(debug_assertions)]
                #[must_use]
                pub fn mock_meow(&mut self, count: impl Into<mry::ArgMatcher<usize>>) -> mry::MockLocator<(usize,), String, String, mry::Behavior1<(usize,), String> > {
                    mry::MockLocator::new(
                        self.mry.mocks(),
                        std::any::Any::type_id(&Self::meow),
                        "Cat::meow",
                        (count.into(),).into(),
                        std::convert::identity,
                    )
                }
            }.to_string()
        );
    }

    #[test]
    fn input_send_wrapper_raw_pointer() {
        let input: ImplItemFn = parse_quote! {
            fn meow(&self, count: *mut String) -> usize {
                count
            }
        };

        assert_eq!(
            t(&input).to_string(),
            quote! {
                #[cfg_attr(debug_assertions, track_caller)]
                fn meow(&self, count: *mut String) -> usize {
                    #[cfg(debug_assertions)]
                    if let Some(out) = self.mry.record_call_and_find_mock_output::<_, usize>(std::any::Any::type_id(&Self::meow), "Cat::meow", (mry::send_wrapper::SendWrapper::new(<*mut String>::clone(&count)),)) {
                        return out;
                    }
                    (move || {
                        count
                    })()
                }

                #[cfg(debug_assertions)]
                #[must_use]
                pub fn mock_meow(&mut self, count: impl Into<mry::ArgMatcher<mry::send_wrapper::SendWrapper<*mut String> >>) -> mry::MockLocator<(mry::send_wrapper::SendWrapper<*mut String>,), usize, usize, mry::Behavior1<(mry::send_wrapper::SendWrapper<*mut String>,), usize> > {
                    mry::MockLocator::new(
                        self.mry.mocks(),
                        std::any::Any::type_id(&Self::meow),
                        "Cat::meow",
                        (count.into(),).into(),
                        std::convert::identity,
                    )
                }
            }
            .to_string()
        );
    }

    #[test]
    fn return_send_wrapper() {
        let input: ImplItemFn = parse_quote! {
            fn meow(&self, count: usize) -> *mut String {
                Box::into_raw(Box::new("meow".repeat(count)))
            }
        };

        assert_eq!(
            t(&input).to_string(),
            quote! {
                #[cfg_attr(debug_assertions, track_caller)]
                fn meow(&self, count: usize) -> *mut String {
                    #[cfg(debug_assertions)]
                    if let Some(out) = self.mry.record_call_and_find_mock_output::<_, mry::send_wrapper::SendWrapper<*mut String> >(std::any::Any::type_id(&Self::meow), "Cat::meow", (<usize>::clone(&count),)) {
                        return mry::send_wrapper::SendWrapper::take(out);
                    }
                    (move || {
                        Box::into_raw(Box::new("meow".repeat(count)))
                    })()
                }

                #[cfg(debug_assertions)]
                #[must_use]
                pub fn mock_meow(&mut self, count: impl Into<mry::ArgMatcher<usize>>) -> mry::MockLocator<(usize,), mry::send_wrapper::SendWrapper<*mut String>, *mut String, mry::BehaviorSendWrapper1<(usize,), *mut String> > {
                    mry::MockLocator::new(
                        self.mry.mocks(),
                        std::any::Any::type_id(&Self::meow),
                        "Cat::meow",
                        (count.into(),).into(),
                        |out| mry::send_wrapper::SendWrapper::new(out),
                    )
                }
            }
            .to_string()
        );
    }

    #[test]
    fn non_send_output() {
        let attr = parse_quote! {
            mry(non_send(T))
        };
        let input: ImplItemFn = parse_quote! {
            fn meow(&self, count: usize) -> T {
                "meow".repeat(count)
            }
        };

        assert_eq!(
            t_with_attr(attr, &input).to_string(),
            quote! {
                #[cfg_attr(debug_assertions, track_caller)]
                fn meow(&self, count: usize) -> T {
                    #[cfg(debug_assertions)]
                    if let Some(out) = self.mry.record_call_and_find_mock_output::<_, mry::send_wrapper::SendWrapper<T> >(std::any::Any::type_id(&Self::meow), "Cat::meow", (<usize>::clone(&count),)) {
                        return mry::send_wrapper::SendWrapper::take(out);
                    }
                    (move || {
                        "meow".repeat(count)
                    })()
                }

                #[cfg(debug_assertions)]
                #[must_use]
                pub fn mock_meow(&mut self, count: impl Into<mry::ArgMatcher<usize>>) -> mry::MockLocator<(usize,), mry::send_wrapper::SendWrapper<T>, T, mry::BehaviorSendWrapper1<(usize,), T> > {
                    mry::MockLocator::new(
                        self.mry.mocks(),
                        std::any::Any::type_id(&Self::meow),
                        "Cat::meow",
                        (count.into(),).into(),
                        |out| mry::send_wrapper::SendWrapper::new(out),
                    )
                }
            }
            .to_string()
        );
    }

    #[test]
    fn non_send_input() {
        let attr = parse_quote! {
            mry(non_send(T))
        };
        let input: ImplItemFn = parse_quote! {
            fn meow(&self, count: T) -> usize {
                "meow".repeat(count)
            }
        };

        assert_eq!(
            t_with_attr(attr, &input).to_string(),
            quote! {
                #[cfg_attr(debug_assertions, track_caller)]
                fn meow(&self, count: T) -> usize {
                    #[cfg(debug_assertions)]
                    if let Some(out) = self.mry.record_call_and_find_mock_output::<_, usize>(std::any::Any::type_id(&Self::meow), "Cat::meow", (mry::send_wrapper::SendWrapper::new(<T>::clone(&count)),)) {
                        return out;
                    }
                    (move || {
                        "meow".repeat(count)
                    })()
                }

                #[cfg(debug_assertions)]
                #[must_use]
                pub fn mock_meow(&mut self, count: impl Into<mry::ArgMatcher<mry::send_wrapper::SendWrapper<T> >>) -> mry::MockLocator<(mry::send_wrapper::SendWrapper<T>,), usize, usize, mry::Behavior1<(mry::send_wrapper::SendWrapper<T>,), usize> > {
                    mry::MockLocator::new(
                        self.mry.mocks(),
                        std::any::Any::type_id(&Self::meow),
                        "Cat::meow",
                        (count.into(),).into(),
                        std::convert::identity,
                    )
                }
            }
            .to_string()
        );
    }

    #[test]
    fn skip() {
        let attr = parse_quote! {
            mry(skip_args(A, B))
        };
        let input: ImplItemFn = parse_quote! {
            fn meow(&self, a: A, b: B, count: usize) -> String {
                "meow".repeat(count)
            }
        };

        assert_eq!(
            t_with_attr(attr, &input).to_string(),
            quote! {
                #[cfg_attr(debug_assertions, track_caller)]
                fn meow(&self, a: A, b: B, count: usize) -> String {
                    #[cfg(debug_assertions)]
                    if let Some(out) = self.mry.record_call_and_find_mock_output::<_, String>(std::any::Any::type_id(&Self::meow), "Cat::meow", (<usize>::clone(&count),)) {
                        return out;
                    }
                    (move || {
                        "meow".repeat(count)
                    })()
                }

                #[cfg(debug_assertions)]
                #[must_use]
                pub fn mock_meow(&mut self, count: impl Into<mry::ArgMatcher<usize>>) -> mry::MockLocator<(usize,), String, String, mry::Behavior1<(usize,), String> > {
                    mry::MockLocator::new(
                        self.mry.mocks(),
                        std::any::Any::type_id(&Self::meow),
                        "Cat::meow",
                        (count.into(),).into(),
                        std::convert::identity,
                    )
                }
            }
            .to_string()
        );
    }

    #[test]
    fn skip_return_type_no_effect() {
        let attr = parse_quote! {
            mry(skip_args(T))
        };

        let input: ImplItemFn = parse_quote! {
            fn meow(&self, a: A, b: B, count: usize) -> T {
                "meow".repeat(count).into()
            }
        };

        assert_eq!(
            t_with_attr(attr, &input).to_string(),
            quote! {
                #[cfg_attr(debug_assertions, track_caller)]
                fn meow(&self, a: A, b: B, count: usize) -> T {
                    #[cfg(debug_assertions)]
                    if let Some(out) = self.mry.record_call_and_find_mock_output::<_, T>(std::any::Any::type_id(&Self::meow), "Cat::meow", (<A>::clone(&a), <B>::clone(&b), <usize>::clone(&count),)) {
                        return out;
                    }
                    (move || {
                        "meow".repeat(count).into()
                    })()
                }

                #[cfg(debug_assertions)]
                #[must_use]
                pub fn mock_meow(&mut self, a: impl Into<mry::ArgMatcher<A>>, b: impl Into<mry::ArgMatcher<B>>, count: impl Into<mry::ArgMatcher<usize>>) -> mry::MockLocator<(A, B, usize,), T, T, mry::Behavior3<(A, B, usize,), T> > {
                    mry::MockLocator::new(
                        self.mry.mocks(),
                        std::any::Any::type_id(&Self::meow),
                        "Cat::meow",
                        (a.into(), b.into(), count.into(),).into(),
                        std::convert::identity,
                    )
                }
            }
            .to_string()
        );
    }

    #[test]
    fn does_not_add_track_caller_when_already_present() {
        let input: ImplItemFn = parse2(quote! {
            #[track_caller]
            fn meow(&self, count: usize) -> String {
                "meow".repeat(count)
            }
        })
        .unwrap();

        assert_eq!(
            t(&input).to_string(),
            quote! {
                #[track_caller]
                fn meow(&self, count: usize) -> String {
                    #[cfg(debug_assertions)]
                    if let Some(out) = self.mry.record_call_and_find_mock_output::<_, String>(std::any::Any::type_id(&Self::meow), "Cat::meow", (<usize>::clone(&count),)) {
                        return out;
                    }
                    "meow".repeat(count)
                }

                #[cfg(debug_assertions)]
                #[must_use]
                pub fn mock_meow(&mut self, count: impl Into<mry::ArgMatcher<usize>>) -> mry::MockLocator<(usize,), String, String, mry::Behavior1<(usize,), String> > {
                    mry::MockLocator::new(
                        self.mry.mocks(),
                        std::any::Any::type_id(&Self::meow),
                        "Cat::meow",
                        (count.into(),).into(),
                        std::convert::identity,
                    )
                }
            }
            .to_string()
        );
    }

    #[test]
    fn does_not_add_track_caller_when_cfg_attr_track_caller_present() {
        let input: ImplItemFn = parse2(quote! {
            #[cfg_attr(debug_assertions, track_caller)]
            fn meow(&self, count: usize) -> String {
                "meow".repeat(count)
            }
        })
        .unwrap();

        assert_eq!(
            t(&input).to_string(),
            quote! {
                #[cfg_attr(debug_assertions, track_caller)]
                fn meow(&self, count: usize) -> String {
                    #[cfg(debug_assertions)]
                    if let Some(out) = self.mry.record_call_and_find_mock_output::<_, String>(std::any::Any::type_id(&Self::meow), "Cat::meow", (<usize>::clone(&count),)) {
                        return out;
                    }
                    "meow".repeat(count)
                }

                #[cfg(debug_assertions)]
                #[must_use]
                pub fn mock_meow(&mut self, count: impl Into<mry::ArgMatcher<usize>>) -> mry::MockLocator<(usize,), String, String, mry::Behavior1<(usize,), String> > {
                    mry::MockLocator::new(
                        self.mry.mocks(),
                        std::any::Any::type_id(&Self::meow),
                        "Cat::meow",
                        (count.into(),).into(),
                        std::convert::identity,
                    )
                }
            }
            .to_string()
        );
    }
}
