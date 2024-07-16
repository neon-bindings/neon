#[derive(Default)]
pub(crate) struct Meta {
    pub(super) kind: Kind,
    pub(super) name: Option<syn::LitStr>,
    pub(super) json: bool,
    pub(super) context: bool,
}

#[derive(Default)]
pub(super) enum Kind {
    #[default]
    Normal,
    Task,
}

impl Meta {
    fn set_name(&mut self, meta: syn::meta::ParseNestedMeta) -> syn::Result<()> {
        self.name = Some(meta.value()?.parse::<syn::LitStr>()?);

        Ok(())
    }

    fn force_json(&mut self, _meta: syn::meta::ParseNestedMeta) -> syn::Result<()> {
        self.json = true;

        Ok(())
    }

    fn force_context(&mut self, meta: syn::meta::ParseNestedMeta) -> syn::Result<()> {
        match self.kind {
            Kind::Normal => {}
            Kind::Task => return Err(meta.error(super::TASK_CX_ERROR)),
        }

        self.context = true;

        Ok(())
    }

    fn make_task(&mut self, meta: syn::meta::ParseNestedMeta) -> syn::Result<()> {
        if self.context {
            return Err(meta.error(super::TASK_CX_ERROR));
        }

        self.kind = Kind::Task;

        Ok(())
    }
}

pub(crate) struct Parser;

impl syn::parse::Parser for Parser {
    type Output = Meta;

    fn parse2(self, tokens: proc_macro2::TokenStream) -> syn::Result<Self::Output> {
        let mut attr = Meta::default();
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

            if meta.path.is_ident("task") {
                return attr.make_task(meta);
            }

            Err(meta.error("unsupported property"))
        });

        parser.parse2(tokens)?;

        Ok(attr)
    }
}
