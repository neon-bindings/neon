//! String → AST parser for TypeScript type expressions.
//!
//! Used as the default impl of [`TypeScript::ts_type_ast`](crate::typescript::TypeScript::ts_type_ast)
//! so that existing `TypeScript` impls (which return strings via `ts_type`) still
//! produce structured AST output. Anything the parser can't handle falls through
//! to [`TsType::Raw`].
//!
//! This is a minimal recursive-descent parser covering the subset of TS type
//! syntax that Neon's own codegen emits. It is not a complete TS parser.

use super::ast::*;

/// Parse a TypeScript type expression. On failure, returns `TsType::Raw`.
pub fn parse(input: &str) -> TsType {
    let mut p = Parser::new(input);
    match p.parse_union() {
        Some(ty) if p.is_done() => ty,
        _ => TsType::raw(input),
    }
}

struct Parser<'a> {
    src: &'a str,
    pos: usize,
}

impl<'a> Parser<'a> {
    fn new(src: &'a str) -> Self {
        Self { src, pos: 0 }
    }

    fn is_done(&mut self) -> bool {
        self.skip_ws();
        self.pos >= self.src.len()
    }

    fn skip_ws(&mut self) {
        while self.pos < self.src.len() && self.src.as_bytes()[self.pos].is_ascii_whitespace() {
            self.pos += 1;
        }
    }

    fn peek(&mut self) -> Option<u8> {
        self.skip_ws();
        self.src.as_bytes().get(self.pos).copied()
    }

    fn peek_raw(&self) -> Option<u8> {
        self.src.as_bytes().get(self.pos).copied()
    }

    fn eat(&mut self, c: u8) -> bool {
        if self.peek() == Some(c) {
            self.pos += 1;
            true
        } else {
            false
        }
    }

    fn eat_str(&mut self, s: &str) -> bool {
        self.skip_ws();
        if self.src[self.pos..].starts_with(s) {
            // Ensure word boundary if s ends in an identifier character.
            let end = self.pos + s.len();
            if s.as_bytes().last().is_some_and(|c| is_ident_continue(*c))
                && end < self.src.len()
                && is_ident_continue(self.src.as_bytes()[end])
            {
                return false;
            }
            self.pos = end;
            true
        } else {
            false
        }
    }

    // ───── union := intersection ('|' intersection)* ─────
    fn parse_union(&mut self) -> Option<TsType> {
        let first = self.parse_intersection()?;
        let mut types = vec![first];
        while self.eat(b'|') {
            types.push(self.parse_intersection()?);
        }
        Some(if types.len() == 1 {
            types.pop().unwrap()
        } else {
            TsType::union(types)
        })
    }

    // ───── intersection := postfix ('&' postfix)* ─────
    fn parse_intersection(&mut self) -> Option<TsType> {
        let first = self.parse_postfix()?;
        let mut types = vec![first];
        while self.eat(b'&') {
            types.push(self.parse_postfix()?);
        }
        Some(if types.len() == 1 {
            types.pop().unwrap()
        } else {
            TsType::intersection(types)
        })
    }

    // ───── postfix := atom ('[' ']')* ─────
    fn parse_postfix(&mut self) -> Option<TsType> {
        let mut ty = self.parse_atom()?;
        loop {
            self.skip_ws();
            if self.pos + 1 < self.src.len()
                && self.src.as_bytes()[self.pos] == b'['
                && self.next_non_ws_after(self.pos + 1) == Some(b']')
            {
                // Eat '['
                self.pos += 1;
                self.skip_ws();
                // Eat ']'
                self.pos += 1;
                ty = TsType::array(ty);
            } else {
                break;
            }
        }
        Some(ty)
    }

    fn next_non_ws_after(&self, mut idx: usize) -> Option<u8> {
        while let Some(&b) = self.src.as_bytes().get(idx) {
            if !b.is_ascii_whitespace() {
                return Some(b);
            }
            idx += 1;
        }
        None
    }

    // ───── atom := primitive | string-literal | identifier (typeArgs)? | '(' union ')' | '{' members '}'
    fn parse_atom(&mut self) -> Option<TsType> {
        self.skip_ws();
        let c = self.peek_raw()?;

        if c == b'(' {
            self.pos += 1;
            let inner = self.parse_union()?;
            self.skip_ws();
            if !self.eat(b')') {
                return None;
            }
            return Some(TsType::TSParenthesizedType(TSParenthesizedType {
                type_annotation: Box::new(inner),
            }));
        }

        if c == b'{' {
            return self.parse_type_literal();
        }

        if c == b'[' {
            return self.parse_tuple();
        }

        if c == b'"' || c == b'\'' {
            return self.parse_string_literal(c);
        }

        // Try keywords first (before generic identifier so e.g. `string` becomes TSStringKeyword)
        if let Some(kw) = self.try_keyword() {
            return Some(kw);
        }

        // Identifier (with optional type arguments)
        if is_ident_start(c) {
            let name = self.parse_identifier()?;
            self.skip_ws();
            if self.peek_raw() == Some(b'<') {
                self.pos += 1;
                let args = self.parse_type_arg_list()?;
                if !self.eat(b'>') {
                    return None;
                }
                return Some(TsType::reference_with(name, args));
            }
            return Some(TsType::reference(name));
        }

        None
    }

    fn try_keyword(&mut self) -> Option<TsType> {
        const KEYWORDS: &[(&str, fn() -> TsType)] = &[
            ("string", || TsType::TSStringKeyword),
            ("number", || TsType::TSNumberKeyword),
            ("boolean", || TsType::TSBooleanKeyword),
            ("any", || TsType::TSAnyKeyword),
            ("void", || TsType::TSVoidKeyword),
            ("undefined", || TsType::TSUndefinedKeyword),
            ("null", || TsType::TSNullKeyword),
            ("bigint", || TsType::TSBigIntKeyword),
            ("unknown", || TsType::TSUnknownKeyword),
            ("never", || TsType::TSNeverKeyword),
            ("object", || TsType::TSObjectKeyword),
            ("symbol", || TsType::TSSymbolKeyword),
            ("true", || {
                TsType::TSLiteralType(TSLiteralType {
                    literal: Literal {
                        kind: LiteralKind::Literal,
                        value: LiteralValue::Bool(true),
                        raw: Some("true".into()),
                    },
                })
            }),
            ("false", || {
                TsType::TSLiteralType(TSLiteralType {
                    literal: Literal {
                        kind: LiteralKind::Literal,
                        value: LiteralValue::Bool(false),
                        raw: Some("false".into()),
                    },
                })
            }),
        ];
        let saved = self.pos;
        for (kw, build) in KEYWORDS {
            if self.eat_str(kw) {
                return Some(build());
            }
            self.pos = saved;
        }
        None
    }

    fn parse_identifier(&mut self) -> Option<String> {
        self.skip_ws();
        let start = self.pos;
        let bytes = self.src.as_bytes();
        if !bytes.get(start).copied().is_some_and(is_ident_start) {
            return None;
        }
        let mut end = start + 1;
        while end < bytes.len() && is_ident_continue(bytes[end]) {
            end += 1;
        }
        self.pos = end;
        Some(self.src[start..end].to_string())
    }

    fn parse_type_arg_list(&mut self) -> Option<Vec<TsType>> {
        let mut args = Vec::new();
        loop {
            self.skip_ws();
            if self.peek_raw() == Some(b'>') {
                break;
            }
            args.push(self.parse_union()?);
            self.skip_ws();
            if !self.eat(b',') {
                break;
            }
        }
        Some(args)
    }

    fn parse_string_literal(&mut self, quote: u8) -> Option<TsType> {
        // Consume opening quote.
        self.pos += 1;
        let start = self.pos;
        let bytes = self.src.as_bytes();
        while self.pos < bytes.len() && bytes[self.pos] != quote {
            // No escape handling: Neon-emitted strings are simple.
            self.pos += 1;
        }
        if self.pos >= bytes.len() {
            return None;
        }
        let value = self.src[start..self.pos].to_string();
        self.pos += 1; // closing quote
        let raw = format!("{}{}{}", quote as char, value, quote as char);
        Some(TsType::TSLiteralType(TSLiteralType {
            literal: Literal {
                kind: LiteralKind::Literal,
                value: LiteralValue::String(value),
                raw: Some(raw),
            },
        }))
    }

    // ───── tuple := '[' (union (',' union)*)? ']' ─────
    fn parse_tuple(&mut self) -> Option<TsType> {
        // consume '['
        self.pos += 1;
        let mut elements = Vec::new();
        loop {
            self.skip_ws();
            if self.peek_raw() == Some(b']') {
                self.pos += 1;
                break;
            }
            elements.push(self.parse_union()?);
            self.skip_ws();
            if !self.eat(b',') {
                self.skip_ws();
                if !self.eat(b']') {
                    return None;
                }
                break;
            }
        }
        Some(TsType::TSTupleType(TSTupleType {
            element_types: elements,
        }))
    }

    // ───── type-literal := '{' member (';' | ',') ... '}' ─────
    fn parse_type_literal(&mut self) -> Option<TsType> {
        // Consume '{'
        self.pos += 1;
        let mut members = Vec::new();
        loop {
            self.skip_ws();
            if self.peek_raw() == Some(b'}') {
                self.pos += 1;
                break;
            }
            let readonly = self.eat_str("readonly");
            let key = self.parse_identifier()?;
            self.skip_ws();
            let optional = self.eat(b'?');
            self.skip_ws();
            if !self.eat(b':') {
                return None;
            }
            let ty = self.parse_union()?;
            members.push(TSPropertySignature {
                key: PropertyKey::Identifier(Identifier::new(key)),
                type_annotation: Some(TSTypeAnnotation {
                    type_annotation: ty,
                }),
                optional,
                readonly,
                is_static: false,
            });
            self.skip_ws();
            // Accept ';' or ',' as separator (with optional trailing one before '}')
            if !self.eat(b';') {
                let _ = self.eat(b',');
            }
        }
        Some(TsType::TSTypeLiteral(TSTypeLiteral { members }))
    }
}

fn is_ident_start(c: u8) -> bool {
    c.is_ascii_alphabetic() || c == b'_' || c == b'$'
}

fn is_ident_continue(c: u8) -> bool {
    c.is_ascii_alphanumeric() || c == b'_' || c == b'$'
}

#[cfg(test)]
mod tests {
    use super::*;

    fn p(s: &str) -> TsType {
        parse(s)
    }

    #[test]
    fn primitives() {
        assert!(matches!(p("string"), TsType::TSStringKeyword));
        assert!(matches!(p("number"), TsType::TSNumberKeyword));
        assert!(matches!(p("any"), TsType::TSAnyKeyword));
        assert!(matches!(p("bigint"), TsType::TSBigIntKeyword));
    }

    #[test]
    fn identifier_reference() {
        let t = p("Foo");
        match t {
            TsType::TSTypeReference(r) => {
                let TypeName::Identifier(id) = r.type_name;
                assert_eq!(id.name, "Foo");
                assert!(r.type_arguments.is_none());
            }
            other => panic!("not a reference: {:?}", other),
        }
    }

    #[test]
    fn generic_reference() {
        let t = p("Promise<number>");
        match t {
            TsType::TSTypeReference(r) => {
                let TypeName::Identifier(id) = r.type_name;
                assert_eq!(id.name, "Promise");
                let args = r.type_arguments.unwrap();
                assert_eq!(args.params.len(), 1);
                assert!(matches!(args.params[0], TsType::TSNumberKeyword));
            }
            _ => panic!("not a reference"),
        }
    }

    #[test]
    fn array_postfix() {
        let t = p("number[]");
        match t {
            TsType::TSArrayType(a) => assert!(matches!(*a.element_type, TsType::TSNumberKeyword)),
            _ => panic!("not array"),
        }
    }

    #[test]
    fn union() {
        let t = p("string | number | undefined");
        match t {
            TsType::TSUnionType(u) => assert_eq!(u.types.len(), 3),
            _ => panic!("not union"),
        }
    }

    #[test]
    fn intersection() {
        let t = p("Foo & Bar");
        match t {
            TsType::TSIntersectionType(i) => assert_eq!(i.types.len(), 2),
            _ => panic!("not intersection"),
        }
    }

    #[test]
    fn type_literal() {
        let t = p("{ x: number; y: string }");
        match t {
            TsType::TSTypeLiteral(l) => {
                assert_eq!(l.members.len(), 2);
            }
            _ => panic!("not type literal"),
        }
    }

    #[test]
    fn string_literal() {
        let t = p("\"foo\"");
        match t {
            TsType::TSLiteralType(l) => match l.literal.value {
                LiteralValue::String(s) => assert_eq!(s, "foo"),
                _ => panic!("not string"),
            },
            _ => panic!("not literal"),
        }
    }

    #[test]
    fn tuple() {
        let t = p("[number, string]");
        match t {
            TsType::TSTupleType(tup) => {
                assert_eq!(tup.element_types.len(), 2);
                assert!(matches!(tup.element_types[0], TsType::TSNumberKeyword));
                assert!(matches!(tup.element_types[1], TsType::TSStringKeyword));
            }
            _ => panic!("not tuple"),
        }
    }

    #[test]
    fn unparseable_falls_through_to_raw() {
        let t = p("???not a type???");
        assert!(matches!(t, TsType::Raw { .. }));
    }

    /// Probe parsing on the type strings that Neon's own codegen produces.
    /// This catches real bugs (e.g. nested generics, paren-arrays) rather than
    /// just exercising untested branches.
    #[test]
    fn probe_real_codegen_strings() {
        // Each case is the string our built-in `TypeScript` impls actually emit.
        // Failures here mean the parser does not round-trip what we generate.
        let cases: &[(&str, fn(&TsType) -> bool, &str)] = &[
            // Vec<Vec<T>> → "T[][]"
            (
                "number[][]",
                |t| {
                    matches!(t, TsType::TSArrayType(a)
                    if matches!(&*a.element_type, TsType::TSArrayType(_)))
                },
                "nested array",
            ),
            // Vec<Option<T>> → "(T | undefined | null)[]"
            (
                "(string | undefined | null)[]",
                |t| {
                    matches!(t, TsType::TSArrayType(a)
                    if matches!(&*a.element_type, TsType::TSParenthesizedType(_)))
                },
                "paren-wrapped union array",
            ),
            // Option<T> → "T | undefined | null"
            (
                "string | undefined | null",
                |t| matches!(t, TsType::TSUnionType(u) if u.types.len() == 3),
                "Option<T>",
            ),
            // HashMap → "Record<K, V>"
            (
                "Record<string, number>",
                |t| {
                    matches!(t, TsType::TSTypeReference(r)
                    if r.type_arguments.as_ref().is_some_and(|a| a.params.len() == 2))
                },
                "two-arg generic Record",
            ),
            // HashMap<String, HashMap<...>> → nested generics
            (
                "Record<string, Record<string, number>>",
                |t| matches!(t, TsType::TSTypeReference(_)),
                "nested generic",
            ),
            // Tuple impls → "[A, B]" and "[A, B, C]"
            (
                "[number, string]",
                |t| matches!(t, TsType::TSTupleType(t) if t.element_types.len() == 2),
                "2-tuple",
            ),
            (
                "[number, string, boolean]",
                |t| matches!(t, TsType::TSTupleType(t) if t.element_types.len() == 3),
                "3-tuple",
            ),
            // Async return → "Promise<T>"
            (
                "Promise<number>",
                |t| {
                    matches!(t, TsType::TSTypeReference(r)
                    if matches!(&r.type_name, TypeName::Identifier(i) if i.name == "Promise"))
                },
                "Promise wrap",
            ),
            // Branded interface body uses single-quote literal in `[__neon_tag]`
            // (we don't expose this to parser, but single-quote literals should work)
            (
                "'foo'",
                |t| matches!(t, TsType::TSLiteralType(_)),
                "single-quote literal",
            ),
            // Vec<(A, B)> → "[A, B][]" (tuple followed by array postfix)
            (
                "[number, string][]",
                |t| {
                    matches!(t, TsType::TSArrayType(a)
                    if matches!(&*a.element_type, TsType::TSTupleType(_)))
                },
                "tuple in array",
            ),
        ];

        for (input, predicate, desc) in cases {
            let parsed = p(input);
            assert!(
                predicate(&parsed),
                "real-codegen probe failed: {desc} ({input:?}) parsed as {parsed:?}"
            );
        }
    }

    /// Inputs that the parser should NOT match (must reach end of input, etc.)
    /// fall through to Raw.
    #[test]
    fn malformed_inputs_fall_through_to_raw() {
        // Trailing garbage after a valid type → must be Raw, not partial parse.
        assert!(matches!(p("number trailing"), TsType::Raw { .. }));
        // Unmatched parens
        assert!(matches!(p("(string"), TsType::Raw { .. }));
        // Unterminated generic
        assert!(matches!(p("Foo<string"), TsType::Raw { .. }));
        // Empty input
        assert!(matches!(p(""), TsType::Raw { .. }));
    }

    /// `eat_str` should respect word boundaries for keywords: `numbery` is an
    /// identifier, not a number-keyword followed by trailing chars.
    #[test]
    fn keyword_word_boundary() {
        match p("numbery") {
            TsType::TSTypeReference(r) => {
                let TypeName::Identifier(id) = r.type_name;
                assert_eq!(id.name, "numbery");
            }
            other => panic!("expected identifier reference, got {other:?}"),
        }
    }

    /// Type literals support both `;` and `,` separators, optional/readonly,
    /// and arbitrary whitespace.
    #[test]
    fn type_literal_variants() {
        // Comma-separated
        let t = p("{ x: number, y: string }");
        assert!(matches!(&t, TsType::TSTypeLiteral(l) if l.members.len() == 2));

        // Optional property
        let t = p("{ x?: number }");
        match &t {
            TsType::TSTypeLiteral(l) => {
                assert_eq!(l.members.len(), 1);
                assert!(l.members[0].optional);
            }
            _ => panic!("not a type literal"),
        }

        // Readonly property
        let t = p("{ readonly x: number }");
        match &t {
            TsType::TSTypeLiteral(l) => {
                assert_eq!(l.members.len(), 1);
                assert!(l.members[0].readonly);
            }
            _ => panic!("not a type literal"),
        }
    }
}
