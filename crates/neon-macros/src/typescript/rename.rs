/// Serde rename conventions.
#[derive(Clone, Copy)]
pub(crate) enum RenameRule {
    CamelCase,
    SnakeCase,
    ScreamingSnakeCase,
    KebabCase,
    PascalCase,
    Lowercase,
    Uppercase,
}

impl RenameRule {
    pub(crate) fn parse(s: &str) -> Option<Self> {
        match s {
            "camelCase" => Some(Self::CamelCase),
            "snake_case" => Some(Self::SnakeCase),
            "SCREAMING_SNAKE_CASE" => Some(Self::ScreamingSnakeCase),
            "kebab-case" => Some(Self::KebabCase),
            "PascalCase" => Some(Self::PascalCase),
            "lowercase" => Some(Self::Lowercase),
            "UPPERCASE" => Some(Self::Uppercase),
            _ => None,
        }
    }

    pub(crate) fn apply(self, name: &str) -> String {
        match self {
            Self::Lowercase => name.to_lowercase(),
            Self::Uppercase => name.to_uppercase(),
            Self::CamelCase => to_camel_case(name),
            Self::SnakeCase => to_snake_case(name),
            Self::ScreamingSnakeCase => to_snake_case(name).to_uppercase(),
            Self::KebabCase => to_snake_case(name).replace('_', "-"),
            Self::PascalCase => to_pascal_case(name),
        }
    }
}

/// Split a name into words, handling both snake_case and PascalCase inputs.
fn split_words(name: &str) -> Vec<String> {
    let mut words = Vec::new();
    let mut current = String::new();

    for ch in name.chars() {
        if ch == '_' {
            if !current.is_empty() {
                words.push(current.clone());
                current.clear();
            }
        } else if ch.is_uppercase()
            && !current.is_empty()
            && current.chars().last().map_or(false, |c| c.is_lowercase())
        {
            words.push(current.clone());
            current.clear();
            current.push(ch);
        } else {
            current.push(ch);
        }
    }

    if !current.is_empty() {
        words.push(current);
    }

    words
}

fn to_camel_case(name: &str) -> String {
    let words = split_words(name);
    let mut out = String::new();

    for (i, word) in words.iter().enumerate() {
        if i == 0 {
            out.push_str(&word.to_lowercase());
        } else {
            let mut chars = word.chars();
            if let Some(first) = chars.next() {
                out.extend(first.to_uppercase());
                out.push_str(&chars.as_str().to_lowercase());
            }
        }
    }

    out
}

fn to_snake_case(name: &str) -> String {
    let words = split_words(name);
    words
        .iter()
        .map(|w| w.to_lowercase())
        .collect::<Vec<_>>()
        .join("_")
}

fn to_pascal_case(name: &str) -> String {
    let words = split_words(name);
    let mut out = String::new();

    for word in &words {
        let mut chars = word.chars();
        if let Some(first) = chars.next() {
            out.extend(first.to_uppercase());
            out.push_str(&chars.as_str().to_lowercase());
        }
    }

    out
}

#[cfg(test)]
mod tests {
    use super::RenameRule;

    #[test]
    fn camel_case_from_snake() {
        assert_eq!(RenameRule::CamelCase.apply("doc_id"), "docId");
        assert_eq!(
            RenameRule::CamelCase.apply("some_field_name"),
            "someFieldName"
        );
    }

    #[test]
    fn camel_case_from_pascal() {
        assert_eq!(RenameRule::CamelCase.apply("DocId"), "docId");
        assert_eq!(RenameRule::CamelCase.apply("Text"), "text");
    }

    #[test]
    fn snake_case_from_pascal() {
        assert_eq!(RenameRule::SnakeCase.apply("DocId"), "doc_id");
        assert_eq!(RenameRule::SnakeCase.apply("Text"), "text");
    }

    #[test]
    fn screaming_snake() {
        assert_eq!(RenameRule::ScreamingSnakeCase.apply("doc_id"), "DOC_ID");
        assert_eq!(RenameRule::ScreamingSnakeCase.apply("DocId"), "DOC_ID");
    }

    #[test]
    fn kebab_case() {
        assert_eq!(RenameRule::KebabCase.apply("doc_id"), "doc-id");
        assert_eq!(RenameRule::KebabCase.apply("DocId"), "doc-id");
    }

    #[test]
    fn pascal_case() {
        assert_eq!(RenameRule::PascalCase.apply("doc_id"), "DocId");
        assert_eq!(RenameRule::PascalCase.apply("some_field"), "SomeField");
    }

    #[test]
    fn lowercase_uppercase() {
        assert_eq!(RenameRule::Lowercase.apply("Text"), "text");
        assert_eq!(RenameRule::Uppercase.apply("Text"), "TEXT");
    }

    #[test]
    fn parse_rejects_unknown_rule() {
        // Unknown rename_all values must return None so the attribute parser
        // can surface a helpful error rather than silently using a wrong rule.
        assert!(RenameRule::parse("camelcase").is_none());
        assert!(RenameRule::parse("CAMEL_CASE").is_none());
        assert!(RenameRule::parse("").is_none());
    }

    #[test]
    fn parse_recognizes_all_known_rules() {
        assert!(RenameRule::parse("camelCase").is_some());
        assert!(RenameRule::parse("snake_case").is_some());
        assert!(RenameRule::parse("SCREAMING_SNAKE_CASE").is_some());
        assert!(RenameRule::parse("kebab-case").is_some());
        assert!(RenameRule::parse("PascalCase").is_some());
        assert!(RenameRule::parse("lowercase").is_some());
        assert!(RenameRule::parse("UPPERCASE").is_some());
    }
}
