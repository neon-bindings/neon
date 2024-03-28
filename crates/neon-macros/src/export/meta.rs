#[derive(Default)]
pub(super) struct ExportMeta {
    pub(super) name: Option<syn::LitStr>,
    pub(super) json: bool,
    pub(super) context: bool,
}

pub(super) struct ExportParser;

impl syn::parse::Parser for ExportParser {
    type Output = ExportMeta;

    fn parse2(self, tokens: proc_macro2::TokenStream) -> syn::Result<Self::Output> {
        let mut attr = ExportMeta::default();
        let parser = syn::meta::parser(|meta| {
            if meta.path.is_ident("name") {
                attr.name = Some(meta.value()?.parse::<syn::LitStr>()?);

                return Ok(());
            }

            if meta.path.is_ident("json") {
                attr.json = true;

                return Ok(());
            }

            if meta.path.is_ident("context") {
                attr.context = true;

                return Ok(());
            }

            Err(meta.error("unsupported property"))
        });

        parser.parse2(tokens)?;

        Ok(attr)
    }
}
