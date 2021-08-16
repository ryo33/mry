use proc_macro2::{Span, TokenStream};
use quote::{quote, ToTokens};
use syn::{FnArg, Ident, ImplItemMethod, ReturnType, Type};

pub fn transform(method: &ImplItemMethod) -> TokenStream {
    if !matches!(method.sig.inputs.iter().next(), Some(FnArg::Receiver(_))) {
        return method.to_token_stream();
    }
    let body = &method.block;
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
                let ty = match &*typed_arg.ty {
                    Type::Reference(ty) => {
                        let ty = &ty.elem;
                        quote!(#ty)
                    }
                    ty => quote!(#ty),
                };
                Some(ty)
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
                let input = match &*typed_arg.ty {
                    Type::Reference(_ty) => {
                        quote!(#pat.clone())
                    }
                    _ => quote!(#pat),
                };
                Some(input)
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
    let input_tuple = quote!((#(#input_tuple),*));
    let behavior_name = Ident::new(&format!("Behavior{}", inputs.len() - 1), Span::call_site());
    let behavior_type = quote! {
        mry::#behavior_name<#input_type_tuple, #output_type>
    };
    let asyn = &method.sig.asyncness;
    quote! {
        #(#attrs)*
        #asyn fn #ident(#inputs) -> #output_type {
            #[cfg(test)]
            if self.mry.is_some() {
                return mry::MOCK_DATA
                    .lock()
                    .get_mut_or_create::<#input_type_tuple, #output_type>(&self.mry, #name)
                    ._inner_called(&#input_tuple);
            }
            #body
        }

        #[cfg(test)]
        fn #mock_ident<'a>(&'a mut self) -> mry::MockLocator<'a, #input_type_tuple, #output_type, #behavior_type> {
            if self.mry.is_none() {
                self.mry = mry::Mry::generate();
            }
            mry::MockLocator {
                id: &self.mry,
                name: #name,
                _phantom: Default::default(),
            }
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use similar_asserts::assert_eq;
    use syn::{parse2, ImplItemMethod};

    #[test]
    fn support_associated_functions() {
        let input: ImplItemMethod = parse2(quote! {
            fn meow() -> String{
                "meow"
            }
        })
        .unwrap();

        assert_eq!(
            transform(&input).to_string(),
            quote! {
                fn meow() -> String{
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
            transform(&input).to_string(),
            quote! {
                fn meow(&self, count: usize) -> String {
                    #[cfg(test)]
                    if self.mry.is_some() {
                        return mry::MOCK_DATA
                            .lock()
                            .get_mut_or_create::<(usize), String>(&self.mry, "meow")
                            ._inner_called(&(count));
                    }
                    {
                        "meow".repeat(count)
                    }
                }

                #[cfg(test)]
                fn mock_meow<'a>(&'a mut self) -> mry::MockLocator<'a, (usize), String, mry::Behavior1<(usize), String> > {
                    if self.mry.is_none() {
                        self.mry = mry::Mry::generate();
                    }
                    mry::MockLocator {
                        id: &self.mry,
                        name: "meow",
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
            transform(&input).to_string(),
            quote! {
                fn meow(&self) -> String {
                    #[cfg(test)]
                    if self.mry.is_some() {
                        return mry::MOCK_DATA
                            .lock()
                            .get_mut_or_create::<(), String>(&self.mry, "meow")
                            ._inner_called(&());
                    }
                    {
                        "meow".into()
                    }
                }

                #[cfg(test)]
                fn mock_meow<'a>(&'a mut self) -> mry::MockLocator<'a, (), String, mry::Behavior0<(), String> > {
                    if self.mry.is_none() {
                        self.mry = mry::Mry::generate();
                    }
                    mry::MockLocator {
                        id: &self.mry,
                        name: "meow",
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
            transform(&input).to_string(),
            quote! {
                fn meow(&self, base: String, count: usize) -> String {
                    #[cfg(test)]
                    if self.mry.is_some() {
                        return mry::MOCK_DATA
                            .lock()
                            .get_mut_or_create::<(String, usize), String>(&self.mry, "meow")
                            ._inner_called(&(base, count));
                    }
                    {
                        base.repeat(count)
                    }
                }

                #[cfg(test)]
                fn mock_meow<'a>(&'a mut self) -> mry::MockLocator<'a, (String, usize), String, mry::Behavior2<(String, usize), String> > {
                    if self.mry.is_none() {
                        self.mry = mry::Mry::generate();
                    }
                    mry::MockLocator {
                        id: &self.mry,
                        name: "meow",
                        _phantom: Default::default(),
                    }
                }
            }
            .to_string()
        );
    }

    #[test]
    fn input_reference() {
        let input: ImplItemMethod = parse2(quote! {
            fn meow(&self, base: &mut String, count: &usize) {
                *base = base.repeat(count);
            }
        })
        .unwrap();

        assert_eq!(
            transform(&input).to_string(),
            quote! {
                fn meow(&self, base: &mut String, count: &usize) -> () {
                    #[cfg(test)]
                    if self.mry.is_some() {
                        return mry::MOCK_DATA
                            .lock()
                            .get_mut_or_create::<(String, usize), ()>(&self.mry, "meow")
                            ._inner_called(&(base.clone(), count.clone()));
                    }
                    {
                        *base = base.repeat(count);
                    }
                }

                #[cfg(test)]
                fn mock_meow<'a>(&'a mut self) -> mry::MockLocator<'a, (String, usize), (), mry::Behavior2<(String, usize), ()> > {
                    if self.mry.is_none() {
                        self.mry = mry::Mry::generate();
                    }
                    mry::MockLocator {
                        id: &self.mry,
                        name: "meow",
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
            transform(&input).to_string(),
            quote! {
                async fn meow(&self, count: usize) -> String{
                    #[cfg(test)]
                    if self.mry.is_some() {
                        return mry::MOCK_DATA
                            .lock()
                            .get_mut_or_create::<(usize), String>(&self.mry, "meow")
                            ._inner_called(&(count));
                    }
                    {
                        base().await.repeat(count);
                    }
                }

                #[cfg(test)]
                fn mock_meow<'a>(&'a mut self) -> mry::MockLocator<'a, (usize), String, mry::Behavior1<(usize), String> > {
                    if self.mry.is_none() {
                        self.mry = mry::Mry::generate();
                    }
                    mry::MockLocator {
                        id: &self.mry,
                        name: "meow",
                        _phantom: Default::default(),
                    }
                }
            }
            .to_string()
        );
    }
}
