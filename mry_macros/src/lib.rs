mod create_behaviors;
mod create_matchers;
mod item_fn;
mod item_impl;
mod item_struct;
mod item_trait;
mod lock;
mod method;
mod new;
use darling::ast::NestedMeta;
use darling::FromMeta;
use lock::LockPaths;
use proc_macro2::TokenStream;
use quote::ToTokens;
use syn::visit_mut::VisitMut;
use syn::{parse, parse2, parse_macro_input, ExprStruct, ItemFn, ItemImpl, ItemStruct, ItemTrait};

/// functions may not be instrumented if they take more than this number of arguments
const MAX_ARGUMENT_COUNT: u32 = 10;

enum TargetItem {
    Struct(ItemStruct),
    Impl(ItemImpl),
    Trait(ItemTrait),
    Fn(ItemFn),
}

#[derive(FromMeta)]
struct MryAttr {
    debug: darling::util::Flag,
}

#[proc_macro_attribute]
pub fn mry(
    attr: proc_macro::TokenStream,
    input: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
    let attr = MryAttr::from_list(&NestedMeta::parse_meta_list(attr.into()).unwrap()).unwrap();
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
            if attr.debug.is_present() {
                println!("{}", token_stream);
            }
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

#[proc_macro]
pub fn unsafe_create_behaviors(_: proc_macro::TokenStream) -> proc_macro::TokenStream {
    create_behaviors::unsafe_create().into()
}

#[proc_macro]
pub fn unsafe_create_matchers(_: proc_macro::TokenStream) -> proc_macro::TokenStream {
    create_matchers::unsafe_create().into()
}

#[proc_macro_attribute]
pub fn unsafe_mry(
    attr: proc_macro::TokenStream,
    input: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
    let attr = MryAttr::from_list(&NestedMeta::parse_meta_list(attr.into()).unwrap()).unwrap();
    match parse(input.clone())
        .map(TargetItem::Struct)
        .or_else(|_| parse(input.clone()).map(TargetItem::Impl))
        .or_else(|_| parse(input.clone()).map(TargetItem::Trait))
        .or_else(|_| parse(input.clone()).map(TargetItem::Fn))
    {
        Ok(target) => {
            let token_stream = match target {
                TargetItem::Struct(target) => item_struct::unsafe_transform(target),
                TargetItem::Impl(target) => item_impl::unsafe_transform(target),
                TargetItem::Trait(target) => item_trait::unsafe_transform(target),
                TargetItem::Fn(target) => item_fn::unsafe_transform(target),
            };
            if attr.debug.is_present() {
                println!("{}", token_stream);
            }
            token_stream.into()
        }
        Err(err) => err.to_compile_error().into(),
    }
}

#[proc_macro_attribute]
pub fn unsafe_lock(
    attribute: proc_macro::TokenStream,
    input: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
    lock::unsafe_transform(
        parse_macro_input!(attribute as LockPaths),
        parse_macro_input!(input as ItemFn),
    )
    .into()
}

struct UnsafeM(TokenStream);

impl VisitMut for UnsafeM {
    fn visit_item_trait_mut(&mut self, i: &mut ItemTrait) {
        item_trait::unsafe_transform(i.clone()).to_tokens(&mut self.0)
    }
    fn visit_item_struct_mut(&mut self, i: &mut ItemStruct) {
        item_struct::unsafe_transform(i.clone()).to_tokens(&mut self.0)
    }
    fn visit_item_impl_mut(&mut self, i: &mut ItemImpl) {
        item_impl::unsafe_transform(i.clone()).to_tokens(&mut self.0)
    }
}

#[proc_macro]
pub fn unsafe_m(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let mut m = UnsafeM(TokenStream::default());
    m.visit_file_mut(&mut parse2(input.into()).unwrap());
    m.0.into()
}
