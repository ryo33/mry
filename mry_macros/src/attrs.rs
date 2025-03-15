use darling::{ast::NestedMeta, FromMeta};
use syn::{visit::Visit, Meta};

#[derive(FromMeta, Default)]
pub(crate) struct MryAttr {
    pub debug: darling::util::Flag,
    pub not_send: Option<NotSend>,
}

pub(crate) struct NotSend(pub Vec<syn::Path>);

impl FromMeta for NotSend {
    fn from_list(list: &[NestedMeta]) -> darling::Result<Self> {
        list.iter()
            .map(|meta| match meta {
                NestedMeta::Meta(Meta::Path(path)) => Ok(path.clone()),
                _ => Err(darling::Error::custom(
                    "expected a list of types like not_send(T, U)",
                )),
            })
            .collect::<Result<Vec<_>, _>>()
            .map(NotSend)
    }
}

impl MryAttr {
    pub fn test_non_send(&self, ty: &syn::Type) -> bool {
        let Some(not_send) = &self.not_send else {
            return false;
        };
        struct Visitor<'a> {
            not_send: &'a NotSend,
            found: bool,
        }
        impl Visit<'_> for Visitor<'_> {
            fn visit_path(&mut self, path: &syn::Path) {
                if self.found {
                    return;
                }
                if self.not_send.0.iter().any(|p| p == path) {
                    self.found = true;
                }
            }
        }
        let mut visitor = Visitor {
            not_send,
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
    fn test_not_send() {
        let attr = MryAttr::from_list(
            &NestedMeta::parse_meta_list(parse_quote! {
                debug,not_send(T, U)
            })
            .unwrap(),
        )
        .unwrap();
        assert!(attr.debug.is_present());
        assert!(matches!(attr.not_send, Some(NotSend(_))));
        let lists = attr.not_send.as_ref().unwrap().0.clone();
        assert_eq!(lists.len(), 2);
        assert_eq!(lists[0], parse_quote!(T));
        assert_eq!(lists[1], parse_quote!(U));
    }
}
