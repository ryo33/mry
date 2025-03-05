use proc_macro2::TokenStream;
use quote::{quote, ToTokens};
use syn::{parse_quote, ItemFn};

pub struct LockPaths(Vec<syn::Type>);

impl syn::parse::Parse for LockPaths {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        Ok(Self(
            input
                .parse_terminated(syn::Type::parse, syn::Token![,])?
                .into_iter()
                .collect(),
        ))
    }
}

pub(crate) fn transform(args: LockPaths, mut input: ItemFn) -> TokenStream {
    if let Some((attr, paths)) = input
        .attrs
        .iter_mut()
        .find(|attr| {
            let path = attr.path();
            if path.segments.len() == 1 && path.segments[0].ident == "lock" {
                return true;
            }
            if path.segments.len() == 2
                && path.segments[0].ident == "mry"
                && path.segments[1].ident == "lock"
            {
                return true;
            }
            false
        })
        .and_then(|attr| {
            attr.parse_args::<LockPaths>()
                .ok()
                .map(|paths| (attr, paths))
        })
    {
        let paths = args.0.iter().chain(paths.0.iter());
        *attr = parse_quote!(#[mry::lock(#(#paths),*)]);
        return input.into_token_stream();
    }
    let args = args.0.into_iter().map(|arg| {
        let name = arg
            .to_token_stream()
            .to_string()
            .replace(" :: ", "::")
            .replace("< ", "<")
            .replace(" >", ">");
        quote![(std::any::Any::type_id(&#arg), #name.to_string())]
    });
    let block = input.block.clone();
    input.block.stmts.clear();
    let mutexes = quote![mry::__mutexes(vec![#(#args,)*])];
    input.block.stmts.insert(
        0,
        syn::Stmt::Expr(
            if input.sig.asyncness.is_some() {
                parse_quote! {
                    mry::__async_lock_and_run(#mutexes, move || Box::pin(async #block)).await
                }
            } else {
                parse_quote! {
                    mry::__lock_and_run(#mutexes, move || #block)
                }
            },
            None,
        ),
    );
    input.into_token_stream()
}

pub(crate) fn unsafe_transform(args: LockPaths, mut input: ItemFn) -> TokenStream {
    if let Some((attr, paths)) = input
        .attrs
        .iter_mut()
        .find(|attr| {
            let path = attr.path();
            if path.segments.len() == 1 && path.segments[0].ident == "unsafe_lock" {
                return true;
            }
            if path.segments.len() == 2
                && path.segments[0].ident == "mry"
                && path.segments[1].ident == "unsafe_lock"
            {
                return true;
            }
            false
        })
        .and_then(|attr| {
            attr.parse_args::<LockPaths>()
                .ok()
                .map(|paths| (attr, paths))
        })
    {
        let paths = args.0.iter().chain(paths.0.iter());
        *attr = parse_quote!(#[mry::unsafe_lock(#(#paths),*)]);
        return input.into_token_stream();
    }
    let args = args.0.into_iter().map(|arg| {
        let name = arg
            .to_token_stream()
            .to_string()
            .replace(" :: ", "::")
            .replace("< ", "<")
            .replace(" >", ">");
        quote![(std::any::Any::type_id(&#arg), #name.to_string())]
    });
    let block = input.block.clone();
    input.block.stmts.clear();
    let mutexes = quote![mry::unsafe_mocks::__mutexes(vec![#(#args,)*])];
    input.block.stmts.insert(
        0,
        syn::Stmt::Expr(
            if input.sig.asyncness.is_some() {
                panic!("async signature is not supported by unsafe mocks.")
            } else {
                parse_quote! {
                    mry::unsafe_mocks::__lock_and_run(#mutexes, move || #block)
                }
            },
            None,
        ),
    );
    input.into_token_stream()
}

#[cfg(test)]
mod test {
    use pretty_assertions::assert_eq;
    use syn::{parse2, parse_str};

    use super::*;

    #[test]
    fn lock() {
        let args = LockPaths(vec![
            parse_str("<A as B>::a").unwrap(),
            parse_str("a::a").unwrap(),
            parse_str("b::b").unwrap(),
        ]);
        let input: ItemFn = parse2(quote! {
            #[test]
            fn test_meow() {
                assert!(true);
            }
        })
        .unwrap();

        assert_eq!(
            transform(args, input).to_string(),
            quote! {
                #[test]
                fn test_meow() {
                    mry::__lock_and_run(mry::__mutexes(vec![
                        (std::any::Any::type_id(&<A as B> :: a), "<A as B>::a".to_string()),
                        (std::any::Any::type_id(&a :: a), "a::a".to_string()),
                        (std::any::Any::type_id(&b :: b), "b::b".to_string()),
                    ]), move | | {
                        assert!(true);
                    })
                }
            }
            .to_string()
        );
    }

    #[test]
    fn concats_multiple_locks() {
        let args = LockPaths(vec![parse_str("a::a").unwrap(), parse_str("b::b").unwrap()]);
        let input: ItemFn = parse2(quote! {
            #[mry::lock(c::c)]
            #[test]
            fn test_meow() {
                assert!(true);
            }
        })
        .unwrap();

        assert_eq!(
            transform(args, input).to_string(),
            quote! {
                #[mry::lock(a::a, b::b, c::c)]
                #[test]
                fn test_meow() {
                    assert!(true);
                }
            }
            .to_string()
        );
    }
}
