use quote::ToTokens;
use syn::{Path, Type};

pub(crate) fn type_name(ty: &Type) -> String {
    match ty {
        Type::Path(type_path) => path_name(&type_path.path),
        ty => ty.to_token_stream().to_string(),
    }
}

pub(crate) fn path_name(path: &Path) -> String {
    path.segments
        .iter()
        .map(|segment| match &segment.arguments {
            syn::PathArguments::None => segment.ident.to_string(),
            syn::PathArguments::AngleBracketed(args) => {
                let args = args
                    .args
                    .iter()
                    .map(|arg| arg.to_token_stream().to_string())
                    .collect::<Vec<_>>()
                    .join(", ");
                format!("{}<{}>", segment.ident.to_string(), args)
            }
            syn::PathArguments::Parenthesized(_) => todo!(),
        })
        .collect::<Vec<_>>()
        .join("::")
}

#[cfg(test)]
mod test {
    use super::*;
    use quote::quote;
    use syn::parse2;

    #[test]
    fn returns_struct_name() {
        let ty = parse2(quote!(A)).unwrap();
        assert_eq!(type_name(&ty), "A");
    }

    #[test]
    fn returns_with_generics() {
        let ty = parse2(quote!(a::A<'a, B, C>)).unwrap();
        assert_eq!(type_name(&ty), "a::A<'a, B, C>");
    }
}
