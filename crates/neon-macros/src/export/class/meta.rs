use syn::parse::{Parse, ParseStream};

/// Metadata for class exports
#[derive(Default)]
pub(crate) struct Meta {
    /// Name for the JavaScript class itself (used in class definition)
    pub class_name: Option<String>,
    /// Name for the module export binding
    pub export_name: Option<String>,
    /// Skip emitting TypeScript metadata for this class
    pub ts_skip: bool,
    /// Override the class name in TypeScript output (without affecting JS)
    pub ts_name: Option<String>,
    /// Require all referenced types to implement `TypeScript` (no fallback to "any")
    pub ts_strict: bool,
    /// Omit the constructor from the generated TypeScript class declaration
    pub ts_no_constructor: bool,
}

impl Parse for Meta {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        // Parse "class" token
        let class_token: syn::Ident = input.parse()?;
        if class_token != "class" {
            return Err(syn::Error::new(
                class_token.span(),
                "Expected 'class' in export attribute",
            ));
        }

        let mut meta = Meta::default();

        // Check for parenthesized attributes: class(name = "...")
        if input.peek(syn::token::Paren) {
            let content;
            syn::parenthesized!(content in input);

            // Parse attributes inside parentheses
            while !content.is_empty() {
                let name_token: syn::Ident = content.parse()?;

                match name_token.to_string().as_str() {
                    "name" => {
                        content.parse::<syn::Token![=]>()?;
                        let name_value: syn::LitStr = content.parse()?;
                        meta.class_name = Some(name_value.value());
                    }
                    "ts_skip" => {
                        meta.ts_skip = true;
                    }
                    "ts_strict" => {
                        meta.ts_strict = true;
                    }
                    "ts_no_constructor" => {
                        meta.ts_no_constructor = true;
                    }
                    "ts_name" => {
                        content.parse::<syn::Token![=]>()?;
                        let value: syn::LitStr = content.parse()?;
                        meta.ts_name = Some(value.value());
                    }
                    _ => {
                        return Err(syn::Error::new(
                            name_token.span(),
                            format!("Unknown class attribute '{}'", name_token),
                        ));
                    }
                }

                // Parse optional comma
                if content.parse::<syn::Token![,]>().is_err() {
                    break;
                }
            }
        }

        // Check if there are additional attributes after "class" or "class(...)"
        if input.parse::<syn::Token![,]>().is_ok() {
            // Parse additional attributes like name = "..."
            while !input.is_empty() {
                let name_token: syn::Ident = input.parse()?;

                match name_token.to_string().as_str() {
                    "name" => {
                        input.parse::<syn::Token![=]>()?;
                        let name_value: syn::LitStr = input.parse()?;
                        meta.export_name = Some(name_value.value());
                    }
                    "ts_skip" => {
                        meta.ts_skip = true;
                    }
                    "ts_strict" => {
                        meta.ts_strict = true;
                    }
                    "ts_no_constructor" => {
                        meta.ts_no_constructor = true;
                    }
                    "ts_name" => {
                        input.parse::<syn::Token![=]>()?;
                        let value: syn::LitStr = input.parse()?;
                        meta.ts_name = Some(value.value());
                    }
                    _ => {
                        return Err(syn::Error::new(
                            name_token.span(),
                            format!("Unknown attribute '{}'", name_token),
                        ));
                    }
                }

                // Parse optional comma
                if input.parse::<syn::Token![,]>().is_err() {
                    break;
                }
            }
        }

        Ok(meta)
    }
}

/// Parser for class export metadata
pub(crate) struct Parser;

impl syn::parse::Parser for Parser {
    type Output = Meta;

    fn parse2(self, tokens: proc_macro2::TokenStream) -> syn::Result<Self::Output> {
        syn::parse2(tokens)
    }
}
