use quote::quote_spanned;

#[derive(Default)]
pub(crate) struct Options {
    pub skip: Vec<Skip>,
}

impl Options {
    pub fn validate(&self) -> Result<(), Vec<proc_macro2::TokenStream>> {
        let errors: Vec<_> = self
            .skip
            .iter()
            .filter_map(|skip| {
                if !skip.used {
                    let name = &skip.type_name;
                    Some(quote_spanned! {
                        name.span() => compile_error!(concat!("Type ", #name, " is not found."));
                    })
                } else {
                    None
                }
            })
            .collect();
        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors)
        }
    }
}

#[derive(Debug)]
pub(crate) struct Skip {
    pub type_name: syn::LitStr,
    pub used: bool,
}

#[cfg(test)]
impl Skip {
    pub fn new(name: &str) -> Self {
        Self {
            type_name: syn::LitStr::new(name, proc_macro2::Span::call_site()),
            used: false,
        }
    }
}
