mod create_behaviors;
mod expr_struct;
mod item_impl;
mod item_struct;
use syn::{parse, parse_macro_input, ExprStruct, ItemImpl, ItemStruct};

enum Target {
    ItemStruct(ItemStruct),
    ItemImpl(ItemImpl),
}

#[proc_macro_attribute]
pub fn mry(_: proc_macro::TokenStream, input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    match parse(input.clone())
        .map(Target::ItemStruct)
        .or_else(|_| parse(input.clone()).map(Target::ItemImpl))
    {
        Ok(target) => {
            let token_stream = match target {
                Target::ItemStruct(target) => item_struct::transform(target),
                Target::ItemImpl(target) => item_impl::transform(target),
            };
            token_stream.into()
        }
        Err(err) => err.to_compile_error().into(),
    }
}

#[proc_macro]
pub fn new(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    expr_struct::transform(parse_macro_input!(input as ExprStruct)).into()
}

#[proc_macro]
pub fn create_behaviors(_: proc_macro::TokenStream) -> proc_macro::TokenStream {
    proc_macro::TokenStream::from(create_behaviors::create())
}
