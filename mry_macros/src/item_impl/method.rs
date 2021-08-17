use proc_macro2::{Span, TokenStream};
use quote::{quote, ToTokens};
use syn::{
    Attribute, Block, FnArg, Ident, ImplItemMethod, Pat, PatIdent, ReturnType, Signature, Type,
    Visibility,
};

use crate::method;

pub fn transform(struct_name: &str, method: &ImplItemMethod) -> (TokenStream, TokenStream) {
    method::transform(
        struct_name,
        method.to_token_stream(),
        Some(&method.vis),
        &method.attrs,
        &method.sig,
        &method.block.to_token_stream(),
    )
}

#[cfg(test)]
mod test {
    use super::*;
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

    #[test]
    fn support_associated_functions() {
        let input: ImplItemMethod = parse2(quote! {
            fn meow() -> String{
                "meow"
            }
        })
        .unwrap();

        assert_eq!(
            transform("Cat", &input).to_string(),
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
            transform("Cat", &input).to_string(),
            quote! {
                fn meow(&self, count: usize) -> String {
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
            transform("Cat", &input).to_string(),
            quote! {
                fn meow(&self, ) -> String {
                    #[cfg(test)]
                    if self.mry.is_some() {
                        return mry::MOCK_DATA
                            .lock()
                            .get_mut_or_create::<(), String>(&self.mry, "Cat::meow")
                            ._inner_called(&());
                    }
                    {
                        "meow".into()
                    }
                }

                #[cfg(test)]
                pub fn mock_meow<'mry>(&'mry mut self) -> mry::MockLocator<'mry, (), String, mry::Behavior0<(), String> > {
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
            transform("Cat", &input).to_string(),
            quote! {
                fn meow(&self, base: String, count: usize) -> String {
                    #[cfg(test)]
                    if self.mry.is_some() {
                        return mry::MOCK_DATA
                            .lock()
                            .get_mut_or_create::<(String, usize), String>(&self.mry, "Cat::meow")
                            ._inner_called(&(base, count));
                    }
                    {
                        base.repeat(count)
                    }
                }

                #[cfg(test)]
                pub fn mock_meow<'mry>(&'mry mut self) -> mry::MockLocator<'mry, (String, usize), String, mry::Behavior2<(String, usize), String> > {
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
            transform("Cat", &input).to_string(),
            quote! {
                fn meow(&self, out: &'static mut String, base: &str, count: &usize) -> () {
                    #[cfg(test)]
                    if self.mry.is_some() {
                        return mry::MOCK_DATA
                            .lock()
                            .get_mut_or_create::<(String, String, usize), ()>(&self.mry, "Cat::meow")
                            ._inner_called(&(*out, base.to_string(), *count));
                    }
                    {
                        *out = base.repeat(count);
                    }
                }

                #[cfg(test)]
                pub fn mock_meow<'mry>(&'mry mut self) -> mry::MockLocator<'mry, (String, String, usize), (), mry::Behavior3<(String, String, usize), ()> > {
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
            transform("Cat", &input).to_string(),
            quote! {
                async fn meow(&self, count: usize) -> String{
                    #[cfg(test)]
                    if self.mry.is_some() {
                        return mry::MOCK_DATA
                            .lock()
                            .get_mut_or_create::<(usize), String>(&self.mry, "Cat::meow")
                            ._inner_called(&(count));
                    }
                    {
                        base().await.repeat(count);
                    }
                }

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
            transform("Cat", &input).to_string(),
            quote! {
				fn meow(&self, arg0: A, count: usize, arg2: String) -> String {
                    #[cfg(test)]
                    if self.mry.is_some() {
                        return mry::MOCK_DATA
                            .lock()
                            .get_mut_or_create::<(A, usize, String), String>(&self.mry, "Cat::meow")
                            ._inner_called(&(arg0, count, arg2));
                    }
					let A { name } = arg0;
					let _ = arg2;
                    {
						name.repeat(count)
                    }
                }

                #[cfg(test)]
                pub fn mock_meow<'mry>(&'mry mut self) -> mry::MockLocator<'mry, (A, usize, String), String, mry::Behavior3<(A, usize, String), String> > {
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
            transform("Cat", &input).to_string(),
            quote! {
                pub fn meow(&self, count: usize) -> String {
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
            .to_string()
        );
    }
}
