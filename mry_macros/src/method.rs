use proc_macro2::{Span, TokenStream};
use quote::quote;
use syn::{Attribute, Block, FnArg, Ident, Pat, PatIdent, ReturnType, Signature, Type, Visibility};

pub fn transform(
    struct_name: &str,
    tokens: TokenStream,
    vis: Option<&Visibility>,
    attrs: &Vec<Attribute>,
    sig: &Signature,
    block: &TokenStream,
) -> (TokenStream, TokenStream) {
    // Split into receiver and other inputs
    let receiver;
    let mut inputs = sig.inputs.iter();
    if let Some(FnArg::Receiver(rcv)) = inputs.next() {
        receiver = rcv;
    } else {
        return (tokens.clone(), TokenStream::default());
    }
    let inputs: Vec<_> = inputs
        .map(|input| {
            if let FnArg::Typed(typed_arg) = input {
                typed_arg.clone()
            } else {
                panic!("multiple receiver?");
            }
        })
        .collect();
    let mut bindings = Vec::new();

    let generics = &sig.generics;
    let body = &block;
    let attrs = attrs.clone();
    let ident = sig.ident.clone();
    let mock_ident = Ident::new(&format!("mock_{}", ident), Span::call_site());
    let args_with_type: Vec<_> = inputs
        .iter()
        .enumerate()
        .map(|(i, input)| {
            if let Pat::Ident(_) = *input.pat {
                input.clone()
            } else {
                let pat = input.pat.clone();
                let arg_name = Ident::new(&format!("arg{}", i), Span::call_site());
                bindings.push((pat, arg_name.clone()));
                let ident = Pat::Ident(PatIdent {
                    attrs: Default::default(),
                    by_ref: Default::default(),
                    mutability: Default::default(),
                    ident: arg_name,
                    subpat: Default::default(),
                });
                let mut arg_with_type = (*input).clone();
                arg_with_type.pat = Box::new(ident.clone());
                arg_with_type
            }
        })
        .collect();
    let derefed_input_type_tuple: Vec<_> = args_with_type
        .iter()
        .map(|input| {
            if is_str(&input.ty) {
                return quote!(String);
            }
            let ty = match &*input.ty {
                Type::Reference(ty) => {
                    let ty = &ty.elem;
                    quote!(#ty)
                }
                ty => quote!(#ty),
            };
            ty
        })
        .collect();
    let derefed_input: Vec<_> = args_with_type
        .iter()
        .map(|input| {
            let pat = &input.pat;
            if is_str(&input.ty) {
                return quote!(#pat.to_string());
            }
            let input = match &*input.ty {
                Type::Reference(_ty) => {
                    quote!(*#pat)
                }
                _ => quote!(#pat),
            };
            input
        })
        .collect();
    let output_type = match &sig.output {
        ReturnType::Default => quote!(()),
        ReturnType::Type(_, ty) => quote!(#ty),
    };
    let asyn = &sig.asyncness;
    let vis = &vis;
    let name = format!("{}::{}", struct_name, ident.to_string());
    let args_with_type = quote!(#(#args_with_type),*);
    let input_type_tuple = quote!((#(#derefed_input_type_tuple),*));
    let derefed_input_tuple = quote!((#(#derefed_input),*));
    let bindings = bindings.iter().map(|(pat, arg)| quote![let #pat = #arg;]);
    let behavior_name = Ident::new(&format!("Behavior{}", inputs.len()), Span::call_site());
    let behavior_type = quote! {
        mry::#behavior_name<#input_type_tuple, #output_type>
    };
    (
        quote! {
            #(#attrs)*
            #vis #asyn fn #ident #generics(#receiver, #args_with_type) -> #output_type {
                #[cfg(test)]
                if self.mry.is_some() {
                    return mry::MOCK_DATA
                        .lock()
                        .get_mut_or_create::<#input_type_tuple, #output_type>(&self.mry, #name)
                        ._inner_called(&#derefed_input_tuple);
                }
                #(#bindings)*
                #body
            }
        },
        quote! {
            #[cfg(test)]
            pub fn #mock_ident<'mry>(&'mry mut self) -> mry::MockLocator<'mry, #input_type_tuple, #output_type, #behavior_type> {
                if self.mry.is_none() {
                    self.mry = mry::Mry::generate();
                }
                mry::MockLocator {
                    id: &self.mry,
                    name: #name,
                    _phantom: Default::default(),
                }
            }
        },
    )
}

pub fn is_str(ty: &Type) -> bool {
    match ty {
        Type::Reference(ty) => {
            if let Type::Path(path) = &*ty.elem {
                if let Some(ident) = path.path.get_ident() {
                    return ident.to_string() == "str";
                }
            }
            false
        }
        _ => false,
    }
}
