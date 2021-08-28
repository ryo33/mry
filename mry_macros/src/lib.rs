mod create_behaviors;
mod create_matchers;
mod item_impl;
mod item_struct;
mod item_trait;
mod method;
mod new;
mod type_name;
use proc_macro2::TokenStream;
use quote::ToTokens;
use syn::visit_mut::VisitMut;
mod alphabets;
use syn::{parse, parse2, parse_macro_input, ExprStruct, ItemImpl, ItemStruct, ItemTrait};

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
                Target::ItemStruct(target) => item_struct::transform(&target),
                Target::ItemImpl(mut target) => item_impl::transform(&mut target),
                Target::ItemTrait(target) => item_trait::transform(&target),
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
    create_behaviors::create().into()
}

#[proc_macro]
pub fn create_matchers(_: proc_macro::TokenStream) -> proc_macro::TokenStream {
    create_matchers::create().into()
}

struct M(TokenStream);

impl VisitMut for M {
    fn visit_item_trait_mut(&mut self, i: &mut ItemTrait) {
        item_trait::transform(i).to_tokens(&mut self.0)
    }
    fn visit_item_struct_mut(&mut self, i: &mut ItemStruct) {
        item_struct::transform(i).to_tokens(&mut self.0)
    }
    fn visit_item_impl_mut(&mut self, i: &mut ItemImpl) {
        item_impl::transform(i).to_tokens(&mut self.0)
    }
}

#[proc_macro]
pub fn m(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let mut m = M(TokenStream::default());
    m.visit_file_mut(&mut parse2(input.into()).unwrap());
    m.0.into()
}
