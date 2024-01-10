mod create_behaviors;
mod create_matchers;
mod item_fn;
mod item_impl;
mod item_struct;
mod item_trait;
mod lock;
mod method;
mod new;
use lock::LockPaths;
use proc_macro2::TokenStream;
use quote::ToTokens;
use syn::visit_mut::VisitMut;
mod alphabets;
use syn::{parse, parse2, parse_macro_input, ExprStruct, ItemFn, ItemImpl, ItemStruct, ItemTrait};

enum TargetItem {
    Struct(ItemStruct),
    Impl(ItemImpl),
    Trait(ItemTrait),
    Fn(ItemFn),
}

#[proc_macro_attribute]
pub fn mry(_: proc_macro::TokenStream, input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    match parse(input.clone())
        .map(TargetItem::Struct)
        .or_else(|_| parse(input.clone()).map(TargetItem::Impl))
        .or_else(|_| parse(input.clone()).map(TargetItem::Trait))
        .or_else(|_| parse(input.clone()).map(TargetItem::Fn))
    {
        Ok(target) => {
            let token_stream = match target {
                TargetItem::Struct(target) => item_struct::transform(target),
                TargetItem::Impl(target) => item_impl::transform(target),
                TargetItem::Trait(target) => item_trait::transform(target),
                TargetItem::Fn(target) => item_fn::transform(target),
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

#[proc_macro_attribute]
pub fn lock(
    attribute: proc_macro::TokenStream,
    input: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
    lock::transform(
        parse_macro_input!(attribute as LockPaths),
        parse_macro_input!(input as ItemFn),
    )
    .into()
}

struct M(TokenStream);

impl VisitMut for M {
    fn visit_item_trait_mut(&mut self, i: &mut ItemTrait) {
        item_trait::transform(i.clone()).to_tokens(&mut self.0)
    }
    fn visit_item_struct_mut(&mut self, i: &mut ItemStruct) {
        item_struct::transform(i.clone()).to_tokens(&mut self.0)
    }
    fn visit_item_impl_mut(&mut self, i: &mut ItemImpl) {
        item_impl::transform(i.clone()).to_tokens(&mut self.0)
    }
}

#[proc_macro]
pub fn m(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let mut m = M(TokenStream::default());
    m.visit_file_mut(&mut parse2(input.into()).unwrap());
    m.0.into()
}
