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

use crate::error::ErrorCode;

impl Meta {
    fn set_name(&mut self, meta: syn::meta::ParseNestedMeta) -> syn::Result<()> {
        self.name = Some(meta.value()?.parse::<syn::LitStr>()?);

        Ok(())
    }

    fn force_json(&mut self, _meta: syn::meta::ParseNestedMeta) -> syn::Result<()> {
        self.json = true;

        Ok(())
    }

    fn force_context(&mut self, _meta: syn::meta::ParseNestedMeta) -> syn::Result<()> {
        self.context = true;

        Ok(())
    }

    fn force_this(&mut self, _meta: syn::meta::ParseNestedMeta) -> syn::Result<()> {
        self.this = true;

        Ok(())
    }

    fn make_async(&mut self, meta: syn::meta::ParseNestedMeta) -> syn::Result<()> {
        if matches!(self.kind, Kind::AsyncFn) {
            return Err(meta.error(
                format!(
                    "{} [{}]",
                    ErrorCode::AsyncAttrAsyncFn.message(),
                    ErrorCode::AsyncAttrAsyncFn.code()
                ),
            ));
        }

        self.kind = Kind::Async;

        Ok(())
    }

    fn make_task(&mut self, _meta: syn::meta::ParseNestedMeta) -> syn::Result<()> {
        self.kind = Kind::Task;

        Ok(())
    }
}

pub(crate) struct Parser(syn::ItemFn);

impl Parser {
    pub(crate) fn new(item: syn::ItemFn) -> Self {
        Self(item)
    }
}

impl syn::parse::Parser for Parser {
    type Output = (syn::ItemFn, Meta);

    fn parse2(self, tokens: proc_macro2::TokenStream) -> syn::Result<Self::Output> {
        let Self(item) = self;
        let mut attr = Meta::default();

        if item.sig.asyncness.is_some() {
            attr.kind = Kind::AsyncFn;
        }

        let parser = syn::meta::parser(|meta| {
            if meta.path.is_ident("name") {
                return attr.set_name(meta);
            }

            if meta.path.is_ident("json") {
                return attr.force_json(meta);
            }

            if meta.path.is_ident("context") {
                return attr.force_context(meta);
            }

            if meta.path.is_ident("this") {
                return attr.force_this(meta);
            }

            if meta.path.is_ident("async") {
                return attr.make_async(meta);
            }

            if meta.path.is_ident("task") {
                return attr.make_task(meta);
            }

            Err(meta.error(
                format!(
                    "{} [{}]",
                    ErrorCode::UnsupportedProperty.message(),
                    ErrorCode::UnsupportedProperty.code()
                ),
            ))
        });

        parser.parse2(tokens)?;

        Ok((item, attr))
    }
}
