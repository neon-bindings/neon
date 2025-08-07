#[derive(Copy, Clone)]
pub(crate) enum ErrorCode {
    UnsupportedExportItem,
    WrongContextChannelRef,
    ContextMustBeMut,
    WrongContextChannel,
    MustBeMutRef,
    ChannelByRef,
    ChannelContextRef,
    ContextNotAvailable,
    MissingContext,
    SelfReceiver,
    UnsupportedProperty,
    AsyncAttrAsyncFn,
}

impl ErrorCode {
    pub(crate) fn code(self) -> &'static str {
        match self {
            ErrorCode::UnsupportedExportItem => "N0001",
            ErrorCode::WrongContextChannelRef => "N0002",
            ErrorCode::ContextMustBeMut => "N0003",
            ErrorCode::WrongContextChannel => "N0004",
            ErrorCode::MustBeMutRef => "N0005",
            ErrorCode::ChannelByRef => "N0006",
            ErrorCode::ChannelContextRef => "N0007",
            ErrorCode::ContextNotAvailable => "N0008",
            ErrorCode::MissingContext => "N0009",
            ErrorCode::SelfReceiver => "N0010",
            ErrorCode::UnsupportedProperty => "N0011",
            ErrorCode::AsyncAttrAsyncFn => "N0012",
        }
    }

    pub(crate) fn message(self) -> &'static str {
        match self {
            ErrorCode::UnsupportedExportItem => "`neon::export` can only be applied to functions, consts, and statics.",
            ErrorCode::WrongContextChannelRef => "Expected `&mut Cx` instead of a `Channel` reference.",
            ErrorCode::ContextMustBeMut => "Context must be a `&mut` reference.",
            ErrorCode::WrongContextChannel => "Expected `&mut Cx` instead of `Channel`.",
            ErrorCode::MustBeMutRef => "Must be a `&mut` reference.",
            ErrorCode::ChannelByRef => "Expected an owned `Channel` instead of a reference.",
            ErrorCode::ChannelContextRef => "Expected an owned `Channel` instead of a context reference.",
            ErrorCode::ContextNotAvailable => "Context is not available in async functions. Try a `Channel` instead.",
            ErrorCode::MissingContext => "Expected a context argument. Try removing the `context` attribute.",
            ErrorCode::SelfReceiver => "Exported functions cannot receive `self`.",
            ErrorCode::UnsupportedProperty => "unsupported property",
            ErrorCode::AsyncAttrAsyncFn => "`async` attribute should not be used with an `async fn`",
        }
    }
}

pub(crate) fn error(span: proc_macro2::Span, code: ErrorCode) -> syn::Error {
    syn::Error::new(span, format!("{} [{}]", code.message(), code.code()))
}
