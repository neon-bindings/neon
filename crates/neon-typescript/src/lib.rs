//! The `TsJson` trait: Neon's minimal, stable contract for describing the
//! TypeScript type of a Rust value **as serialized to JSON** by serde_json.
//! Deliberately tiny and dependency-free.
//!
//! This is the trait third-party generator adapters (e.g. `neon-ts-rs`)
//! implement. It describes only the JSON boundary (`Json<T>` and
//! `#[neon::export(json)]`); the TypeScript types of Neon's own bare
//! conversions (`TryFromJs` / `TryIntoJs`) are described by separate traits in
//! `neon` itself.

use std::borrow::Cow;
use std::cell::RefCell;
use std::collections::{BTreeMap, BTreeSet, HashMap, HashSet};
use std::hash::Hash;
use std::marker::PhantomData;
use std::rc::Rc;
use std::sync::Arc;

/// A Rust type whose serde_json serialization has a known TypeScript type.
pub trait TsJson {
    /// The TypeScript type *expression* for this type's JSON representation
    /// (e.g. `"number"`, `"string[]"`, `"SearchResult"`).
    fn ts_json() -> Cow<'static, str>;

    /// An optional top-level declaration this type needs (an interface or type
    /// alias). `None` for primitives.
    fn ts_json_decl() -> Option<Cow<'static, str>> {
        None
    }

    /// Add this type's own declaration plus its transitive declarations to
    /// `decls`, keyed by name for dedup. Composite types override this to also
    /// collect from their component types.
    fn ts_json_collect(decls: &mut BTreeMap<String, String>) {
        if let Some(d) = Self::ts_json_decl() {
            decls
                .entry(Self::ts_json().into_owned())
                .or_insert_with(|| d.into_owned());
        }
    }
}

/// Wrap a union type in parentheses when it's used as an array element:
/// `(Foo | Bar)[]`, not `Foo | Bar[]`.
pub fn array_of(inner: &str) -> String {
    if inner.contains(" | ") {
        format!("({inner})[]")
    } else {
        format!("{inner}[]")
    }
}

// ——— JSON autoref specialization probe ———
//
// This lets macro-generated metadata gracefully handle JSON payload types that
// don't implement `TsJson`. Method resolution is a three-rung ladder tried
// most-specific first (see `doc/typescript.md`):
//
//   Rung 1: inherent method on `TsJsonProbe<T>` (requires `T: TsJson`).
//   Rung 2: an adapter's trait `impl … for TsJsonProbe<T>` (e.g. `neon-ts-rs`'s
//           `TsJsonExt`), only visible when that trait is `use`d in scope.
//   Rung 3: the `TsJsonFallback` trait impl for `&TsJsonProbe<T>`, reached via
//           autoref for all `T`, yielding `"unknown"`.
//
// The probe lives here (not in `neon`) so an adapter crate can hang rung 2 on it.
//
// Usage in macro-generated code:
//   let __probe = TsJsonProbe::<SomeType>(PhantomData);
//   (&__probe).ts_json_of()

/// Compile-time probe for whether a JSON payload type implements `TsJson`.
pub struct TsJsonProbe<T>(pub PhantomData<T>);

// Higher priority: inherent method, available only when T: TsJson
impl<T: TsJson> TsJsonProbe<T> {
    pub fn ts_json_of(&self) -> Cow<'static, str> {
        T::ts_json()
    }

    pub fn ts_json_collect_of(&self, decls: &mut BTreeMap<String, String>) {
        T::ts_json_collect(decls);
    }
}

/// Lower-priority fallback for JSON payload types with no known TypeScript
/// type. Reached via autoref (`(&probe).method()` resolves to `&TsJsonProbe<T>`
/// first, then falls through to `&&TsJsonProbe<T>`, which matches this impl).
pub trait TsJsonFallback {
    fn ts_json_of(&self) -> Cow<'static, str>;
    fn ts_json_collect_of(&self, decls: &mut BTreeMap<String, String>);
}

impl<T> TsJsonFallback for &TsJsonProbe<T> {
    fn ts_json_of(&self) -> Cow<'static, str> {
        "unknown".into()
    }

    fn ts_json_collect_of(&self, _: &mut BTreeMap<String, String>) {}
}

// ——— Primitive impls ———

macro_rules! ts_primitive {
    ($ty:ty, $ts:literal) => {
        impl TsJson for $ty {
            fn ts_json() -> Cow<'static, str> {
                $ts.into()
            }
        }
    };
}

// serde_json emits every Rust number type (including 64-bit integers) as a
// JSON number.
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
// serde_json serializes `()` as `null`.
ts_primitive!((), "null");

// ——— Wrapper impls ———

// serde_json serializes `None` as a present `null`.
impl<T: TsJson> TsJson for Option<T> {
    fn ts_json() -> Cow<'static, str> {
        format!("{} | null", T::ts_json()).into()
    }

    fn ts_json_collect(decls: &mut BTreeMap<String, String>) {
        T::ts_json_collect(decls);
    }
}

impl<T: TsJson> TsJson for Vec<T> {
    fn ts_json() -> Cow<'static, str> {
        array_of(&T::ts_json()).into()
    }

    fn ts_json_collect(decls: &mut BTreeMap<String, String>) {
        T::ts_json_collect(decls);
    }
}

impl<T: TsJson + Eq + Hash> TsJson for HashSet<T> {
    fn ts_json() -> Cow<'static, str> {
        Vec::<T>::ts_json()
    }

    fn ts_json_collect(decls: &mut BTreeMap<String, String>) {
        T::ts_json_collect(decls);
    }
}

impl<T: TsJson + Ord> TsJson for BTreeSet<T> {
    fn ts_json() -> Cow<'static, str> {
        Vec::<T>::ts_json()
    }

    fn ts_json_collect(decls: &mut BTreeMap<String, String>) {
        T::ts_json_collect(decls);
    }
}

// Maps serialize to JSON objects whose keys are *always* strings, regardless of
// the Rust key type `K` (serde stringifies keys, and JSON has no other kind of
// object key). So the observable TypeScript shape is always `Record<string, V>`.
// We deliberately ignore `K` here: describing a `HashMap<u32, V>` as
// `Record<number, V>` would be misleading (the runtime keys are strings), and
// non-primitive `K` would produce invalid TypeScript (`Record` keys must be
// `string | number | symbol`).
impl<K: TsJson, V: TsJson> TsJson for HashMap<K, V> {
    fn ts_json() -> Cow<'static, str> {
        format!("Record<string, {}>", V::ts_json()).into()
    }

    fn ts_json_collect(decls: &mut BTreeMap<String, String>) {
        V::ts_json_collect(decls);
    }
}

impl<K: TsJson + Ord, V: TsJson> TsJson for BTreeMap<K, V> {
    fn ts_json() -> Cow<'static, str> {
        format!("Record<string, {}>", V::ts_json()).into()
    }

    fn ts_json_collect(decls: &mut BTreeMap<String, String>) {
        V::ts_json_collect(decls);
    }
}

// As *data* inside a JSON payload, serde serializes `Result` as an externally
// tagged enum. (A `Result` *returned* from an export is control flow, not data:
// the macro peels it before consulting this trait, because the `Err` is thrown.)
impl<T: TsJson, E: TsJson> TsJson for Result<T, E> {
    fn ts_json() -> Cow<'static, str> {
        format!("{{ Ok: {} }} | {{ Err: {} }}", T::ts_json(), E::ts_json()).into()
    }

    fn ts_json_collect(decls: &mut BTreeMap<String, String>) {
        T::ts_json_collect(decls);
        E::ts_json_collect(decls);
    }
}

// serde serializes these wrappers transparently, as their inner value. (`Arc`
// and `Rc` require serde's `rc` feature.)
macro_rules! ts_transparent {
    ($($ty:ty),* $(,)?) => {$(
        impl<T: TsJson> TsJson for $ty {
            fn ts_json() -> Cow<'static, str> {
                T::ts_json()
            }

            fn ts_json_collect(decls: &mut BTreeMap<String, String>) {
                T::ts_json_collect(decls);
            }
        }
    )*};
}

ts_transparent!(Box<T>, Arc<T>, Rc<T>, RefCell<T>);

// ——— References ———

impl<T: TsJson + ?Sized> TsJson for &T {
    fn ts_json() -> Cow<'static, str> {
        T::ts_json()
    }

    fn ts_json_collect(decls: &mut BTreeMap<String, String>) {
        T::ts_json_collect(decls);
    }
}

impl<T: TsJson + ?Sized> TsJson for &mut T {
    fn ts_json() -> Cow<'static, str> {
        T::ts_json()
    }

    fn ts_json_collect(decls: &mut BTreeMap<String, String>) {
        T::ts_json_collect(decls);
    }
}

// ——— Tuples ———

impl<A: TsJson, B: TsJson> TsJson for (A, B) {
    fn ts_json() -> Cow<'static, str> {
        format!("[{}, {}]", A::ts_json(), B::ts_json()).into()
    }

    fn ts_json_collect(decls: &mut BTreeMap<String, String>) {
        A::ts_json_collect(decls);
        B::ts_json_collect(decls);
    }
}

impl<A: TsJson, B: TsJson, C: TsJson> TsJson for (A, B, C) {
    fn ts_json() -> Cow<'static, str> {
        format!("[{}, {}, {}]", A::ts_json(), B::ts_json(), C::ts_json()).into()
    }

    fn ts_json_collect(decls: &mut BTreeMap<String, String>) {
        A::ts_json_collect(decls);
        B::ts_json_collect(decls);
        C::ts_json_collect(decls);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn primitives() {
        assert_eq!(f64::ts_json(), "number");
        assert_eq!(u8::ts_json(), "number");
        assert_eq!(i64::ts_json(), "number");
        assert_eq!(String::ts_json(), "string");
        assert_eq!(<&str>::ts_json(), "string");
        assert_eq!(bool::ts_json(), "boolean");
    }

    #[test]
    fn unit_is_null() {
        assert_eq!(<()>::ts_json(), "null");
    }

    #[test]
    fn option_maps_to_null_union() {
        assert_eq!(<Option<f64>>::ts_json(), "number | null");
    }

    #[test]
    fn vec_and_sets() {
        assert_eq!(<Vec<String>>::ts_json(), "string[]");
        assert_eq!(<HashSet<u32>>::ts_json(), "number[]");
        assert_eq!(<BTreeSet<bool>>::ts_json(), "boolean[]");
        // Union inner types get wrapped in parens.
        assert_eq!(<Vec<Option<f64>>>::ts_json(), "(number | null)[]");
    }

    #[test]
    fn maps_ignore_key_type() {
        assert_eq!(<HashMap<String, f64>>::ts_json(), "Record<string, number>");
        assert_eq!(<HashMap<u32, f64>>::ts_json(), "Record<string, number>");
        assert_eq!(
            <BTreeMap<String, String>>::ts_json(),
            "Record<string, string>"
        );
    }

    #[test]
    fn tuples() {
        assert_eq!(<(f64, String)>::ts_json(), "[number, string]");
        assert_eq!(
            <(f64, String, bool)>::ts_json(),
            "[number, string, boolean]"
        );
    }

    #[test]
    fn result_as_data_is_externally_tagged() {
        assert_eq!(
            <Result<String, u32>>::ts_json(),
            "{ Ok: string } | { Err: number }"
        );
    }

    #[test]
    fn transparent_wrappers_delegate() {
        assert_eq!(<Box<f64>>::ts_json(), "number");
        assert_eq!(<Arc<String>>::ts_json(), "string");
        assert_eq!(<Rc<Vec<u32>>>::ts_json(), "number[]");
        assert_eq!(<RefCell<bool>>::ts_json(), "boolean");
        assert_eq!(<&f64>::ts_json(), "number");
        assert_eq!(<&mut String>::ts_json(), "string");
    }

    // The `(&probe)` borrow mirrors the macro-emitted call shape.
    #[allow(clippy::needless_borrow)]
    #[test]
    fn probe_falls_back_to_unknown() {
        struct NoImpl;
        let probe = TsJsonProbe::<NoImpl>(PhantomData);
        assert_eq!((&probe).ts_json_of(), "unknown");
        let probe = TsJsonProbe::<f64>(PhantomData);
        assert_eq!((&probe).ts_json_of(), "number");
    }
}
