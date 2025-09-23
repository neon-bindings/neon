#[derive(Default)]
pub(crate) struct Meta {
    pub(super) kind: Kind,
    pub(super) name: Option<syn::LitStr>,
    pub(super) json: bool,
    pub(super) context: bool,
    pub(super) this: bool,
}

#[derive(Default)]
pub(super) enum Kind {
    Async,
    AsyncFn,
    #[default]
    Normal,
    Task,
}

// Property metadata structure (simpler than method metadata)
#[derive(Default)]
pub(crate) struct PropertyMeta {
    pub(super) name: Option<syn::LitStr>,
    pub(super) json: bool,
}

pub(crate) struct PropertyParser;

impl syn::parse::Parser for PropertyParser {
    type Output = PropertyMeta;

    fn parse2(self, tokens: proc_macro2::TokenStream) -> syn::Result<Self::Output> {
        let mut attr = PropertyMeta::default();
        let parser = syn::meta::parser(|meta: syn::meta::ParseNestedMeta<'_>| {
            if meta.path.is_ident("name") {
                attr.name = Some(meta.value()?.parse::<syn::LitStr>()?);
                return Ok(());
            }

            if meta.path.is_ident("json") {
                attr.json = true;
                return Ok(());
            }

            // Properties don't support method-specific attributes
            if meta.path.is_ident("async") || meta.path.is_ident("task") ||
               meta.path.is_ident("context") || meta.path.is_ident("this") {
                return Err(meta.error("attribute not supported on const properties"));
            }

            Err(meta.error("unsupported property attribute"))
        });

        parser.parse2(tokens)?;

        Ok(attr)
    }
}

// Parser that preserves existing metadata (like async detection)
pub(crate) struct Parser(pub(crate) Meta);

impl syn::parse::Parser for Parser {
    type Output = Meta;

    fn parse2(self, tokens: proc_macro2::TokenStream) -> syn::Result<Self::Output> {
        let Parser(mut attr) = self;
        let parser = syn::meta::parser(|meta: syn::meta::ParseNestedMeta<'_>| {
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

            if meta.path.is_ident("this") {
                attr.this = true;
                return Ok(());
            }

            if meta.path.is_ident("async") {
                if matches!(attr.kind, Kind::AsyncFn) {
                    return Err(
                        meta.error("`async` attribute should not be used with an `async fn`")
                    );
                }
                attr.kind = Kind::Async;
                return Ok(());
            }

            if meta.path.is_ident("task") {
                attr.kind = Kind::Task;
                return Ok(());
            }

            Err(meta.error("unsupported property"))
        });

        parser.parse2(tokens)?;

        Ok(attr)
    }
}
