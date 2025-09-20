// Convert identifiers to camel case with the following rules:
// * All leading and trailing underscores are preserved
// * All other underscores are removed
// * Characters immediately following a non-leading underscore are uppercased
// * Bail (no conversion) if an unexpected condition is encountered:
//   - Uppercase character
//   - More than one adjacent interior underscore
pub(crate) fn to_camel_case(name: &str) -> String {
    let mut out = String::with_capacity(name.len());
    let mut it = name.chars();
    let mut next = it.next();
    let mut count = 0usize;

    // Keep leading underscores
    while matches!(next, Some('_')) {
        out.push('_');
        next = it.next();
    }

    // Convert to camel case
    while let Some(c) = next {
        match c {
            // Keep a count for maintaining trailing underscores
            '_' => count += 1,

            // Bail if there is an unexpected uppercase character or extra underscore
            _ if c.is_uppercase() || count >= 2 => {
                return name.to_string();
            }

            // Don't uppercase the middle of a word
            _ if count == 0 => {
                out.push(c);
                count = 0;
            }

            // Uppercase characters following an underscore
            _ => {
                out.extend(c.to_uppercase());
                count = 0;
            }
        }

        next = it.next();
    }

    // We don't know underscores are a suffix until iteration has completed;
    // add them back.
    for _ in 0..count {
        out.push('_');
    }

    out
}

#[cfg(test)]
mod test {
    #[test]
    fn to_camel_case() {
        use super::to_camel_case;

        assert_eq!(to_camel_case(""), "");
        assert_eq!(to_camel_case("one"), "one");
        assert_eq!(to_camel_case("two_words"), "twoWords");
        assert_eq!(to_camel_case("three_word_name"), "threeWordName");
        assert_eq!(to_camel_case("extra__underscore"), "extra__underscore");
        assert_eq!(to_camel_case("PreserveCase"), "PreserveCase");
        assert_eq!(to_camel_case("PreServe_case"), "PreServe_case");
        assert_eq!(to_camel_case("_preserve_leading"), "_preserveLeading");
        assert_eq!(to_camel_case("__preserve_leading"), "__preserveLeading");
        assert_eq!(to_camel_case("preserve_trailing_"), "preserveTrailing_");
        assert_eq!(to_camel_case("preserve_trailing__"), "preserveTrailing__");
        assert_eq!(to_camel_case("_preserve_both_"), "_preserveBoth_");
        assert_eq!(to_camel_case("__preserve_both__"), "__preserveBoth__");
        assert_eq!(to_camel_case("_"), "_");
        assert_eq!(to_camel_case("__"), "__");
        assert_eq!(to_camel_case("___"), "___");
    }
}
