use syn::parse::{Parse, ParseStream};

/// Metadata for class exports
#[derive(Default)]
pub(crate) struct Meta {
    /// Name for the JavaScript class itself (used in class definition)
    pub class_name: Option<String>,
    /// Name for the module export binding
    pub export_name: Option<String>,
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
