mod create_behaviors;
mod create_matchers;
mod item_fn;
mod item_impl;
mod item_struct;
mod item_trait;
mod lock;
mod method;
mod new;
mod options;
use darling::ast::NestedMeta;
use darling::FromMeta;
use lock::LockPaths;
mod alphabets;
use options::{Options, Skip};
use quote::{quote, ToTokens};
use syn::{
    parse2, parse_macro_input, visit_mut::VisitMut, ExprStruct, ItemFn, ItemImpl, ItemStruct,
    ItemTrait,
};

enum TargetItem {
    Struct(ItemStruct),
    Impl(ItemImpl),
    Trait(ItemTrait),
    Fn(ItemFn),
}

#[derive(FromMeta)]
struct MryAttr {
    debug: darling::util::Flag,
    skip: Option<Vec<syn::LitStr>>,
}

#[proc_macro_attribute]
pub fn mry(
    attr: proc_macro::TokenStream,
    input: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
    let attr = MryAttr::from_list(&NestedMeta::parse_meta_list(attr.into()).unwrap()).unwrap();
    run(attr, input.into()).into()
}

fn run(attr: MryAttr, input: proc_macro2::TokenStream) -> proc_macro2::TokenStream {
    let mut options = Options {
        skip: attr
            .skip
            .into_iter()
            .flatten()
            .map(|lit| Skip {
                type_name: lit,
                used: false,
            })
            .collect(),
    };
    match parse2(input.clone())
        .map(TargetItem::Struct)
        .or_else(|_| parse2(input.clone()).map(TargetItem::Impl))
        .or_else(|_| parse2(input.clone()).map(TargetItem::Trait))
        .or_else(|_| parse2(input.clone()).map(TargetItem::Fn))
    {
        Ok(target) => {
            let token_stream = match target {
                TargetItem::Struct(target) => item_struct::transform(&mut options, target),
                TargetItem::Impl(target) => item_impl::transform(&mut options, target),
                TargetItem::Trait(target) => item_trait::transform(&mut options, target),
                TargetItem::Fn(target) => item_fn::transform(&mut options, target),
            };
            if attr.debug.is_present() {
                println!("{}", token_stream);
            }
            if let Err(errors) = options.validate() {
                quote! {
                    #(#errors)*
                }
            } else {
                token_stream
            }
        }
        Err(err) => err.to_compile_error(),
    }
}

#[test]
fn validate_skip() {
    let output = run(
        MryAttr {
            debug: false.into(),
            skip: Some(vec![syn::LitStr::new(
                "String",
                proc_macro2::Span::call_site(),
            )]),
        },
        quote! { struct A; },
    )
    .to_string();
    assert_eq!(
        output,
        quote! { compile_error!(concat!("Type ", "String", " is not found.")); }.to_string()
    );
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

struct M(proc_macro2::TokenStream);

impl VisitMut for M {
    fn visit_item_trait_mut(&mut self, i: &mut ItemTrait) {
        item_trait::transform(&mut Default::default(), i.clone()).to_tokens(&mut self.0)
    }
    fn visit_item_struct_mut(&mut self, i: &mut ItemStruct) {
        item_struct::transform(&mut Default::default(), i.clone()).to_tokens(&mut self.0)
    }
    fn visit_item_impl_mut(&mut self, i: &mut ItemImpl) {
        item_impl::transform(&mut Default::default(), i.clone()).to_tokens(&mut self.0)
    }
}

#[proc_macro]
pub fn m(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let mut m = M(proc_macro2::TokenStream::default());
    m.visit_file_mut(&mut parse2(input.into()).unwrap());
    m.0.into()
}
