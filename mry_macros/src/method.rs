use proc_macro2::{Span, TokenStream};
use quote::quote;
use syn::{Attribute, FnArg, Ident, Pat, PatIdent, ReturnType, Signature, Type, Visibility};

pub fn transform(
    struct_name: &str,
    tokens: TokenStream,
    vis: Option<&Visibility>,
    attrs: &Vec<Attribute>,
    sig: &Signature,
    block: &TokenStream,
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

    let generics = &sig.generics;
    let body = &block;
    let attrs = attrs.clone();
    let ident = sig.ident.clone();
    let mock_ident = Ident::new(&format!("mock_{}", ident), Span::call_site());
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
    let derefed_input_type_tuple: Vec<_> = args
        .iter()
        .map(|input| {
            if is_str(&input.ty) {
                return quote!(String);
            }
            let ty = match &*input.ty {
                Type::Reference(ty) => {
                    let ty = &ty.elem;
                    quote!(#ty)
                }
                ty => quote!(#ty),
            };
            ty
        })
        .collect();
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
    let asyn = &sig.asyncness;
    let vis = &vis;
    let name = format!("{}::{}", struct_name, ident.to_string());
    let args = quote!(#(#args),*);
    let input_type_tuple = quote!((#(#derefed_input_type_tuple),*));
    let cloned_input_tuple = quote!((#(#cloned_input),*));
    let bindings = bindings.iter().map(|(pat, arg)| quote![let #pat = #arg;]);
    let behavior_name = Ident::new(&format!("Behavior{}", inputs.len()), Span::call_site());
    let behavior_type = quote! {
        mry::#behavior_name<#input_type_tuple, #output_type>
    };
    (
        quote! {
            #(#attrs)*
            #vis #asyn fn #ident #generics(#receiver, #args) -> #output_type {
                #[cfg(test)]
                if self.mry.id().is_some() {
                    if let Some(out) = mry::MOCK_DATA
                        .write()
                        .get_mut_or_create::<#input_type_tuple, #output_type>(&self.mry, #name)
                        ._inner_called(#cloned_input_tuple) {
                        return out;
                    }
                }
                #(#bindings)*
                #body
            }
        },
        quote! {
            #[cfg(test)]
            pub fn #mock_ident<'mry>(&'mry mut self) -> mry::MockLocator<'mry, #input_type_tuple, #output_type, #behavior_type> {
                if self.mry.id().is_none() {
                    self.mry = mry::Mry::generate();
                }
                mry::MockLocator {
                    id: &self.mry,
                    name: #name,
                    _phantom: Default::default(),
                }
            }
        },
    )
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
    use quote::{quote, ToTokens};
    use similar_asserts::assert_eq;
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

    fn t(name: &str, method: &ImplItemMethod) -> String {
        transform(
            name,
            method.to_token_stream(),
            Some(&method.vis),
            &method.attrs,
            &method.sig,
            &method.block.to_token_stream(),
        )
        .to_string()
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
            t("Cat", &input),
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
            t("Cat", &input),
            quote! {
                fn meow(&self, count: usize) -> String {
                    #[cfg(test)]
                    if self.mry.id().is_some() {
                        if let Some(out) = mry::MOCK_DATA
                            .write()
                            .get_mut_or_create::<(usize), String>(&self.mry, "Cat::meow")
                            ._inner_called((count.clone())) {
                            return out;
                        }
                    }
                    {
                        "meow".repeat(count)
                    }
                }

                #[cfg(test)]
                pub fn mock_meow<'mry>(&'mry mut self) -> mry::MockLocator<'mry, (usize), String, mry::Behavior1<(usize), String> > {
                    if self.mry.id().is_none() {
                        self.mry = mry::Mry::generate();
                    }
                    mry::MockLocator {
                        id: &self.mry,
                        name: "Cat::meow",
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
            t("Cat", &input),
            quote! {
                fn meow(&self, ) -> String {
                    #[cfg(test)]
                    if self.mry.id().is_some() {
                        if let Some(out) = mry::MOCK_DATA
                            .write()
                            .get_mut_or_create::<(), String>(&self.mry, "Cat::meow")
                            ._inner_called(()) {
                            return out;
                        }
                    }
                    {
                        "meow".into()
                    }
                }

                #[cfg(test)]
                pub fn mock_meow<'mry>(&'mry mut self) -> mry::MockLocator<'mry, (), String, mry::Behavior0<(), String> > {
                    if self.mry.id().is_none() {
                        self.mry = mry::Mry::generate();
                    }
                    mry::MockLocator {
                        id: &self.mry,
                        name: "Cat::meow",
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
            t("Cat", &input),
            quote! {
                fn meow(&self, base: String, count: usize) -> String {
                    #[cfg(test)]
                    if self.mry.id().is_some() {
                        if let Some(out) = mry::MOCK_DATA
                            .write()
                            .get_mut_or_create::<(String, usize), String>(&self.mry, "Cat::meow")
                            ._inner_called((base.clone(), count.clone())) {
                            return out;
                        }
                    }
                    {
                        base.repeat(count)
                    }
                }

                #[cfg(test)]
                pub fn mock_meow<'mry>(&'mry mut self) -> mry::MockLocator<'mry, (String, usize), String, mry::Behavior2<(String, usize), String> > {
                    if self.mry.id().is_none() {
                        self.mry = mry::Mry::generate();
                    }
                    mry::MockLocator {
                        id: &self.mry,
                        name: "Cat::meow",
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
            t("Cat", &input),
            quote! {
                fn meow(&self, out: &'static mut String, base: &str, count: &usize) -> () {
                    #[cfg(test)]
                    if self.mry.id().is_some() {
                        if let Some(out) = mry::MOCK_DATA
                            .write()
                            .get_mut_or_create::<(String, String, usize), ()>(&self.mry, "Cat::meow")
                            ._inner_called((out.clone(), base.to_string(), count.clone())) {
                            return out;
                        }
                    }
                    {
                        *out = base.repeat(count);
                    }
                }

                #[cfg(test)]
                pub fn mock_meow<'mry>(&'mry mut self) -> mry::MockLocator<'mry, (String, String, usize), (), mry::Behavior3<(String, String, usize), ()> > {
                    if self.mry.id().is_none() {
                        self.mry = mry::Mry::generate();
                    }
                    mry::MockLocator {
                        id: &self.mry,
                        name: "Cat::meow",
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
            t("Cat", &input),
            quote! {
                async fn meow(&self, count: usize) -> String{
                    #[cfg(test)]
                    if self.mry.id().is_some() {
                        if let Some(out) = mry::MOCK_DATA
                            .write()
                            .get_mut_or_create::<(usize), String>(&self.mry, "Cat::meow")
                            ._inner_called((count.clone())) {
                            return out;
                        }
                    }
                    {
                        base().await.repeat(count);
                    }
                }

                #[cfg(test)]
                pub fn mock_meow<'mry>(&'mry mut self) -> mry::MockLocator<'mry, (usize), String, mry::Behavior1<(usize), String> > {
                    if self.mry.id().is_none() {
                        self.mry = mry::Mry::generate();
                    }
                    mry::MockLocator {
                        id: &self.mry,
                        name: "Cat::meow",
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
            t("Cat", &input),
            quote! {
				fn meow(&self, arg0: A, count: usize, arg2: String) -> String {
                    #[cfg(test)]
                    if self.mry.id().is_some() {
                        if let Some(out) = mry::MOCK_DATA
                            .write()
                            .get_mut_or_create::<(A, usize, String), String>(&self.mry, "Cat::meow")
                            ._inner_called((arg0.clone(), count.clone(), arg2.clone())) {
                            return out;
                        }
                    }
					let A { name } = arg0;
					let _ = arg2;
                    {
						name.repeat(count)
                    }
                }

                #[cfg(test)]
                pub fn mock_meow<'mry>(&'mry mut self) -> mry::MockLocator<'mry, (A, usize, String), String, mry::Behavior3<(A, usize, String), String> > {
                    if self.mry.id().is_none() {
                        self.mry = mry::Mry::generate();
                    }
                    mry::MockLocator {
                        id: &self.mry,
                        name: "Cat::meow",
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
            t("Cat", &input),
            quote! {
                pub fn meow(&self, count: usize) -> String {
                    #[cfg(test)]
                    if self.mry.id().is_some() {
                        if let Some(out) = mry::MOCK_DATA
                            .write()
                            .get_mut_or_create::<(usize), String>(&self.mry, "Cat::meow")
                            ._inner_called((count.clone())) {
                            return out;
                        }
                    }
                    {
                        "meow".repeat(count)
                    }
                }

                #[cfg(test)]
                pub fn mock_meow<'mry>(&'mry mut self) -> mry::MockLocator<'mry, (usize), String, mry::Behavior1<(usize), String> > {
                    if self.mry.id().is_none() {
                        self.mry = mry::Mry::generate();
                    }
                    mry::MockLocator {
                        id: &self.mry,
                        name: "Cat::meow",
                        _phantom: Default::default(),
                    }
                }
            }
            .to_string()
        );
    }
}
