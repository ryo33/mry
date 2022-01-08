use proc_macro2::TokenStream;
use quote::{quote, ToTokens};
use syn::{parse_quote, AttributeArgs, ItemFn, Stmt};

pub(crate) fn transform(args: AttributeArgs, mut input: ItemFn) -> TokenStream {
    let args = args
        .into_iter()
        .map(|arg| quote![std::any::Any::type_id(&#arg)]);
    let block = input.block.clone();
    input.block.stmts.clear();
    let mutexes = quote![mry::__mutexes(vec![#(#args,)*])];
    input.block.stmts.insert(
        0,
        Stmt::Expr(if input.sig.asyncness.is_some() {
            parse_quote! {
                mry::__async_lock_and_run(#mutexes, move || Box::pin(async #block)).await
            }
        } else {
            parse_quote! {
                mry::__lock_and_run(#mutexes, move || #block)
            }
        }),
    );
    input.into_token_stream()
}

#[cfg(test)]
mod test {
    use pretty_assertions::assert_eq;
    use syn::{parse2, parse_str, NestedMeta};

    use super::*;

    #[test]
    fn lock() {
        let args: AttributeArgs = vec![
            NestedMeta::Meta(syn::Meta::Path(parse_str("a").unwrap())),
            NestedMeta::Meta(syn::Meta::Path(parse_str("b").unwrap())),
        ];
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
                        std::any::Any::type_id(&a),
                        std::any::Any::type_id(&b),
                    ]), move | | {
                        assert!(true);
                    })
                }
            }
            .to_string()
        );
    }
}
