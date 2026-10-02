//! The `TypeScript` trait: Neon's minimal, stable contract for mapping a Rust
//! type to its TypeScript representation. Deliberately tiny and dependency-free.

use std::borrow::Cow;
use std::cell::{Ref, RefCell, RefMut};
use std::collections::{BTreeMap, BTreeSet, HashMap, HashSet};
use std::hash::Hash;
use std::marker::PhantomData;
use std::rc::Rc;
use std::sync::Arc;

/// A Rust type with a known TypeScript representation.
pub trait TypeScript {
    /// The TypeScript type *expression* for this type (e.g. `"number"`,
    /// `"string[]"`, `"SearchResult"`).
    fn ts_type() -> Cow<'static, str>;

    /// An optional top-level declaration this type needs (an interface, type
    /// alias, or `declare class`). `None` for primitives.
    fn ts_decl() -> Option<Cow<'static, str>> {
        None
    }

    /// Add this type's own declaration plus its transitive declarations to
    /// `decls`, keyed by name for dedup. Composite types override this to also
    /// collect from their component types.
    fn ts_collect(decls: &mut BTreeMap<String, String>) {
        if let Some(d) = Self::ts_decl() {
            decls
                .entry(Self::ts_type().into_owned())
                .or_insert_with(|| d.into_owned());
        }
    }
}

// ——— TypeScript autoref specialization probe ———
//
// This enables macro-generated metadata to gracefully handle types that don't
// implement `TypeScript`. Method resolution is a three-rung ladder tried
// most-specific first (see `doc/typescript.md`):
//
//   Rung 1: inherent method on `TsProbe<T>` (requires `T: TypeScript`).
//   Rung 2: an adapter's trait `impl … for TsProbe<T>` (e.g. `neon-ts-rs`'s
//           `TypeScriptExt`), only visible when that trait is `use`d in scope.
//   Rung 3: the `TsFallback` trait impl for `&TsProbe<T>`, reached via autoref
//           for all `T`, yielding `"any"`.
//
// The probe lives here (not in `neon`) so an adapter crate can hang rung 2 on it.
//
// Usage in macro-generated code:
//   let __probe = TsProbe::<SomeType>(PhantomData);
//   (&__probe).ts_type_of()

/// Compile-time probe for whether a type implements `TypeScript`.
pub struct TsProbe<T>(pub PhantomData<T>);

// Higher priority: inherent method, available only when T: TypeScript
impl<T: TypeScript> TsProbe<T> {
    pub fn ts_type_of(&self) -> Cow<'static, str> {
        T::ts_type()
    }

    pub fn ts_collect_of(&self, decls: &mut BTreeMap<String, String>) {
        T::ts_collect(decls);
    }
}

/// Lower-priority fallback for types that don't implement `TypeScript`.
/// Reached via autoref (`(&probe).method()` resolves to `&TsProbe<T>` first,
/// then falls through to `&&TsProbe<T>` which matches this trait impl).
pub trait TsFallback {
    fn ts_type_of(&self) -> Cow<'static, str>;
    fn ts_collect_of(&self, decls: &mut BTreeMap<String, String>);
}

impl<T> TsFallback for &TsProbe<T> {
    fn ts_type_of(&self) -> Cow<'static, str> {
        "any".into()
    }

    fn ts_collect_of(&self, _: &mut BTreeMap<String, String>) {}
}

// ——— Primitive impls ———

macro_rules! ts_primitive {
    ($ty:ty, $ts:literal) => {
        impl TypeScript for $ty {
            fn ts_type() -> Cow<'static, str> {
                $ts.into()
            }
        }
    };
}

ts_primitive!(f64, "number");
ts_primitive!(f32, "number");
ts_primitive!(i64, "number");
ts_primitive!(i32, "number");
ts_primitive!(i16, "number");
ts_primitive!(i8, "number");
ts_primitive!(isize, "number");
ts_primitive!(u64, "number");
ts_primitive!(u32, "number");
ts_primitive!(u16, "number");
ts_primitive!(u8, "number");
ts_primitive!(usize, "number");
ts_primitive!(String, "string");
ts_primitive!(&str, "string");
ts_primitive!(bool, "boolean");
ts_primitive!((), "undefined");

// ——— Wrapper impls ———

impl<T: TypeScript> TypeScript for Option<T> {
    fn ts_type() -> Cow<'static, str> {
        let inner = T::ts_type();
        format!("{inner} | null").into()
    }

    fn ts_collect(decls: &mut BTreeMap<String, String>) {
        T::ts_collect(decls);
    }
}

impl<T: TypeScript> TypeScript for Vec<T> {
    fn ts_type() -> Cow<'static, str> {
        let inner = T::ts_type();
        // Wrap union types in parens: (Foo | Bar)[] not Foo | Bar[]
        if inner.contains(" | ") {
            format!("({inner})[]").into()
        } else {
            format!("{inner}[]").into()
        }
    }

    fn ts_collect(decls: &mut BTreeMap<String, String>) {
        T::ts_collect(decls);
    }
}

impl<T: TypeScript + Eq + Hash> TypeScript for HashSet<T> {
    fn ts_type() -> Cow<'static, str> {
        Vec::<T>::ts_type()
    }

    fn ts_collect(decls: &mut BTreeMap<String, String>) {
        T::ts_collect(decls);
    }
}

impl<T: TypeScript + Ord> TypeScript for BTreeSet<T> {
    fn ts_type() -> Cow<'static, str> {
        Vec::<T>::ts_type()
    }

    fn ts_collect(decls: &mut BTreeMap<String, String>) {
        T::ts_collect(decls);
    }
}

// Maps serialize to JSON objects whose keys are *always* strings, regardless of
// the Rust key type `K` (serde stringifies keys, and JSON has no other kind of
// object key). So the observable TypeScript shape is always `Record<string, V>`.
// We deliberately ignore `K` here: describing a `HashMap<u32, V>` as
// `Record<number, V>` would be misleading (the runtime keys are strings), and
// non-primitive `K` would produce invalid TypeScript (`Record` keys must be
// `string | number | symbol`).
impl<K: TypeScript, V: TypeScript> TypeScript for HashMap<K, V> {
    fn ts_type() -> Cow<'static, str> {
        let v = V::ts_type();
        format!("Record<string, {v}>").into()
    }

    fn ts_collect(decls: &mut BTreeMap<String, String>) {
        V::ts_collect(decls);
    }
}

impl<K: TypeScript + Ord, V: TypeScript> TypeScript for BTreeMap<K, V> {
    fn ts_type() -> Cow<'static, str> {
        let v = V::ts_type();
        format!("Record<string, {v}>").into()
    }

    fn ts_collect(decls: &mut BTreeMap<String, String>) {
        V::ts_collect(decls);
    }
}

impl<T: TypeScript, E> TypeScript for Result<T, E> {
    fn ts_type() -> Cow<'static, str> {
        T::ts_type()
    }

    fn ts_collect(decls: &mut BTreeMap<String, String>) {
        T::ts_collect(decls);
    }
}

impl<T: TypeScript> TypeScript for Box<T> {
    fn ts_type() -> Cow<'static, str> {
        T::ts_type()
    }

    fn ts_collect(decls: &mut BTreeMap<String, String>) {
        T::ts_collect(decls);
    }
}

// ——— References ———

impl<T: TypeScript> TypeScript for &T {
    fn ts_type() -> Cow<'static, str> {
        T::ts_type()
    }

    fn ts_collect(decls: &mut BTreeMap<String, String>) {
        T::ts_collect(decls);
    }
}

impl<T: TypeScript> TypeScript for &mut T {
    fn ts_type() -> Cow<'static, str> {
        T::ts_type()
    }

    fn ts_collect(decls: &mut BTreeMap<String, String>) {
        T::ts_collect(decls);
    }
}

// ——— Tuples ———

impl<A: TypeScript, B: TypeScript> TypeScript for (A, B) {
    fn ts_type() -> Cow<'static, str> {
        let a = A::ts_type();
        let b = B::ts_type();
        format!("[{a}, {b}]").into()
    }

    fn ts_collect(decls: &mut BTreeMap<String, String>) {
        A::ts_collect(decls);
        B::ts_collect(decls);
    }
}

impl<A: TypeScript, B: TypeScript, C: TypeScript> TypeScript for (A, B, C) {
    fn ts_type() -> Cow<'static, str> {
        let a = A::ts_type();
        let b = B::ts_type();
        let c = C::ts_type();
        format!("[{a}, {b}, {c}]").into()
    }

    fn ts_collect(decls: &mut BTreeMap<String, String>) {
        A::ts_collect(decls);
        B::ts_collect(decls);
        C::ts_collect(decls);
    }
}

// ——— Opaque boxed (smart-pointer) types ———
//
// Neon boxes smart pointers (`Arc<T>`, `Rc<T>`, `RefCell<T>`, `Ref<T>`,
// `RefMut<T>`) into opaque JavaScript values. The matching TypeScript is a
// *branded interface* so that distinct boxed types are not interchangeable.
// These impls live here (rather than in `neon`) because the orphan rule forbids
// implementing `TypeScript` for these foreign types outside the trait's crate.
// The branded-interface convention is shared with `neon`'s own `Boxed<T>` type
// via the public helpers below. The brand value carries the inner type's
// identity; the interface name is synthesized as `Boxed` + a sanitized identifier
// (so a composed inner type like `string | null` still yields a valid name).

/// Reduce an arbitrary TypeScript type expression to an identifier-safe string
/// (ASCII alphanumerics and `_`), used to synthesize the `Boxed{...}` name.
pub fn sanitize_identifier(s: &str) -> String {
    let mut out: String = s
        .chars()
        .filter(|c| c.is_ascii_alphanumeric() || *c == '_')
        .collect();
    if out.chars().next().is_some_and(|c| c.is_ascii_digit()) {
        out.insert(0, '_');
    }
    out
}

/// The synthesized `Boxed{Inner}` type name for a boxed `T`.
pub fn boxed_ts_type<T: TypeScript>() -> Cow<'static, str> {
    format!("Boxed{}", sanitize_identifier(&T::ts_type())).into()
}

/// The branded-interface declaration for a boxed `T`.
pub fn boxed_ts_decl<T: TypeScript>() -> Option<Cow<'static, str>> {
    let inner = T::ts_type();
    let boxed = format!("Boxed{}", sanitize_identifier(&inner));
    Some(format!("interface {boxed} {{ readonly [__neon_tag]: '{inner}' }}").into())
}

/// Collect the branded-interface declaration for a boxed `T`.
pub fn boxed_ts_collect<T: TypeScript>(decls: &mut BTreeMap<String, String>) {
    if let Some(d) = boxed_ts_decl::<T>() {
        decls
            .entry(boxed_ts_type::<T>().into_owned())
            .or_insert_with(|| d.into_owned());
    }
}

macro_rules! ts_boxed {
    ($($ty:ty),* $(,)?) => {$(
        impl<T: TypeScript> TypeScript for $ty {
            fn ts_type() -> Cow<'static, str> { boxed_ts_type::<T>() }
            fn ts_decl() -> Option<Cow<'static, str>> { boxed_ts_decl::<T>() }
            fn ts_collect(decls: &mut BTreeMap<String, String>) { boxed_ts_collect::<T>(decls) }
        }
    )*};
}

ts_boxed!(Arc<T>, Rc<T>, RefCell<T>);

impl<'a, T: TypeScript> TypeScript for Ref<'a, T> {
    fn ts_type() -> Cow<'static, str> {
        boxed_ts_type::<T>()
    }
    fn ts_decl() -> Option<Cow<'static, str>> {
        boxed_ts_decl::<T>()
    }
    fn ts_collect(decls: &mut BTreeMap<String, String>) {
        boxed_ts_collect::<T>(decls)
    }
}

impl<'a, T: TypeScript> TypeScript for RefMut<'a, T> {
    fn ts_type() -> Cow<'static, str> {
        boxed_ts_type::<T>()
    }
    fn ts_decl() -> Option<Cow<'static, str>> {
        boxed_ts_decl::<T>()
    }
    fn ts_collect(decls: &mut BTreeMap<String, String>) {
        boxed_ts_collect::<T>(decls)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sanitize_identifier_strips_invalid_chars() {
        assert_eq!(sanitize_identifier("Database"), "Database");
        assert_eq!(sanitize_identifier("string | null"), "stringnull");
        assert_eq!(sanitize_identifier("number[]"), "number");
        assert_eq!(sanitize_identifier("3d"), "_3d");
    }

    #[test]
    fn arc_produces_branded_boxed_type() {
        assert_eq!(<Arc<String>>::ts_type(), "Boxedstring");
        // Composed inner types still yield a valid identifier.
        assert_eq!(<Arc<Option<String>>>::ts_type(), "Boxedstringnull");
        let decl = <Arc<Option<String>>>::ts_decl().unwrap();
        assert!(decl.contains("__neon_tag]: 'string | null'"), "{decl}");
    }

    #[test]
    fn primitives() {
        assert_eq!(f64::ts_type(), "number");
        assert_eq!(u8::ts_type(), "number");
        assert_eq!(isize::ts_type(), "number");
        assert_eq!(String::ts_type(), "string");
        assert_eq!(<&str>::ts_type(), "string");
        assert_eq!(bool::ts_type(), "boolean");
        assert_eq!(<()>::ts_type(), "undefined");
    }

    #[test]
    fn option_maps_to_null_union() {
        assert_eq!(<Option<f64>>::ts_type(), "number | null");
    }

    #[test]
    fn vec_and_sets() {
        assert_eq!(<Vec<String>>::ts_type(), "string[]");
        assert_eq!(<HashSet<u32>>::ts_type(), "number[]");
        assert_eq!(<BTreeSet<bool>>::ts_type(), "boolean[]");
        // Union inner types get wrapped in parens.
        assert_eq!(<Vec<Option<f64>>>::ts_type(), "(number | null)[]");
    }

    #[test]
    fn maps_ignore_key_type() {
        assert_eq!(<HashMap<String, f64>>::ts_type(), "Record<string, number>");
        assert_eq!(<HashMap<u32, f64>>::ts_type(), "Record<string, number>");
        assert_eq!(
            <BTreeMap<String, String>>::ts_type(),
            "Record<string, string>"
        );
    }

    #[test]
    fn tuples() {
        assert_eq!(<(f64, String)>::ts_type(), "[number, string]");
        assert_eq!(
            <(f64, String, bool)>::ts_type(),
            "[number, string, boolean]"
        );
    }

    #[test]
    fn result_delegates_to_ok() {
        assert_eq!(<Result<String, ()>>::ts_type(), "string");
    }

    #[test]
    fn box_and_refs_delegate() {
        assert_eq!(<Box<f64>>::ts_type(), "number");
        assert_eq!(<&f64>::ts_type(), "number");
        assert_eq!(<&mut String>::ts_type(), "string");
    }
}
