/// Parsed contents of a `#[neon::export(...)]` attribute on a function, e.g.
/// `#[neon::export(name = "add", json, ts_returns = "bigint")]`.
#[derive(Default)]
pub(crate) struct Meta {
    /// Sync vs `async`/task — determines whether the TS return is `Promise<T>`.
    pub(super) kind: Kind,
    /// `name = "..."` — the JavaScript export name (also the TS declaration name).
    pub(super) name: Option<syn::LitStr>,
    /// `json` — arguments/return are (de)serialized via `Json<T>`.
    pub(super) json: bool,
    /// `context` — force-pass the `Cx`/`Channel` first argument.
    pub(super) context: bool,
    /// `this` — bind the receiver as JS `this`.
    pub(super) this: bool,
    /// `ts_skip` — omit this export from the generated `.d.ts`.
    pub(super) ts_skip: bool,
    /// `ts_strict` — error (instead of falling back to `any`) if a referenced
    /// type doesn't implement `TypeScript`.
    pub(super) ts_strict: bool,
    /// `ts_returns = "..."` — override the inferred TS return type with a literal.
    pub(super) ts_returns: Option<String>,
}

/// How the exported function is invoked, which drives the TS return shape.
#[derive(Default)]
pub(super) enum Kind {
    /// `#[neon::export(async)]` on a sync fn returning a future → `Promise<T>`.
    Async,
    /// A plain `async fn` → `Promise<T>`.
    AsyncFn,
    /// An ordinary synchronous export → `T`.
    #[default]
    Normal,
    /// `#[neon::export(task)]` runs on the libuv pool → `Promise<T>`.
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
            return Err(meta.error("`async` attribute should not be used with an `async fn`"));
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

            if meta.path.is_ident("ts_skip") {
                attr.ts_skip = true;
                return Ok(());
            }

            if meta.path.is_ident("ts_strict") {
                attr.ts_strict = true;
                return Ok(());
            }

            if meta.path.is_ident("ts_returns") {
                attr.ts_returns = Some(meta.value()?.parse::<syn::LitStr>()?.value());
                return Ok(());
            }

            Err(meta.error("unsupported property"))
        });

        parser.parse2(tokens)?;

        Ok((item, attr))
    }
}
