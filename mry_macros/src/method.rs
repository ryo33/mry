use proc_macro2::{Span, TokenStream};
use quote::quote;
use syn::{Attribute, FnArg, Ident, Pat, PatIdent, ReturnType, Signature, Type, Visibility};

pub fn transform(
    struct_name: &str,
    tokens: TokenStream,
    vis: Option<&Visibility>,
    attrs: &Vec<Attribute>,
    sig: &Signature,
    body: &TokenStream,
) -> (TokenStream, TokenStream) {
    // Split into receiver and other inputs
    let receiver;
    let mut inputs = sig.inputs.iter();
    if let Some(FnArg::Receiver(rcv)) = inputs.next() {
        receiver = rcv;
    } else {
        return (tokens.clone(), TokenStream::default());
    }
    let inputs: Vec<_> = inputs
        .map(|input| {
            if let FnArg::Typed(typed_arg) = input {
                typed_arg.clone()
            } else {
                panic!("multiple receiver?");
            }
        })
        .collect();
    let mut bindings = Vec::new();

    let args: Vec<_> = inputs
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
    let derefed_input_type_tuple: Vec<_> =
        args.iter().map(|input| deref_type(&*input.ty)).collect();
    let cloned_input: Vec<_> = args
        .iter()
        .map(|input| {
            let pat = &input.pat;
            if is_str(&input.ty) {
                return quote!(#pat.to_string());
            }
            quote!(#pat.clone())
        })
        .collect();
    let output_type = match &sig.output {
        ReturnType::Default => quote!(()),
        ReturnType::Type(_, ty) => quote!(#ty),
    };
    let generics = &sig.generics;
    let attrs = attrs.clone();
    let ident = sig.ident.clone();
    let mock_ident = Ident::new(&format!("mock_{}", ident), Span::call_site());
    let asyn = &sig.asyncness;
    let vis = &vis;
    let name = format!("{}::{}", struct_name, ident.to_string());
    let args = quote!(#(#args),*);
    let input_type_tuple = quote!((#(#derefed_input_type_tuple),*));
    let cloned_input_tuple = quote!((#(#cloned_input),*));
    let bindings = bindings.iter().map(|(pat, arg)| quote![let #pat = #arg;]);
    let behavior_name = Ident::new(&format!("Behavior{}", inputs.len()), Span::call_site());
    let behavior_type = quote![mry::#behavior_name<#input_type_tuple, #output_type>];
    let (mock_args, mock_args_into): (Vec<_>, Vec<_>) = inputs
        .iter()
        .enumerate()
        .map(|(index, input)| {
            let name = Ident::new(&format!("arg{}", index), Span::call_site());
            let ty = deref_type(&*input.ty);
            let mock_arg = quote![#name: impl Into<mry::Matcher<#ty>>];
            let mock_arg_into = quote![#name.into()];
            (mock_arg, mock_arg_into)
        })
        .unzip();
    (
        quote! {
            #(#attrs)*
            #vis #asyn fn #ident #generics(#receiver, #args) -> #output_type {
                #[cfg(test)]
                if let Some(out) = self.mry.record_call_and_find_mock_output(#name, #cloned_input_tuple) {
                    return out;
                }
                #(#bindings)*
                #body
            }
        },
        quote! {
            #[cfg(test)]
            pub fn #mock_ident<'mry>(&'mry mut self, #(#mock_args),*) -> mry::MockLocator<impl std::ops::DerefMut<Target=mry::Mocks> + 'mry, #input_type_tuple, #output_type, #behavior_type> {
                mry::MockLocator {
                    mocks: self.mry.mocks_write(),
                    name: #name,
                    matcher: Some((#(#mock_args_into,)*).into()),
                    _phantom: Default::default(),
                }
            }
        },
    )
}

pub fn deref_type(ty: &Type) -> TokenStream {
    if is_str(&ty) {
        return quote!(String);
    }
    match &ty {
        Type::Reference(ty) => {
            let ty = &ty.elem;
            quote!(#ty)
        }
        ty => quote!(#ty),
    }
}

pub fn is_str(ty: &Type) -> bool {
    match ty {
        Type::Reference(ty) => {
            if let Type::Path(path) = &*ty.elem {
                if let Some(ident) = path.path.get_ident() {
                    return ident.to_string() == "str";
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
    use syn::{parse2, ImplItemMethod};

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

    fn t(name: &str, method: &ImplItemMethod) -> (TokenStream, TokenStream) {
        transform(
            name,
            method.to_token_stream(),
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

    #[test]
    fn support_associated_functions() {
        let input: ImplItemMethod = parse2(quote! {
            fn meow() -> String{
                "meow"
            }
        })
        .unwrap();

        assert_eq!(
            t("Cat", &input).to_string(),
            quote! {
                fn meow() -> String {
                    "meow"
                }
            }
            .to_string()
        );
    }

    #[test]
    fn adds_mock_function() {
        let input: ImplItemMethod = parse2(quote! {
            fn meow(&self, count: usize) -> String {
                "meow".repeat(count)
            }
        })
        .unwrap();

        assert_eq!(
            t("Cat", &input).to_string(),
            quote! {
                fn meow(&self, count: usize) -> String {
                    #[cfg(test)]
                    if let Some(out) = self.mry.record_call_and_find_mock_output("Cat::meow", (count.clone())) {
                        return out;
                    }
                    "meow".repeat(count)
                }

                #[cfg(test)]
                pub fn mock_meow<'mry>(&'mry mut self, arg0: impl Into<mry::Matcher<usize>>) -> mry::MockLocator<impl std::ops::DerefMut<Target = mry::Mocks> + 'mry, (usize), String, mry::Behavior1<(usize), String> > {
                    mry::MockLocator {
                        mocks: self.mry.mocks_write(),
                        name: "Cat::meow",
                        matcher: Some((arg0.into(),).into()),
                        _phantom: Default::default(),
                    }
                }
            }
            .to_string()
        );
    }

    #[test]
    fn empty_args() {
        let input: ImplItemMethod = parse2(quote! {
            fn meow(&self) -> String {
                "meow".into()
            }
        })
        .unwrap();

        assert_eq!(
            t("Cat", &input).to_string(),
            quote! {
                fn meow(&self, ) -> String {
                    #[cfg(test)]
                    if let Some(out) = self.mry.record_call_and_find_mock_output("Cat::meow", ()) {
                        return out;
                    }
                    "meow".into()
                }

                #[cfg(test)]
                pub fn mock_meow<'mry>(&'mry mut self, ) -> mry::MockLocator<impl std::ops::DerefMut<Target = mry::Mocks> + 'mry, (), String, mry::Behavior0<(), String> > {
                    mry::MockLocator {
                        mocks: self.mry.mocks_write(),
                        name: "Cat::meow",
                        matcher: Some(().into()),
                        _phantom: Default::default(),
                    }
                }
            }
            .to_string()
        );
    }

    #[test]
    fn multiple_args() {
        let input: ImplItemMethod = parse2(quote! {
            fn meow(&self, base: String, count: usize) -> String {
                base.repeat(count)
            }
        })
        .unwrap();

        assert_eq!(
            t("Cat", &input).to_string(),
            quote! {
                fn meow(&self, base: String, count: usize) -> String {
                    #[cfg(test)]
                    if let Some(out) = self.mry.record_call_and_find_mock_output("Cat::meow", (base.clone(), count.clone())) {
                        return out;
                    }
                    base.repeat(count)
                }

                #[cfg(test)]
                pub fn mock_meow<'mry>(&'mry mut self, arg0: impl Into<mry::Matcher<String>>, arg1: impl Into<mry::Matcher<usize>>) -> mry::MockLocator<impl std::ops::DerefMut<Target = mry::Mocks> + 'mry, (String, usize), String, mry::Behavior2<(String, usize), String> > {
                    mry::MockLocator {
                        mocks: self.mry.mocks_write(),
                        name: "Cat::meow",
                        matcher: Some((arg0.into(), arg1.into(),).into()),
                        _phantom: Default::default(),
                    }
                }
            }
            .to_string()
        );
    }

    #[test]
    fn input_reference_and_str() {
        let input: ImplItemMethod = parse2(quote! {
            fn meow(&self, out: &'static mut String, base: &str, count: &usize) {
                *out = base.repeat(count);
            }
        })
        .unwrap();

        assert_eq!(
            t("Cat", &input).to_string(),
            quote! {
                fn meow(&self, out: &'static mut String, base: &str, count: &usize) -> () {
                    #[cfg(test)]
                    if let Some(out) = self.mry.record_call_and_find_mock_output("Cat::meow", (out.clone(), base.to_string(), count.clone())) {
                        return out;
                    }
                    *out = base.repeat(count);
                }

                #[cfg(test)]
                pub fn mock_meow<'mry>(&'mry mut self, arg0: impl Into <mry::Matcher<String>>, arg1: impl Into<mry::Matcher<String>>, arg2: impl Into<mry::Matcher<usize>>)
                    -> mry::MockLocator<impl std::ops::DerefMut<Target = mry::Mocks> + 'mry, (String, String, usize), (), mry::Behavior3<(String, String, usize), ()> > {
                    mry::MockLocator {
                        mocks: self.mry.mocks_write(),
                        name: "Cat::meow",
                        matcher: Some((arg0.into(), arg1.into(), arg2.into(),).into()),
                        _phantom: Default::default(),
                    }
                }
            }
            .to_string()
        );
    }

    #[test]
    fn supports_async() {
        let input: ImplItemMethod = parse2(quote! {
            async fn meow(&self, count: usize) -> String{
                base().await.repeat(count);
            }
        })
        .unwrap();

        assert_eq!(
            t("Cat", &input).to_string(),
            quote! {
                async fn meow(&self, count: usize) -> String {
                    #[cfg(test)]
                    if let Some(out) = self.mry.record_call_and_find_mock_output("Cat::meow", (count.clone())) {
                        return out;
                    }
                    base().await.repeat(count);
                }

                #[cfg(test)]
                pub fn mock_meow<'mry>(&'mry mut self, arg0: impl Into<mry::Matcher<usize>>) -> mry::MockLocator<impl std::ops::DerefMut<Target = mry::Mocks> + 'mry, (usize), String, mry::Behavior1<(usize), String> > {
                    mry::MockLocator {
                        mocks: self.mry.mocks_write(),
                        name: "Cat::meow",
                        matcher: Some((arg0.into(),).into()),
                        _phantom: Default::default(),
                    }
                }
            }
            .to_string()
        );
    }

    #[test]
    fn support_pattern() {
        let input: ImplItemMethod = parse2(quote! {
            fn meow(&self, A { name }: A, count: usize, _: String) -> String {
                name.repeat(count)
            }
        })
        .unwrap();

        assert_eq!(
            t("Cat", &input).to_string(),
            quote! {
				fn meow(&self, arg0: A, count: usize, arg2: String) -> String {
                    #[cfg(test)]
                    if let Some(out) = self.mry.record_call_and_find_mock_output("Cat::meow", (arg0.clone(), count.clone(), arg2.clone())) {
                        return out;
                    }
					let A { name } = arg0;
					let _ = arg2;
                    name.repeat(count)
                }

                #[cfg(test)]
                pub fn mock_meow<'mry>(&'mry mut self, arg0: impl Into<mry::Matcher<A>>, arg1: impl Into<mry::Matcher<usize>>, arg2: impl Into<mry::Matcher<String>>) -> mry::MockLocator<impl std::ops::DerefMut<Target = mry::Mocks> + 'mry, (A, usize, String), String, mry::Behavior3<(A, usize, String), String> > {
                    mry::MockLocator {
                        mocks: self.mry.mocks_write(),
                        name: "Cat::meow",
                        matcher: Some((arg0.into(), arg1.into(), arg2.into(),).into()),
                        _phantom: Default::default(),
                    }
                }
            }
            .to_string()
        );
    }

    #[test]
    fn respect_visibility() {
        let input: ImplItemMethod = parse2(quote! {
            pub fn meow(&self, count: usize) -> String {
                "meow".repeat(count)
            }
        })
        .unwrap();

        assert_eq!(
            t("Cat", &input).0.to_string(),
            quote! {
                pub fn meow(&self, count: usize) -> String {
                    #[cfg(test)]
                    if let Some(out) = self.mry.record_call_and_find_mock_output("Cat::meow", (count.clone())) {
                        return out;
                    }
                    "meow".repeat(count)
                }
            }
            .to_string()
        );
    }
}
