// Adapted from https://github.com/serde-rs/json/blob/master/tests/test.rs

use std::{any, collections::BTreeMap, fmt, marker::PhantomData};

use neon::{prelude::*, types::buffer::TypedArray};

use serde::{
    de::{self, DeserializeOwned},
    ser, Deserialize, Serialize,
};

use serde_bytes::{ByteBuf, Bytes};

use serde_json::json;

const MAX_SAFE_INTEGER: u64 = 9_007_199_254_740_991;
const MIN_SAFE_INTEGER: i64 = -9_007_199_254_740_991;

macro_rules! json_str {
    ([]) => {
        "[]"
    };
    ([ $e0:tt $(, $e:tt)* $(,)? ]) => {
        concat!("[",
            json_str!($e0),
            $(",", json_str!($e),)*
        "]")
    };
    ({}) => {
        "{}"
    };
    ({ $k0:tt : $v0:tt $(, $k:tt : $v:tt)* $(,)? }) => {
        concat!("{",
            stringify!($k0), ":", json_str!($v0),
            $(",", stringify!($k), ":", json_str!($v),)*
        "}")
    };
    (($other:tt)) => {
        $other
    };
    ($other:tt) => {
        stringify!($other)
    };
}

macro_rules! treemap {
    () => {
        BTreeMap::new()
    };
    ($($k:expr => $v:expr),+) => {
        {
            let mut m = BTreeMap::new();
            $(
                m.insert($k, $v);
            )+
            m
        }
    };
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
enum Animal {
    Dog,
    Frog(String, Vec<isize>),
    Cat { age: usize, name: String },
    AntHive(Vec<String>),
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
struct Inner {
    a: (),
    b: usize,
    c: Vec<String>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
struct Outer {
    inner: Vec<Inner>,
}

fn export<F>(cx: &mut FunctionContext, o: &JsObject, f: F) -> NeonResult<()>
where
    F: Fn(&mut FunctionContext) + 'static,
{
    let f = JsFunction::new(cx, move |mut cx| {
        f(&mut cx);
        Ok(cx.undefined())
    })?;

    o.set(cx, any::type_name::<F>(), f)?;

    Ok(())
}

fn test_encode_ok<T>(cx: &mut FunctionContext, tests: &[(T, &str)])
where
    T: PartialEq + fmt::Debug + ser::Serialize + de::DeserializeOwned,
{
    for &(ref value, out) in tests {
        let out = out.to_string();
        let s = to_string(cx, value).unwrap();
        assert_eq!(s, out);

        // Make sure we can round trip
        let v = neon::serialize(cx, value).unwrap();
        let d = neon::deserialize::<T, JsValue, _>(cx, v).unwrap();
        assert_eq!(value, &d);
    }
}

fn test_parse_ok<T>(cx: &mut FunctionContext, tests: Vec<(&str, T)>)
where
    T: Clone + fmt::Debug + PartialEq + ser::Serialize + de::DeserializeOwned,
{
    for (s, value) in tests {
        let v: T = from_str(cx, s).unwrap();
        assert_eq!(v, value.clone());

        // Make sure we can round trip
        let s2 = to_string(cx, &v).unwrap();
        let v2 = from_str(cx, &s2).unwrap();
        assert_eq!(v, v2);
    }
}

// For testing representations that the deserializer accepts but the serializer
// never generates. These do not survive a round-trip.
fn test_parse_unusual_ok<T>(cx: &mut FunctionContext, tests: Vec<(&str, T)>)
where
    T: Clone + fmt::Debug + PartialEq + ser::Serialize + de::DeserializeOwned,
{
    for (s, value) in tests {
        let v: T = from_str(cx, s).unwrap();
        assert_eq!(v, value);
    }
}

fn to_string<T>(cx: &mut FunctionContext, v: &T) -> NeonResult<String>
where
    T: ?Sized + serde::Serialize,
{
    let v = neon::serialize(cx, v)?;
    let s = cx
        .global()
        .get::<JsObject, _, _>(cx, "JSON")?
        .get::<JsFunction, _, _>(cx, "stringify")?
        .call_with(cx)
        .arg::<JsValue>(v)
        .apply::<JsString, _>(cx)?;

    Ok(s.value(cx))
}

fn from_str<T>(cx: &mut FunctionContext, s: &str) -> NeonResult<T>
where
    T: de::DeserializeOwned,
{
    let v = cx
        .global()
        .get::<JsObject, _, _>(cx, "JSON")?
        .get::<JsFunction, _, _>(cx, "parse")?
        .call_with(cx)
        .arg(cx.string(s))
        .apply::<JsValue, _>(cx)?;

    neon::deserialize(cx, v)
}

fn test_write_null(cx: &mut FunctionContext) {
    let tests = &[((), "null")];
    test_encode_ok(cx, tests);
}

fn test_write_u64(cx: &mut FunctionContext) {
    let tests = &[
        (3u64, "3"),
        (MAX_SAFE_INTEGER, &MAX_SAFE_INTEGER.to_string()),
    ];
    test_encode_ok(cx, tests);
}

fn test_write_i64(cx: &mut FunctionContext) {
    let tests = &[
        (3i64, "3"),
        (-2i64, "-2"),
        (-1234i64, "-1234"),
        (MIN_SAFE_INTEGER, &MIN_SAFE_INTEGER.to_string()),
    ];
    test_encode_ok(cx, tests);
}

fn test_write_f64(cx: &mut FunctionContext) {
    let tests = &[
        (3.0, "3"),
        (3.1, "3.1"),
        (-1.5, "-1.5"),
        (0.5, "0.5"),
        (f64::MIN, "-1.7976931348623157e+308"),
        (f64::MAX, "1.7976931348623157e+308"),
        (f64::EPSILON, "2.220446049250313e-16"),
    ];
    test_encode_ok(cx, tests);
}

fn test_encode_nonfinite_float_yields_null(cx: &mut FunctionContext) {
    let v = to_string(cx, &f64::NAN).unwrap();
    assert_eq!(v, "null");

    let v = to_string(cx, &f64::INFINITY).unwrap();
    assert_eq!(v, "null");

    let v = to_string(cx, &f32::NAN).unwrap();
    assert_eq!(v, "null");

    let v = to_string(cx, &f32::INFINITY).unwrap();
    assert_eq!(v, "null");
}

fn test_write_str(cx: &mut FunctionContext) {
    let tests = &[("".to_owned(), "\"\""), ("foo".to_owned(), "\"foo\"")];
    test_encode_ok(cx, tests);
}

fn test_write_bool(cx: &mut FunctionContext) {
    let tests = &[(true, "true"), (false, "false")];
    test_encode_ok(cx, tests);
}

fn test_write_char(cx: &mut FunctionContext) {
    let tests = &[
        ('n', "\"n\""),
        ('"', "\"\\\"\""),
        ('\\', "\"\\\\\""),
        ('/', "\"/\""),
        ('\x08', "\"\\b\""),
        ('\x0C', "\"\\f\""),
        ('\n', "\"\\n\""),
        ('\r', "\"\\r\""),
        ('\t', "\"\\t\""),
        ('\x0B', "\"\\u000b\""),
        ('\u{3A3}', "\"\u{3A3}\""),
    ];
    test_encode_ok(cx, tests);
}

fn test_write_list(cx: &mut FunctionContext) {
    test_encode_ok(
        cx,
        &[
            (vec![], "[]"),
            (vec![true], "[true]"),
            (vec![true, false], "[true,false]"),
        ],
    );

    test_encode_ok(
        cx,
        &[
            (vec![vec![], vec![], vec![]], "[[],[],[]]"),
            (vec![vec![1, 2, 3], vec![], vec![]], "[[1,2,3],[],[]]"),
            (vec![vec![], vec![1, 2, 3], vec![]], "[[],[1,2,3],[]]"),
            (vec![vec![], vec![], vec![1, 2, 3]], "[[],[],[1,2,3]]"),
        ],
    );

    let long_test_list = json!([false, null, ["foo\nbar", 3.5]]);

    test_encode_ok(
        cx,
        &[(long_test_list, json_str!([false, null, ["foo\nbar", 3.5]]))],
    );
}

fn test_write_object(cx: &mut FunctionContext) {
    test_encode_ok(
        cx,
        &[
            (treemap!(), "{}"),
            (treemap!("a".to_string() => true), "{\"a\":true}"),
            (
                treemap!(
                    "a".to_string() => true,
                    "b".to_string() => false
                ),
                "{\"a\":true,\"b\":false}",
            ),
        ],
    );

    test_encode_ok(
        cx,
        &[
            (
                treemap![
                    "a".to_string() => treemap![],
                    "b".to_string() => treemap![],
                    "c".to_string() => treemap![]
                ],
                "{\"a\":{},\"b\":{},\"c\":{}}",
            ),
            (
                treemap![
                    "a".to_string() => treemap![
                        "a".to_string() => treemap!["a".to_string() => vec![1,2,3]],
                        "b".to_string() => treemap![],
                        "c".to_string() => treemap![]
                    ],
                    "b".to_string() => treemap![],
                    "c".to_string() => treemap![]
                ],
                "{\"a\":{\"a\":{\"a\":[1,2,3]},\"b\":{},\"c\":{}},\"b\":{},\"c\":{}}",
            ),
            (
                treemap![
                    "a".to_string() => treemap![],
                    "b".to_string() => treemap![
                        "a".to_string() => treemap!["a".to_string() => vec![1,2,3]],
                        "b".to_string() => treemap![],
                        "c".to_string() => treemap![]
                    ],
                    "c".to_string() => treemap![]
                ],
                "{\"a\":{},\"b\":{\"a\":{\"a\":[1,2,3]},\"b\":{},\"c\":{}},\"c\":{}}",
            ),
            (
                treemap![
                    "a".to_string() => treemap![],
                    "b".to_string() => treemap![],
                    "c".to_string() => treemap![
                        "a".to_string() => treemap!["a".to_string() => vec![1,2,3]],
                        "b".to_string() => treemap![],
                        "c".to_string() => treemap![]
                    ]
                ],
                "{\"a\":{},\"b\":{},\"c\":{\"a\":{\"a\":[1,2,3]},\"b\":{},\"c\":{}}}",
            ),
        ],
    );

    test_encode_ok(cx, &[(treemap!['c' => ()], "{\"c\":null}")]);

    let complex_obj = json!({
        "b": [
            {"c": "\x0c\x1f\r"},
            {"d": ""}
        ]
    });

    test_encode_ok(
        cx,
        &[(
            complex_obj,
            json_str!({
                "b": [
                    {
                        "c": (r#""\f\u001f\r""#)
                    },
                    {
                        "d": ""
                    }
                ]
            }),
        )],
    );
}

fn test_write_tuple(cx: &mut FunctionContext) {
    test_encode_ok(cx, &[((5,), "[5]")]);

    test_encode_ok(cx, &[((5, (6, "abc".to_owned())), "[5,[6,\"abc\"]]")]);
}

fn test_write_enum(cx: &mut FunctionContext) {
    test_encode_ok(
        cx,
        &[
            (Animal::Dog, "\"Dog\""),
            (
                Animal::Frog("Henry".to_string(), vec![]),
                "{\"Frog\":[\"Henry\",[]]}",
            ),
            (
                Animal::Frog("Henry".to_string(), vec![349]),
                "{\"Frog\":[\"Henry\",[349]]}",
            ),
            (
                Animal::Frog("Henry".to_string(), vec![349, 102]),
                "{\"Frog\":[\"Henry\",[349,102]]}",
            ),
            (
                Animal::Cat {
                    age: 5,
                    name: "Kate".to_string(),
                },
                "{\"Cat\":{\"age\":5,\"name\":\"Kate\"}}",
            ),
            (
                Animal::AntHive(vec!["Bob".to_string(), "Stuart".to_string()]),
                "{\"AntHive\":[\"Bob\",\"Stuart\"]}",
            ),
        ],
    );
}

fn test_write_option(cx: &mut FunctionContext) {
    test_encode_ok(
        cx,
        &[
            (None, "null"),
            (Some("jodhpurs".to_owned()), "\"jodhpurs\""),
        ],
    );

    test_encode_ok(
        cx,
        &[
            (None, "null"),
            (
                Some(vec!["foo".to_owned(), "bar".to_owned()]),
                "[\"foo\",\"bar\"]",
            ),
        ],
    );
}

fn test_write_newtype_struct(cx: &mut FunctionContext) {
    #[derive(Clone, Deserialize, Serialize, PartialEq, Debug)]
    struct Newtype(BTreeMap<String, i32>);

    let inner = Newtype(treemap!(String::from("inner") => 123));

    test_encode_ok(cx, &[(inner.clone(), r#"{"inner":123}"#)]);

    let outer = treemap!(String::from("outer") => inner);

    test_encode_ok(cx, &[(outer, r#"{"outer":{"inner":123}}"#)]);
}

fn test_deserialize_number_to_untagged_enum(cx: &mut FunctionContext) {
    #[derive(PartialEq, Deserialize, Debug)]
    #[serde(untagged)]
    enum E<T> {
        N(T),
    }

    fn test<T>(h: Handle<JsNumber>, v: T, cx: &mut FunctionContext)
    where
        T: PartialEq + DeserializeOwned + fmt::Debug,
    {
        assert_eq!(neon::deserialize::<E<T>, _, _>(cx, h).unwrap(), E::N(v));
    }

    test(cx.number(5), 5i64, cx);
    test(cx.number(0), 0i64, cx);
    test(cx.number(-0), 0i64, cx);
    test(cx.number(-5), -5i64, cx);
    test(cx.number(0), 0u64, cx);
    test(cx.number(5), 5u64, cx);
    test(cx.number(-5), -5f64, cx);
    test(cx.number(-5.5), -5.5f64, cx);
    test(cx.number(0), 0f64, cx);
    test(cx.number(5), 5f64, cx);
    test(cx.number(5.5), 5.5f64, cx);
}

fn test_parse_null(cx: &mut FunctionContext) {
    test_parse_ok(cx, vec![("null", ())]);
}

fn test_parse_bool(cx: &mut FunctionContext) {
    test_parse_ok(
        cx,
        vec![
            ("true", true),
            (" true ", true),
            ("false", false),
            (" false ", false),
        ],
    );
}

fn test_parse_char(cx: &mut FunctionContext) {
    test_parse_ok(
        cx,
        vec![
            ("\"n\"", 'n'),
            ("\"\\\"\"", '"'),
            ("\"\\\\\"", '\\'),
            ("\"/\"", '/'),
            ("\"\\b\"", '\x08'),
            ("\"\\f\"", '\x0C'),
            ("\"\\n\"", '\n'),
            ("\"\\r\"", '\r'),
            ("\"\\t\"", '\t'),
            ("\"\\u000b\"", '\x0B'),
            ("\"\\u000B\"", '\x0B'),
            ("\"\u{3A3}\"", '\u{3A3}'),
        ],
    );
}

fn test_parse_i64(cx: &mut FunctionContext) {
    test_parse_ok(
        cx,
        vec![
            ("-2", -2),
            ("-1234", -1234),
            (" -1234 ", -1234),
            (&i64::MIN.to_string(), i64::MIN),
            (&i64::MAX.to_string(), i64::MAX),
        ],
    );
}

fn test_parse_u64(cx: &mut FunctionContext) {
    test_parse_ok(
        cx,
        vec![
            ("0", 0u64),
            ("3", 3u64),
            ("1234", 1234),
            (&u64::MAX.to_string(), u64::MAX),
        ],
    );
}

fn test_parse_f64(cx: &mut FunctionContext) {
    test_parse_ok(
        cx,
        vec![
            ("0.0", 0.0f64),
            ("3.0", 3.0f64),
            ("3.1", 3.1),
            ("-1.2", -1.2),
            ("0.4", 0.4),
            // Edge case from:
            // https://github.com/serde-rs/json/issues/536#issuecomment-583714900
            ("2.638344616030823e-256", 2.638344616030823e-256),
        ],
    );

    test_parse_ok(
        cx,
        vec![
            // With arbitrary-precision enabled, this parses as Number{"3.00"}
            // but the float is Number{"3.0"}
            ("3.00", 3.0f64),
            ("0.4e5", 0.4e5),
            ("0.4e+5", 0.4e5),
            ("0.4e15", 0.4e15),
            ("0.4e+15", 0.4e15),
            ("0.4e-01", 0.4e-1),
            (" 0.4e-01 ", 0.4e-1),
            ("0.4e-001", 0.4e-1),
            ("0.4e-0", 0.4e0),
            ("0.00e00", 0.0),
            ("0.00e+00", 0.0),
            ("0.00e-00", 0.0),
            ("3.5E-2147483647", 0.0),
            ("0.0100000000000000000001", 0.01),
            (
                &format!("{}", (i64::MIN as f64) - 1.0),
                (i64::MIN as f64) - 1.0,
            ),
            (
                &format!("{}", (u64::MAX as f64) + 1.0),
                (u64::MAX as f64) + 1.0,
            ),
            (&format!("{}", f64::EPSILON), f64::EPSILON),
            (
                "0.0000000000000000000000000000000000000000000000000123e50",
                1.23,
            ),
            ("100e-777777777777777777777777777", 0.0),
            (
                "1010101010101010101010101010101010101010",
                1.010_101_010_101_01e39,
            ),
            (
                "0.1010101010101010101010101010101010101010",
                0.101_010_101_010_101_01,
            ),
            ("0e1000000000000000000000000000000000000000000000", 0.0),
            (
                "1000000000000000000000000000000000000000000000000000000000000\
             000000000000000000000000000000000000000000000000000000000000\
             000000000000000000000000000000000000000000000000000000000000\
             000000000000000000000000000000000000000000000000000000000000\
             000000000000000000000000000000000000000000000000000000000000\
             00000000",
                1e308,
            ),
            (
                "1000000000000000000000000000000000000000000000000000000000000\
             000000000000000000000000000000000000000000000000000000000000\
             000000000000000000000000000000000000000000000000000000000000\
             000000000000000000000000000000000000000000000000000000000000\
             000000000000000000000000000000000000000000000000000000000000\
             .0e8",
                1e308,
            ),
            (
                "1000000000000000000000000000000000000000000000000000000000000\
             000000000000000000000000000000000000000000000000000000000000\
             000000000000000000000000000000000000000000000000000000000000\
             000000000000000000000000000000000000000000000000000000000000\
             000000000000000000000000000000000000000000000000000000000000\
             e8",
                1e308,
            ),
            (
                "1000000000000000000000000000000000000000000000000000000000000\
             000000000000000000000000000000000000000000000000000000000000\
             000000000000000000000000000000000000000000000000000000000000\
             000000000000000000000000000000000000000000000000000000000000\
             000000000000000000000000000000000000000000000000000000000000\
             000000000000000000e-10",
                1e308,
            ),
        ],
    );
}

fn test_value_as_f64(cx: &mut FunctionContext) {
    test_parse_unusual_ok(
        cx,
        vec![
            ("1e1000", f64::INFINITY), // Serializes as `null`
        ],
    );
}

fn test_roundtrip_f64(cx: &mut FunctionContext) {
    for &float in &[
        // Samples from quickcheck-ing roundtrip with `input: f64`. Comments
        // indicate the value returned by the old deserializer.
        51.24817837550540_4,  // 51.2481783755054_1
        -93.3113703768803_3,  // -93.3113703768803_2
        -36.5739948427534_36, // -36.5739948427534_4
        52.31400820410624_4,  // 52.31400820410624_
        97.4536532003468_5,   // 97.4536532003468_4
        // Samples from `rng.next_u64` + `f64::from_bits` + `is_finite` filter.
        2.0030397744267762e-253,
        7.101215824554616e260,
        1.769268377902049e74,
        -1.6727517818542075e58,
        3.9287532173373315e299,
    ] {
        let json = to_string(cx, &float).unwrap();
        let output: f64 = from_str(cx, &json).unwrap();
        assert_eq!(float, output);
    }
}

fn test_roundtrip_f32(cx: &mut FunctionContext) {
    let float = 7.038531e-26;
    let json = to_string(cx, &float).unwrap();
    let output: f64 = from_str(cx, &json).unwrap();
    assert_eq!(float, output);
}

fn test_serialize_char(cx: &mut FunctionContext) {
    let value = json!(
        ({
            let mut map = BTreeMap::new();
            map.insert('c', ());
            map
        })
    );

    neon::serialize::<JsObject, _, _>(cx, &value)
        .unwrap()
        .get::<JsNull, _, _>(cx, "c")
        .unwrap();
}

fn test_parse_number(cx: &mut FunctionContext) {
    test_parse_ok(
        cx,
        vec![
            ("0.0", 0.0f64),
            ("3.0", 3.0f64),
            ("3.1", 3.1),
            ("-1.2", -1.2),
            ("0.4", 0.4),
        ],
    );
}

fn test_parse_string(cx: &mut FunctionContext) {
    test_parse_ok(
        cx,
        vec![
            ("\"\"", String::new()),
            ("\"foo\"", "foo".to_string()),
            (" \"foo\" ", "foo".to_string()),
            ("\"\\\"\"", "\"".to_string()),
            ("\"\\b\"", "\x08".to_string()),
            ("\"\\n\"", "\n".to_string()),
            ("\"\\r\"", "\r".to_string()),
            ("\"\\t\"", "\t".to_string()),
            ("\"\\u12ab\"", "\u{12ab}".to_string()),
            ("\"\\uAB12\"", "\u{AB12}".to_string()),
            ("\"\\uD83C\\uDF95\"", "\u{1F395}".to_string()),
        ],
    );
}

fn test_parse_list(cx: &mut FunctionContext) {
    test_parse_ok(
        cx,
        vec![
            ("[]", vec![]),
            ("[ ]", vec![]),
            ("[null]", vec![()]),
            (" [ null ] ", vec![()]),
        ],
    );

    test_parse_ok(cx, vec![("[true]", vec![true])]);

    test_parse_ok(
        cx,
        vec![("[3,1]", vec![3u64, 1]), (" [ 3 , 1 ] ", vec![3, 1])],
    );

    test_parse_ok(cx, vec![("[[3], [1, 2]]", vec![vec![3u64], vec![1, 2]])]);

    test_parse_ok(cx, vec![("[1]", (1u64,))]);

    test_parse_ok(cx, vec![("[1, 2]", (1u64, 2u64))]);

    test_parse_ok(cx, vec![("[1, 2, 3]", (1u64, 2u64, 3u64))]);

    test_parse_ok(cx, vec![("[1, [2, 3]]", (1u64, (2u64, 3u64)))]);
}

fn test_parse_object(cx: &mut FunctionContext) {
    test_parse_ok(
        cx,
        vec![
            ("{}", treemap!()),
            ("{ }", treemap!()),
            ("{\"a\":3}", treemap!("a".to_string() => 3u64)),
            ("{ \"a\" : 3 }", treemap!("a".to_string() => 3)),
            (
                "{\"a\":3,\"b\":4}",
                treemap!("a".to_string() => 3, "b".to_string() => 4),
            ),
            (
                " { \"a\" : 3 , \"b\" : 4 } ",
                treemap!("a".to_string() => 3, "b".to_string() => 4),
            ),
        ],
    );

    test_parse_ok(
        cx,
        vec![(
            "{\"a\": {\"b\": 3, \"c\": 4}}",
            treemap!(
                "a".to_string() => treemap!(
                    "b".to_string() => 3u64,
                    "c".to_string() => 4
                )
            ),
        )],
    );

    test_parse_ok(cx, vec![("{\"c\":null}", treemap!('c' => ()))]);
}

fn test_parse_struct(cx: &mut FunctionContext) {
    test_parse_ok(
        cx,
        vec![
            (
                "{
                \"inner\": []
            }",
                Outer { inner: vec![] },
            ),
            (
                "{
                \"inner\": [
                    { \"a\": null, \"b\": 2, \"c\": [\"abc\", \"xyz\"] }
                ]
            }",
                Outer {
                    inner: vec![Inner {
                        a: (),
                        b: 2,
                        c: vec!["abc".to_string(), "xyz".to_string()],
                    }],
                },
            ),
        ],
    );

    let v: Outer = from_str(
        cx,
        "[
            [
                [ null, 2, [\"abc\", \"xyz\"] ]
            ]
        ]",
    )
    .unwrap();

    assert_eq!(
        v,
        Outer {
            inner: vec![Inner {
                a: (),
                b: 2,
                c: vec!["abc".to_string(), "xyz".to_string()],
            }],
        }
    );
}

fn test_parse_option(cx: &mut FunctionContext) {
    test_parse_ok(
        cx,
        vec![
            ("null", None::<String>),
            ("\"jodhpurs\"", Some("jodhpurs".to_string())),
        ],
    );

    #[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
    struct Foo {
        x: Option<isize>,
    }

    let value: Foo = from_str(cx, "{}").unwrap();
    assert_eq!(value, Foo { x: None });

    test_parse_ok(
        cx,
        vec![
            ("{\"x\": null}", Foo { x: None }),
            ("{\"x\": 5}", Foo { x: Some(5) }),
        ],
    );
}

fn test_parse_enum(cx: &mut FunctionContext) {
    test_parse_ok(
        cx,
        vec![
            ("\"Dog\"", Animal::Dog),
            (" \"Dog\" ", Animal::Dog),
            (
                "{\"Frog\":[\"Henry\",[]]}",
                Animal::Frog("Henry".to_string(), vec![]),
            ),
            (
                " { \"Frog\": [ \"Henry\" , [ 349, 102 ] ] } ",
                Animal::Frog("Henry".to_string(), vec![349, 102]),
            ),
            (
                "{\"Cat\": {\"age\": 5, \"name\": \"Kate\"}}",
                Animal::Cat {
                    age: 5,
                    name: "Kate".to_string(),
                },
            ),
            (
                " { \"Cat\" : { \"age\" : 5 , \"name\" : \"Kate\" } } ",
                Animal::Cat {
                    age: 5,
                    name: "Kate".to_string(),
                },
            ),
            (
                " { \"AntHive\" : [\"Bob\", \"Stuart\"] } ",
                Animal::AntHive(vec!["Bob".to_string(), "Stuart".to_string()]),
            ),
        ],
    );

    test_parse_unusual_ok(
        cx,
        vec![
            ("{\"Dog\":null}", Animal::Dog),
            (" { \"Dog\" : null } ", Animal::Dog),
        ],
    );

    test_parse_ok(
        cx,
        vec![(
            concat!(
                "{",
                "  \"a\": \"Dog\",",
                "  \"b\": {\"Frog\":[\"Henry\", []]}",
                "}"
            ),
            treemap!(
                "a".to_string() => Animal::Dog,
                "b".to_string() => Animal::Frog("Henry".to_string(), vec![])
            ),
        )],
    );
}

fn test_missing_option_field(cx: &mut FunctionContext) {
    #[derive(Debug, PartialEq, Deserialize)]
    struct Foo {
        x: Option<u32>,
    }

    let value: Foo = from_str(cx, "{}").unwrap();
    assert_eq!(value, Foo { x: None });

    let value: Foo = from_str(cx, "{\"x\": 5}").unwrap();
    assert_eq!(value, Foo { x: Some(5) });
}

fn test_missing_renamed_field(cx: &mut FunctionContext) {
    #[derive(Debug, PartialEq, Deserialize)]
    struct Foo {
        #[serde(rename = "y")]
        x: Option<u32>,
    }

    let value: Foo = from_str(cx, "{}").unwrap();
    assert_eq!(value, Foo { x: None });

    let value: Foo = from_str(cx, "{\"y\": 5}").unwrap();
    assert_eq!(value, Foo { x: Some(5) });
}

fn test_serialize_map_with_no_len(cx: &mut FunctionContext) {
    #[derive(Clone, Debug, PartialEq)]
    struct MyMap<K, V>(BTreeMap<K, V>);

    impl<K, V> ser::Serialize for MyMap<K, V>
    where
        K: ser::Serialize + Ord,
        V: ser::Serialize,
    {
        #[inline]
        fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
        where
            S: ser::Serializer,
        {
            let mut map = serializer.serialize_map(None)?;
            for (k, v) in &self.0 {
                ser::SerializeMap::serialize_entry(&mut map, k, v)?;
            }
            ser::SerializeMap::end(map)
        }
    }

    struct Visitor<K, V> {
        marker: PhantomData<MyMap<K, V>>,
    }

    impl<'de, K, V> de::Visitor<'de> for Visitor<K, V>
    where
        K: de::Deserialize<'de> + Eq + Ord,
        V: de::Deserialize<'de>,
    {
        type Value = MyMap<K, V>;

        fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
            formatter.write_str("map")
        }

        #[inline]
        fn visit_unit<E>(self) -> Result<MyMap<K, V>, E>
        where
            E: de::Error,
        {
            Ok(MyMap(BTreeMap::new()))
        }

        #[inline]
        fn visit_map<Visitor>(self, mut visitor: Visitor) -> Result<MyMap<K, V>, Visitor::Error>
        where
            Visitor: de::MapAccess<'de>,
        {
            let mut values = BTreeMap::new();

            while let Some((key, value)) = visitor.next_entry()? {
                values.insert(key, value);
            }

            Ok(MyMap(values))
        }
    }

    impl<'de, K, V> de::Deserialize<'de> for MyMap<K, V>
    where
        K: de::Deserialize<'de> + Eq + Ord,
        V: de::Deserialize<'de>,
    {
        fn deserialize<D>(deserializer: D) -> Result<MyMap<K, V>, D::Error>
        where
            D: de::Deserializer<'de>,
        {
            deserializer.deserialize_map(Visitor {
                marker: PhantomData,
            })
        }
    }

    let mut map = BTreeMap::new();
    map.insert("a".to_owned(), MyMap(BTreeMap::new()));
    map.insert("b".to_owned(), MyMap(BTreeMap::new()));
    let map: MyMap<_, MyMap<u32, u32>> = MyMap(map);

    test_encode_ok(cx, &[(map, "{\"a\":{},\"b\":{}}")]);
}

fn test_serialize_rejects_bool_keys(cx: &mut FunctionContext) {
    let map = treemap!(
        true => 2,
        false => 4
    );

    let r = cx.try_catch(|cx| neon::serialize::<JsValue, _, _>(cx, &map));
    assert!(r.is_err());
}

fn test_serialize_rejects_adt_keys(cx: &mut FunctionContext) {
    let map = treemap!(
        Some("a") => 2,
        Some("b") => 4,
        None => 6
    );

    let r = cx.try_catch(|cx| neon::serialize::<JsValue, _, _>(cx, &map));
    assert!(r.is_err());
}

fn test_bytes_ser(cx: &mut FunctionContext) {
    let buf = vec![];
    let bytes = Bytes::new(&buf);
    let v = neon::serialize::<JsArrayBuffer, _, _>(cx, &bytes).unwrap();

    assert_eq!(&buf, v.as_slice(cx));

    let buf = vec![1, 2, 3];
    let bytes = Bytes::new(&buf);
    let v = neon::serialize::<JsArrayBuffer, _, _>(cx, &bytes).unwrap();

    assert_eq!(&buf, v.as_slice(cx));
}

fn test_byte_buf_ser(cx: &mut FunctionContext) {
    let bytes = ByteBuf::new();
    let v = neon::serialize::<JsArrayBuffer, _, _>(cx, &bytes).unwrap();

    assert_eq!(&bytes, v.as_slice(cx));

    let bytes = ByteBuf::from(vec![1, 2, 3]);
    let v = neon::serialize::<JsArrayBuffer, _, _>(cx, &bytes).unwrap();

    assert_eq!(&bytes, v.as_slice(cx));
}

fn test_byte_buf_de(cx: &mut FunctionContext) {
    let bytes = ByteBuf::new();
    let buf = cx.array_buffer(0).unwrap();
    let v = neon::deserialize::<ByteBuf, _, _>(cx, buf).unwrap();
    assert_eq!(v, bytes);

    let bytes = ByteBuf::from(vec![1, 2, 3]);
    let buf = JsArrayBuffer::from_slice(cx, &bytes).unwrap();

    let v = neon::deserialize::<ByteBuf, _, _>(cx, buf).unwrap();
    assert_eq!(v, bytes);
}

fn test_array_view_de(cx: &mut FunctionContext) {
    let bytes = ByteBuf::new();
    let buf = JsUint8Array::from_slice(cx, &bytes).unwrap();
    let v = neon::deserialize::<ByteBuf, _, _>(cx, buf).unwrap();
    assert_eq!(v, bytes);

    let bytes = ByteBuf::from(vec![1, 2, 3]);
    let buf = JsUint8Array::from_slice(cx, &bytes).unwrap();

    let v = neon::deserialize::<ByteBuf, _, _>(cx, buf).unwrap();
    assert_eq!(v, bytes);
}

fn test_byte_buf_de_multiple(cx: &mut FunctionContext) {
    let a = ByteBuf::from(b"ab\nc".to_vec());
    let b = ByteBuf::from(b"cd\ne".to_vec());
    let left = vec![a, b];
    let v = neon::serialize(cx, &left).unwrap();
    let right = neon::deserialize::<Vec<ByteBuf>, JsValue, _>(cx, v).unwrap();

    assert_eq!(left, right);
}

fn test_deny_float_key(cx: &mut FunctionContext) {
    #[derive(Eq, PartialEq, Ord, PartialOrd)]
    struct Float;
    impl Serialize for Float {
        fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
        where
            S: ser::Serializer,
        {
            serializer.serialize_f32(1.0)
        }
    }

    // map with float key
    let map = treemap!(Float => "x");
    let r = cx.try_catch(|cx| neon::serialize::<JsValue, _, _>(cx, &map));
    assert!(r.is_err());
}

fn test_effectively_string_keys(cx: &mut FunctionContext) {
    #[derive(Eq, PartialEq, Ord, PartialOrd, Debug, Clone, Serialize, Deserialize)]
    enum Enum {
        One,
        Two,
    }
    let map = treemap! {
        Enum::One => 1,
        Enum::Two => 2
    };
    let expected = r#"{"One":1,"Two":2}"#;
    test_encode_ok(cx, &[(map.clone(), expected)]);
    test_parse_ok(cx, vec![(expected, map)]);

    #[derive(Eq, PartialEq, Ord, PartialOrd, Debug, Clone, Serialize, Deserialize)]
    struct Wrapper(String);
    let map = treemap! {
        Wrapper("zero".to_owned()) => 0,
        Wrapper("one".to_owned()) => 1
    };
    let expected = r#"{"one":1,"zero":0}"#;
    test_encode_ok(cx, &[(map.clone(), expected)]);
    test_parse_ok(cx, vec![(expected, map)]);
}

// Note: This is `Issue #220` in the `serde/serde-json` repository
fn issue_220(cx: &mut FunctionContext) {
    #[derive(Debug, PartialEq, Eq, Deserialize, Serialize)]
    enum E {
        V(u8),
    }

    assert_eq!(from_str::<E>(cx, r#"{"V": 0}"#).unwrap(), E::V(0));
}

pub fn build_suite(mut cx: FunctionContext) -> JsResult<JsObject> {
    let o = cx.empty_object();

    export(&mut cx, &o, test_write_null)?;
    export(&mut cx, &o, test_write_u64)?;
    export(&mut cx, &o, test_write_i64)?;
    export(&mut cx, &o, test_write_f64)?;
    export(&mut cx, &o, test_encode_nonfinite_float_yields_null)?;
    export(&mut cx, &o, test_write_str)?;
    export(&mut cx, &o, test_write_bool)?;
    export(&mut cx, &o, test_write_char)?;
    export(&mut cx, &o, test_write_list)?;
    export(&mut cx, &o, test_write_object)?;
    export(&mut cx, &o, test_write_tuple)?;
    export(&mut cx, &o, test_write_enum)?;
    export(&mut cx, &o, test_write_option)?;
    export(&mut cx, &o, test_write_newtype_struct)?;
    export(&mut cx, &o, test_deserialize_number_to_untagged_enum)?;
    export(&mut cx, &o, test_parse_null)?;
    export(&mut cx, &o, test_parse_bool)?;
    export(&mut cx, &o, test_parse_char)?;
    export(&mut cx, &o, test_parse_i64)?;
    export(&mut cx, &o, test_parse_u64)?;
    export(&mut cx, &o, test_parse_f64)?;
    export(&mut cx, &o, test_value_as_f64)?;
    export(&mut cx, &o, test_roundtrip_f64)?;
    export(&mut cx, &o, test_roundtrip_f32)?;
    export(&mut cx, &o, test_serialize_char)?;
    export(&mut cx, &o, test_parse_number)?;
    export(&mut cx, &o, test_parse_string)?;
    export(&mut cx, &o, test_parse_list)?;
    export(&mut cx, &o, test_parse_object)?;
    export(&mut cx, &o, test_parse_struct)?;
    export(&mut cx, &o, test_parse_option)?;
    export(&mut cx, &o, test_parse_enum)?;
    export(&mut cx, &o, test_missing_option_field)?;
    export(&mut cx, &o, test_missing_renamed_field)?;
    export(&mut cx, &o, test_serialize_map_with_no_len)?;
    export(&mut cx, &o, test_serialize_rejects_bool_keys)?;
    export(&mut cx, &o, test_serialize_rejects_adt_keys)?;
    export(&mut cx, &o, test_bytes_ser)?;
    export(&mut cx, &o, test_byte_buf_ser)?;
    export(&mut cx, &o, test_byte_buf_de)?;
    export(&mut cx, &o, test_array_view_de)?;
    export(&mut cx, &o, test_byte_buf_de_multiple)?;
    export(&mut cx, &o, test_deny_float_key)?;
    export(&mut cx, &o, test_effectively_string_keys)?;
    export(&mut cx, &o, issue_220)?;

    Ok(o)
}
