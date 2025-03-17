use darling::{ast::NestedMeta, FromMeta};
use syn::{visit::Visit, Meta};

#[derive(FromMeta, Default)]
pub(crate) struct MryAttr {
    pub debug: darling::util::Flag,
    pub non_send: Option<NotSend>,
    pub skip_args: Option<Skip>,
    pub skip_fns: Option<Skip>,
}

pub(crate) struct NotSend(pub Vec<syn::Path>);
pub(crate) struct Skip(pub Vec<syn::Path>);

impl FromMeta for NotSend {
    fn from_list(list: &[NestedMeta]) -> darling::Result<Self> {
        list.iter()
            .map(|meta| match meta {
                NestedMeta::Meta(Meta::Path(path)) => Ok(path.clone()),
                _ => Err(darling::Error::custom(
                    "expected a list of types like non_send(T, U)",
                )),
            })
            .collect::<Result<Vec<_>, _>>()
            .map(NotSend)
    }
}

impl FromMeta for Skip {
    fn from_list(list: &[NestedMeta]) -> darling::Result<Self> {
        list.iter()
            .map(|meta| match meta {
                NestedMeta::Meta(Meta::Path(path)) => Ok(path.clone()),
                _ => Err(darling::Error::custom(
                    "expected a list of types like skip(T, U)",
                )),
            })
            .collect::<Result<Vec<_>, _>>()
            .map(Skip)
    }
}

impl MryAttr {
    pub fn test_non_send(&self, ty: &syn::Type) -> bool {
        let Some(non_send) = &self.non_send else {
            return false;
        };
        let mut visitor = TypeVisitor {
            paths: &non_send.0,
            found: false,
        };
        visitor.visit_type(ty);
        visitor.found
    }

    pub fn test_skip_args(&self, ty: &syn::Type) -> bool {
        let Some(skip) = &self.skip_args else {
            return false;
        };
        let mut visitor = TypeVisitor {
            paths: &skip.0,
            found: false,
        };
        visitor.visit_type(ty);
        visitor.found
    }

    pub fn should_skip_method(&self, method_name: &syn::Ident) -> bool {
        if let Some(skip) = &self.skip_fns {
            if skip.0.iter().any(|p| p.is_ident(method_name)) {
                return true;
            }
        }
        false
    }
}

struct TypeVisitor<'a> {
    paths: &'a Vec<syn::Path>,
    found: bool,
}
impl Visit<'_> for TypeVisitor<'_> {
    fn visit_path(&mut self, path: &syn::Path) {
        if self.found {
            return;
        }
        if self.paths.iter().any(|p| p == path) {
            self.found = true;
            return;
        }
        for segment in path.segments.iter() {
            self.visit_path_segment(segment);
        }
    }
    fn visit_ident(&mut self, ident: &syn::Ident) {
        if self.found {
            return;
        }
        if self.paths.iter().any(|p| p.is_ident(ident)) {
            self.found = true;
        }
    }
}

#[cfg(test)]
mod tests {
    use syn::parse_quote;

    use super::*;

    #[test]
    fn test_debug() {
        let attr = MryAttr::from_list(
            &NestedMeta::parse_meta_list(parse_quote! {
                debug
            })
            .unwrap(),
        )
        .unwrap();
        assert!(attr.debug.is_present());
    }

    #[test]
    fn test_non_send() {
        let attr = MryAttr::from_list(
            &NestedMeta::parse_meta_list(parse_quote! {
                debug,non_send(T, U)
            })
            .unwrap(),
        )
        .unwrap();
        let lists = attr.non_send.unwrap().0;
        assert_eq!(lists.len(), 2);
        assert_eq!(lists[0], parse_quote!(T));
        assert_eq!(lists[1], parse_quote!(U));
    }

    #[test]
    fn test_non_send_path() {
        let attr = MryAttr::from_list(
            &NestedMeta::parse_meta_list(parse_quote! {
                debug,non_send(A::B)
            })
            .unwrap(),
        )
        .unwrap();

        assert!(attr.test_non_send(&parse_quote!(A::B)));
        assert!(!attr.test_non_send(&parse_quote!(A::C)));
    }

    #[test]
    fn test_non_send_rc() {
        let attr = MryAttr::from_list(
            &NestedMeta::parse_meta_list(parse_quote! {
                debug,non_send(Rc)
            })
            .unwrap(),
        )
        .unwrap();

        assert!(attr.test_non_send(&parse_quote!(Rc<String>)));
        assert!(!attr.test_non_send(&parse_quote!(String)));
    }

    #[test]
    fn test_skip_method() {
        let attr = MryAttr::from_list(
            &NestedMeta::parse_meta_list(parse_quote! {
                skip_fns(skipped)
            })
            .unwrap(),
        )
        .unwrap();
        assert!(attr.should_skip_method(&parse_quote!(skipped)));
        assert!(!attr.should_skip_method(&parse_quote!(not_skipped)));
    }
}
