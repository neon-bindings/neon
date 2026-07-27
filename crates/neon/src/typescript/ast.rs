//! TypeScript AST node types.
//!
//! These types mirror [TSESTree](https://typescript-eslint.io/packages/typescript-estree/)
//! node shapes for the subset of TypeScript syntax that Neon emits in `.d.ts` files.
//! When serialized to JSON, the output is compatible with tools that consume the
//! TSESTree format (e.g. `@typescript-eslint/typescript-estree`, Prettier).
//!
//! The AST is intentionally a strict subset: we only model the declaration kinds
//! and type expressions that appear in Neon-generated declarations. Anything we
//! cannot structure escapes through [`TsType::Raw`].

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

// ───── Top-level declarations ─────

/// A top-level declaration in a `.d.ts` file. All declarations are
/// implicitly `export`-ed when rendered.
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Clone, Debug)]
#[cfg_attr(feature = "serde", serde(tag = "type"))]
pub enum Decl {
    TSDeclareFunction(TSDeclareFunction),
    TSInterfaceDeclaration(TSInterfaceDeclaration),
    TSTypeAliasDeclaration(TSTypeAliasDeclaration),
    ClassDeclaration(ClassDeclaration),
    VariableDeclaration(VariableDeclaration),
    TSModuleDeclaration(TSModuleDeclaration),
    /// Escape hatch for top-level declarations that have not yet been
    /// promoted to structured form. Renders verbatim.
    ///
    /// Future releases may promote `Raw` decls to structured variants —
    /// downstream tools should treat this as best-effort and key on the
    /// other variants when possible.
    Raw {
        value: String,
    },
}

/// A `declare module "X" { ... }` declaration, used to wrap all of a module's
/// declarations under a string-literal module name (e.g. `declare module
/// "./load.cjs" { ... }`).
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Clone, Debug)]
pub struct TSModuleDeclaration {
    /// Module name, e.g. `"./load.cjs"`. Rendered as a string literal.
    pub id: StringLiteral,
    pub body: TSModuleBlock,
    pub declare: bool,
}

#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Clone, Debug)]
pub struct TSModuleBlock {
    pub body: Vec<Decl>,
}

#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Clone, Debug)]
pub struct StringLiteral {
    pub value: String,
}

#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Clone, Debug)]
pub struct TSDeclareFunction {
    pub id: Identifier,
    pub params: Vec<Param>,
    #[cfg_attr(
        feature = "serde",
        serde(
            rename = "returnType",
            skip_serializing_if = "Option::is_none",
            default
        )
    )]
    pub return_type: Option<TSTypeAnnotation>,
}

#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Clone, Debug)]
pub struct TSInterfaceDeclaration {
    pub id: Identifier,
    #[cfg_attr(
        feature = "serde",
        serde(
            rename = "typeParameters",
            skip_serializing_if = "Option::is_none",
            default
        )
    )]
    pub type_parameters: Option<TSTypeParameterDeclaration>,
    pub body: TSInterfaceBody,
    #[cfg_attr(
        feature = "serde",
        serde(skip_serializing_if = "Vec::is_empty", default)
    )]
    pub extends: Vec<TSInterfaceHeritage>,
}

#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Clone, Debug)]
pub struct TSInterfaceBody {
    pub body: Vec<TSPropertySignature>,
}

#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Clone, Debug)]
pub struct TSInterfaceHeritage {
    pub expression: TsType,
}

#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Clone, Debug)]
pub struct TSTypeAliasDeclaration {
    pub id: Identifier,
    #[cfg_attr(
        feature = "serde",
        serde(
            rename = "typeParameters",
            skip_serializing_if = "Option::is_none",
            default
        )
    )]
    pub type_parameters: Option<TSTypeParameterDeclaration>,
    #[cfg_attr(feature = "serde", serde(rename = "typeAnnotation"))]
    pub type_annotation: TsType,
}

#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Clone, Debug)]
pub struct ClassDeclaration {
    pub id: Identifier,
    /// Always `true` for Neon-generated classes (they are `declare class`).
    pub declare: bool,
    pub body: ClassBody,
}

#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Clone, Debug)]
pub struct ClassBody {
    pub body: Vec<ClassMember>,
}

#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Clone, Debug)]
#[cfg_attr(feature = "serde", serde(tag = "type"))]
pub enum ClassMember {
    MethodDefinition(MethodDefinition),
    PropertyDefinition(PropertyDefinition),
}

#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Clone, Debug)]
pub struct MethodDefinition {
    pub key: Identifier,
    pub kind: MethodKind,
    pub value: FunctionExpression,
    #[cfg_attr(feature = "serde", serde(rename = "static"))]
    pub is_static: bool,
}

#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Clone, Debug)]
#[cfg_attr(feature = "serde", serde(rename_all = "lowercase"))]
pub enum MethodKind {
    Constructor,
    Method,
}

#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Clone, Debug)]
pub struct FunctionExpression {
    pub params: Vec<Param>,
    #[cfg_attr(
        feature = "serde",
        serde(
            rename = "returnType",
            skip_serializing_if = "Option::is_none",
            default
        )
    )]
    pub return_type: Option<TSTypeAnnotation>,
}

#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Clone, Debug)]
pub struct PropertyDefinition {
    pub key: Identifier,
    #[cfg_attr(
        feature = "serde",
        serde(
            rename = "typeAnnotation",
            skip_serializing_if = "Option::is_none",
            default
        )
    )]
    pub type_annotation: Option<TSTypeAnnotation>,
    #[cfg_attr(feature = "serde", serde(rename = "static"))]
    pub is_static: bool,
    pub readonly: bool,
}

#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Clone, Debug)]
pub struct VariableDeclaration {
    pub kind: VariableKind,
    pub declarations: Vec<VariableDeclarator>,
    /// Always `true` for `.d.ts` declarations.
    pub declare: bool,
}

#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Clone, Debug)]
#[cfg_attr(feature = "serde", serde(rename_all = "lowercase"))]
pub enum VariableKind {
    Const,
    Let,
    Var,
}

#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Clone, Debug)]
pub struct VariableDeclarator {
    pub id: Identifier,
    #[cfg_attr(
        feature = "serde",
        serde(
            rename = "typeAnnotation",
            skip_serializing_if = "Option::is_none",
            default
        )
    )]
    pub type_annotation: Option<TSTypeAnnotation>,
    #[cfg_attr(
        feature = "serde",
        serde(rename = "uniqueSymbol", skip_serializing_if = "is_false", default)
    )]
    pub unique_symbol: bool,
}

#[cfg(feature = "serde")]
fn is_false(b: &bool) -> bool {
    !b
}

// ───── Parameters and identifiers ─────

#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Clone, Debug)]
pub struct Identifier {
    pub name: String,
}

impl Identifier {
    pub fn new(name: impl Into<String>) -> Self {
        Self { name: name.into() }
    }
}

#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Clone, Debug)]
pub struct Param {
    pub name: String,
    #[cfg_attr(feature = "serde", serde(rename = "typeAnnotation"))]
    pub type_annotation: TSTypeAnnotation,
    #[cfg_attr(feature = "serde", serde(skip_serializing_if = "is_false", default))]
    pub optional: bool,
}

#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Clone, Debug)]
pub struct TSTypeAnnotation {
    #[cfg_attr(feature = "serde", serde(rename = "typeAnnotation"))]
    pub type_annotation: TsType,
}

// ───── Type parameters ─────

#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Clone, Debug)]
pub struct TSTypeParameterDeclaration {
    pub params: Vec<TSTypeParameter>,
}

#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Clone, Debug)]
pub struct TSTypeParameter {
    pub name: Identifier,
}

#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Clone, Debug)]
pub struct TSTypeParameterInstantiation {
    pub params: Vec<TsType>,
}

// ───── Type expressions ─────

/// A TypeScript type expression.
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Clone, Debug)]
#[cfg_attr(feature = "serde", serde(tag = "type"))]
pub enum TsType {
    TSStringKeyword,
    TSNumberKeyword,
    TSBooleanKeyword,
    TSAnyKeyword,
    TSVoidKeyword,
    TSUndefinedKeyword,
    TSNullKeyword,
    TSBigIntKeyword,
    TSUnknownKeyword,
    TSNeverKeyword,
    TSObjectKeyword,
    TSSymbolKeyword,
    TSArrayType(TSArrayType),
    TSUnionType(TSUnionType),
    TSIntersectionType(TSIntersectionType),
    TSLiteralType(TSLiteralType),
    TSTypeReference(TSTypeReference),
    TSTypeLiteral(TSTypeLiteral),
    TSParenthesizedType(TSParenthesizedType),
    TSTupleType(TSTupleType),
    /// Escape hatch for type expressions the parser couldn't structure.
    ///
    /// Future parser improvements may convert previously-`Raw` outputs into
    /// structured nodes — downstream tools should treat this as best-effort.
    /// To produce stable structured output, override `ts_type_ast()` directly
    /// instead of relying on the parser to round-trip.
    Raw {
        value: String,
    },
}

#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Clone, Debug)]
pub struct TSArrayType {
    #[cfg_attr(feature = "serde", serde(rename = "elementType"))]
    pub element_type: Box<TsType>,
}

#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Clone, Debug)]
pub struct TSUnionType {
    pub types: Vec<TsType>,
}

#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Clone, Debug)]
pub struct TSIntersectionType {
    pub types: Vec<TsType>,
}

#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Clone, Debug)]
pub struct TSLiteralType {
    pub literal: Literal,
}

#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Clone, Debug)]
pub struct Literal {
    #[cfg_attr(feature = "serde", serde(rename = "type"))]
    pub kind: LiteralKind,
    pub value: LiteralValue,
    #[cfg_attr(
        feature = "serde",
        serde(skip_serializing_if = "Option::is_none", default)
    )]
    pub raw: Option<String>,
}

#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Clone, Debug)]
pub enum LiteralKind {
    Literal,
}

#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Clone, Debug)]
#[cfg_attr(feature = "serde", serde(untagged))]
pub enum LiteralValue {
    Bool(bool),
    Number(f64),
    String(String),
    Null,
}

#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Clone, Debug)]
pub struct TSTypeReference {
    #[cfg_attr(feature = "serde", serde(rename = "typeName"))]
    pub type_name: TypeName,
    #[cfg_attr(
        feature = "serde",
        serde(
            rename = "typeArguments",
            skip_serializing_if = "Option::is_none",
            default
        )
    )]
    pub type_arguments: Option<TSTypeParameterInstantiation>,
}

#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Clone, Debug)]
#[cfg_attr(feature = "serde", serde(untagged))]
pub enum TypeName {
    Identifier(Identifier),
}

#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Clone, Debug)]
pub struct TSTypeLiteral {
    pub members: Vec<TSPropertySignature>,
}

#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Clone, Debug)]
pub struct TSPropertySignature {
    pub key: PropertyKey,
    #[cfg_attr(
        feature = "serde",
        serde(
            rename = "typeAnnotation",
            skip_serializing_if = "Option::is_none",
            default
        )
    )]
    pub type_annotation: Option<TSTypeAnnotation>,
    #[cfg_attr(feature = "serde", serde(skip_serializing_if = "is_false", default))]
    pub optional: bool,
    #[cfg_attr(feature = "serde", serde(skip_serializing_if = "is_false", default))]
    pub readonly: bool,
    #[cfg_attr(
        feature = "serde",
        serde(rename = "static", skip_serializing_if = "is_false", default)
    )]
    pub is_static: bool,
}

#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Clone, Debug)]
#[cfg_attr(feature = "serde", serde(untagged))]
pub enum PropertyKey {
    Identifier(Identifier),
}

#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Clone, Debug)]
pub struct TSParenthesizedType {
    #[cfg_attr(feature = "serde", serde(rename = "typeAnnotation"))]
    pub type_annotation: Box<TsType>,
}

#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Clone, Debug)]
pub struct TSTupleType {
    #[cfg_attr(feature = "serde", serde(rename = "elementTypes"))]
    pub element_types: Vec<TsType>,
}

// ───── Convenience constructors ─────

impl TsType {
    pub fn reference(name: impl Into<String>) -> TsType {
        TsType::TSTypeReference(TSTypeReference {
            type_name: TypeName::Identifier(Identifier::new(name)),
            type_arguments: None,
        })
    }

    pub fn reference_with(name: impl Into<String>, args: Vec<TsType>) -> TsType {
        TsType::TSTypeReference(TSTypeReference {
            type_name: TypeName::Identifier(Identifier::new(name)),
            type_arguments: Some(TSTypeParameterInstantiation { params: args }),
        })
    }

    pub fn array(element: TsType) -> TsType {
        TsType::TSArrayType(TSArrayType {
            element_type: Box::new(element),
        })
    }

    pub fn union(types: Vec<TsType>) -> TsType {
        TsType::TSUnionType(TSUnionType { types })
    }

    pub fn intersection(types: Vec<TsType>) -> TsType {
        TsType::TSIntersectionType(TSIntersectionType { types })
    }

    pub fn string_literal(value: impl Into<String>) -> TsType {
        let v = value.into();
        let raw = format!("\"{v}\"");
        TsType::TSLiteralType(TSLiteralType {
            literal: Literal {
                kind: LiteralKind::Literal,
                value: LiteralValue::String(v),
                raw: Some(raw),
            },
        })
    }

    pub fn raw(value: impl Into<String>) -> TsType {
        TsType::Raw {
            value: value.into(),
        }
    }
}
