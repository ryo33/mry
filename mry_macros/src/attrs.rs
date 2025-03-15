use darling::{ast::NestedMeta, FromMeta};
use syn::{visit::Visit, Meta};

#[derive(FromMeta, Default)]
pub(crate) struct MryAttr {
    pub debug: darling::util::Flag,
    pub non_send: Option<NotSend>,
}

pub(crate) struct NotSend(pub Vec<syn::Path>);

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

impl MryAttr {
    pub fn test_non_send(&self, ty: &syn::Type) -> bool {
        let Some(non_send) = &self.non_send else {
            return false;
        };
        struct Visitor<'a> {
            non_send: &'a NotSend,
            found: bool,
        }
        impl Visit<'_> for Visitor<'_> {
            fn visit_path(&mut self, path: &syn::Path) {
                if self.found {
                    return;
                }
                if self.non_send.0.iter().any(|p| p == path) {
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
                if self.non_send.0.iter().any(|p| p.is_ident(ident)) {
                    self.found = true;
                }
            }
        }
        let mut visitor = Visitor {
            non_send,
            found: false,
        };
        visitor.visit_type(ty);
        visitor.found
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
}
