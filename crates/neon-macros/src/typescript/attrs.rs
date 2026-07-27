//! Parses the subset of `#[serde(...)]` attributes that affect the serialized
//! shape, plus the neon-specific `#[neon(ts_type = "...")]` override. Unknown
//! serde attributes are skipped so the derive stays forward-compatible.

use super::rename::RenameRule;

/// Container-level attributes (on the struct or enum itself), e.g.
/// `#[serde(rename_all = "camelCase", tag = "kind")]` or `#[neon(ts_type = "...")]`.
pub(crate) struct ContainerAttrs {
    /// `#[serde(rename_all = "...")]` — case convention applied to every field/variant.
    pub rename_all: Option<RenameRule>,
    /// `#[serde(tag = "...")]` — internally/adjacently tagged enum discriminant field.
    pub tag: Option<String>,
    /// `#[serde(content = "...")]` — adjacently tagged enum payload field (needs `tag`).
    pub content: Option<String>,
    /// `#[serde(untagged)]` — enum union with no discriminant.
    pub untagged: bool,
    /// `#[serde(transparent)]` — struct serializes as its single field.
    pub transparent: bool,
    /// `#[neon(ts_type = "...")]` — replace the whole generated type with a literal.
    pub ts_type: Option<String>,
}

/// Field-level attributes, e.g. `#[serde(rename = "docId", default)]` on a field.
pub(crate) struct FieldAttrs {
    /// `#[serde(rename = "...")]` — this field's serialized name.
    pub rename: Option<String>,
    /// `#[serde(skip)]` / `skip_serializing` — omit the field from the type.
    pub skip: bool,
    /// `#[serde(default)]` — field is optional, rendered as `name?: T`.
    pub default: bool,
    /// `#[serde(flatten)]` — inline the field's type as an ` & T` intersection.
    pub flatten: bool,
    /// `#[neon(ts_type = "...")]` — override just this field's type.
    pub ts_type: Option<String>,
}

/// Variant-level attributes, e.g. `#[serde(rename = "dot")]` on an enum variant.
pub(crate) struct VariantAttrs {
    /// `#[serde(rename = "...")]` — this variant's serialized tag.
    pub rename: Option<String>,
    /// `#[serde(rename_all = "...")]` — case convention for this variant's fields.
    pub rename_all: Option<RenameRule>,
    /// `#[serde(skip)]` — omit the variant from the union.
    pub skip: bool,
}

impl ContainerAttrs {
    pub(crate) fn parse(attrs: &[syn::Attribute]) -> syn::Result<Self> {
        let mut result = Self {
            rename_all: None,
            tag: None,
            content: None,
            untagged: false,
            transparent: false,
            ts_type: None,
        };

        for attr in attrs {
            if attr.path().is_ident("serde") {
                attr.parse_nested_meta(|meta| {
                    if meta.path.is_ident("rename_all") {
                        let value = meta.value()?.parse::<syn::LitStr>()?;
                        result.rename_all = RenameRule::parse(&value.value());
                        if result.rename_all.is_none() {
                            return Err(meta.error(format!(
                                "unknown rename_all convention: {:?}",
                                value.value()
                            )));
                        }
                        return Ok(());
                    }

                    if meta.path.is_ident("tag") {
                        let value = meta.value()?.parse::<syn::LitStr>()?;
                        result.tag = Some(value.value());
                        return Ok(());
                    }

                    if meta.path.is_ident("content") {
                        let value = meta.value()?.parse::<syn::LitStr>()?;
                        result.content = Some(value.value());
                        return Ok(());
                    }

                    if meta.path.is_ident("untagged") {
                        result.untagged = true;
                        return Ok(());
                    }

                    if meta.path.is_ident("transparent") {
                        result.transparent = true;
                        return Ok(());
                    }

                    // Silently ignore other serde attributes for forward compatibility
                    if meta.input.peek(syn::Token![=]) {
                        meta.value()?.parse::<syn::Lit>()?;
                    } else if meta.input.peek(syn::token::Paren) {
                        meta.parse_nested_meta(|_| Ok(()))?;
                    }

                    Ok(())
                })?;
            }

            if attr.path().is_ident("neon") {
                attr.parse_nested_meta(|meta| {
                    if meta.path.is_ident("ts_type") {
                        let value = meta.value()?.parse::<syn::LitStr>()?;
                        result.ts_type = Some(value.value());
                        return Ok(());
                    }

                    Err(meta.error("unsupported neon attribute"))
                })?;
            }
        }

        Ok(result)
    }
}

impl FieldAttrs {
    pub(crate) fn parse(attrs: &[syn::Attribute]) -> syn::Result<Self> {
        let mut result = Self {
            rename: None,
            skip: false,
            default: false,
            flatten: false,
            ts_type: None,
        };

        for attr in attrs {
            if attr.path().is_ident("serde") {
                attr.parse_nested_meta(|meta| {
                    if meta.path.is_ident("rename") {
                        let value = meta.value()?.parse::<syn::LitStr>()?;
                        result.rename = Some(value.value());
                        return Ok(());
                    }

                    if meta.path.is_ident("skip") || meta.path.is_ident("skip_serializing") {
                        result.skip = true;
                        return Ok(());
                    }

                    if meta.path.is_ident("default") {
                        result.default = true;
                        return Ok(());
                    }

                    if meta.path.is_ident("flatten") {
                        result.flatten = true;
                        return Ok(());
                    }

                    // Silently ignore other serde attributes
                    if meta.input.peek(syn::Token![=]) {
                        meta.value()?.parse::<syn::Lit>()?;
                    } else if meta.input.peek(syn::token::Paren) {
                        meta.parse_nested_meta(|_| Ok(()))?;
                    }

                    Ok(())
                })?;
            }

            if attr.path().is_ident("neon") {
                attr.parse_nested_meta(|meta| {
                    if meta.path.is_ident("ts_type") {
                        let value = meta.value()?.parse::<syn::LitStr>()?;
                        result.ts_type = Some(value.value());
                        return Ok(());
                    }

                    Err(meta.error("unsupported neon attribute"))
                })?;
            }
        }

        Ok(result)
    }
}

impl VariantAttrs {
    pub(crate) fn parse(attrs: &[syn::Attribute]) -> syn::Result<Self> {
        let mut result = Self {
            rename: None,
            rename_all: None,
            skip: false,
        };

        for attr in attrs {
            if attr.path().is_ident("serde") {
                attr.parse_nested_meta(|meta| {
                    if meta.path.is_ident("rename") {
                        let value = meta.value()?.parse::<syn::LitStr>()?;
                        result.rename = Some(value.value());
                        return Ok(());
                    }

                    if meta.path.is_ident("rename_all") {
                        let value = meta.value()?.parse::<syn::LitStr>()?;
                        result.rename_all = RenameRule::parse(&value.value());
                        if result.rename_all.is_none() {
                            return Err(meta.error(format!(
                                "unknown rename_all convention: {:?}",
                                value.value()
                            )));
                        }
                        return Ok(());
                    }

                    if meta.path.is_ident("skip") {
                        result.skip = true;
                        return Ok(());
                    }

                    // Silently ignore other serde attributes
                    if meta.input.peek(syn::Token![=]) {
                        meta.value()?.parse::<syn::Lit>()?;
                    } else if meta.input.peek(syn::token::Paren) {
                        meta.parse_nested_meta(|_| Ok(()))?;
                    }

                    Ok(())
                })?;
            }
        }

        Ok(result)
    }
}
