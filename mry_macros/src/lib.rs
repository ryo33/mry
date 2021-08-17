mod create_behaviors;
mod item_impl;
mod item_struct;
mod item_trait;
mod method;
mod new;
use syn::{parse, parse_macro_input, ExprStruct, ItemImpl, ItemStruct, ItemTrait};

enum Target {
    ItemStruct(ItemStruct),
    ItemImpl(ItemImpl),
    ItemTrait(ItemTrait),
}

#[proc_macro_attribute]
pub fn mry(_: proc_macro::TokenStream, input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    match parse(input.clone())
        .map(Target::ItemStruct)
        .or_else(|_| parse(input.clone()).map(Target::ItemImpl))
        .or_else(|_| parse(input.clone()).map(Target::ItemTrait))
    {
        Ok(target) => {
            let token_stream = match target {
                Target::ItemStruct(target) => item_struct::transform(target),
                Target::ItemImpl(target) => item_impl::transform(target),
                Target::ItemTrait(target) => item_trait::transform(target),
            };
            token_stream.into()
        }
        Err(err) => err.to_compile_error().into(),
    }
}

#[proc_macro]
pub fn new(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    new::transform(parse_macro_input!(input as ExprStruct)).into()
}

#[proc_macro]
pub fn create_behaviors(_: proc_macro::TokenStream) -> proc_macro::TokenStream {
    proc_macro::TokenStream::from(create_behaviors::create())
}
