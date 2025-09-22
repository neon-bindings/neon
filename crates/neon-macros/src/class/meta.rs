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
