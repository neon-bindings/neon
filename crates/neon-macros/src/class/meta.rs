#[derive(Default)]
pub(crate) struct Meta {
    pub(super) name: Option<syn::LitStr>,
}

pub(crate) struct Parser;

impl syn::parse::Parser for Parser {
    type Output = Meta;

    fn parse2(self, tokens: proc_macro2::TokenStream) -> syn::Result<Self::Output> {
        let mut attr = Meta::default();
        let parser = syn::meta::parser(|meta: syn::meta::ParseNestedMeta<'_>| {
            if meta.path.is_ident("name") {
                attr.name = Some(meta.value()?.parse::<syn::LitStr>()?);

                return Ok(());
            }

            Err(meta.error("unsupported property"))
        });

        parser.parse2(tokens)?;

        Ok(attr)
    }
}
