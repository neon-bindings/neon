use serde::de;

use crate::{
    context::Context,
    handle::Handle,
    serde::{stringify, sys, Error},
    types::{JsString, Value},
};

pub(super) fn deserialize<'cx, T, V, C>(cx: &mut C, value: Handle<V>) -> Result<T, Error>
where
    T: de::DeserializeOwned + ?Sized,
    V: Value,
    C: Context<'cx>,
{
    let env = cx.env().to_raw();
    let v = value.to_raw();

    match T::deserialize(unsafe { Deserializer::new(env, v) }) {
        Err(Error::FallbackJson) => {}
        res => return res,
    }

    let this = cx.undefined();
    let s = stringify(cx)?
        .call(cx, this, [value.upcast()])?
        .downcast_or_throw::<JsString, _>(cx)?
        .value(cx);

    Ok(serde_json::from_str(&s)?)
}

struct Deserializer {
    env: sys::Env,
    value: sys::Value,
}

impl Deserializer {
    unsafe fn new(env: sys::Env, value: sys::Value) -> Self {
        Self { env, value }
    }
}

#[derive(Debug, Copy, Clone)]
struct Number(f64);

impl Number {
    unsafe fn new(env: sys::Env, value: sys::Value) -> Result<Self, sys::Status> {
        sys::get_value_double(env, value).map(Self)
    }

    fn check_int(self, max: f64) -> Result<f64, Error> {
        let Self(n) = self;

        if n.is_nan() {
            return Err(Error::NaN);
        }

        if n.fract() != 0.0 {
            return Err(Error::NotInt(n));
        }

        if n > max {
            return Err(Error::Overflow(n));
        }

        Ok(n)
    }

    fn check_signed(self, min: f64, max: f64) -> Result<f64, Error> {
        if self.0 < min {
            return Err(Error::Underflow(self.0));
        }

        self.check_int(max)
    }

    fn check_unsigned(self, max: f64) -> Result<f64, Error> {
        if self.0.is_sign_negative() {
            return Err(Error::Underflow(self.0));
        }

        self.check_int(max)
    }

    fn into_u8(self) -> Result<u8, Error> {
        self.check_unsigned(u8::MAX as f64).map(|v| v as u8)
    }

    fn into_u16(self) -> Result<u16, Error> {
        self.check_unsigned(u16::MAX as f64).map(|v| v as u16)
    }

    fn into_u32(self) -> Result<u32, Error> {
        self.check_unsigned(u32::MAX as f64).map(|v| v as u32)
    }

    fn into_u64(self) -> Result<u64, Error> {
        self.check_unsigned(u64::MAX as f64).map(|v| v as u64)
    }

    fn into_u128(self) -> Result<u128, Error> {
        self.check_unsigned(u128::MAX as f64).map(|v| v as u128)
    }

    fn into_i8(self) -> Result<i8, Error> {
        self.check_signed(i8::MIN as f64, i8::MAX as f64)
            .map(|v| v as i8)
    }

    fn into_i16(self) -> Result<i16, Error> {
        self.check_signed(i16::MIN as f64, i16::MAX as f64)
            .map(|v| v as i16)
    }

    fn into_i32(self) -> Result<i32, Error> {
        self.check_signed(i32::MIN as f64, i32::MAX as f64)
            .map(|v| v as i32)
    }

    // FIXME: Does this work?
    fn into_i64(self) -> Result<i64, Error> {
        self.check_signed(i64::MIN as f64, i64::MAX as f64)
            .map(|v| v as i64)
    }

    // FIXME: Does this work?
    fn into_i128(self) -> Result<i128, Error> {
        self.check_signed(i128::MIN as f64, i128::MAX as f64)
            .map(|v| v as i128)
    }
}

impl de::Deserializer<'static> for Deserializer {
    type Error = Error;

    fn deserialize_any<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'static>,
    {
        match unsafe { sys::typeof_value(self.env, self.value)? } {
            sys::ValueType::Undefined => self.deserialize_unit(visitor),
            sys::ValueType::Null => self.deserialize_unit(visitor),
            sys::ValueType::Boolean => self.deserialize_bool(visitor),
            sys::ValueType::Number => {
                let n = unsafe { sys::get_value_double(self.env, self.value)? };

                match (n.fract() == 0.0, n.is_sign_positive()) {
                    (true, true) => visitor.visit_u64(n as u64),
                    (true, false) => visitor.visit_i64(n as i64),
                    _ => visitor.visit_f64(n),
                }
            }
            sys::ValueType::String => self.deserialize_string(visitor),
            sys::ValueType::Object => match self.deserialize_byte_buf(visitor) {
                Err(Error::Status(sys::Status::InvalidArg)) => Err(Error::FallbackJson),
                res => res,
            },
            typ => Err(Error::Unsupported(typ)),
        }
    }

    fn deserialize_bool<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'static>,
    {
        visitor.visit_bool(unsafe { sys::get_value_bool(self.env, self.value)? })
    }

    fn deserialize_i8<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'static>,
    {
        visitor.visit_i8(unsafe { Number::new(self.env, self.value)?.into_i8()? })
    }

    fn deserialize_i16<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'static>,
    {
        visitor.visit_i16(unsafe { Number::new(self.env, self.value)?.into_i16()? })
    }

    fn deserialize_i32<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'static>,
    {
        visitor.visit_i32(unsafe { Number::new(self.env, self.value)?.into_i32()? })
    }

    fn deserialize_i64<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'static>,
    {
        visitor.visit_i64(unsafe { Number::new(self.env, self.value)?.into_i64()? })
    }

    fn deserialize_i128<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'static>,
    {
        visitor.visit_i128(unsafe { Number::new(self.env, self.value)?.into_i128()? })
    }

    fn deserialize_u8<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'static>,
    {
        visitor.visit_u8(unsafe { Number::new(self.env, self.value)?.into_u8()? })
    }

    fn deserialize_u16<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'static>,
    {
        visitor.visit_u16(unsafe { Number::new(self.env, self.value)?.into_u16()? })
    }

    fn deserialize_u32<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'static>,
    {
        visitor.visit_u32(unsafe { Number::new(self.env, self.value)?.into_u32()? })
    }

    fn deserialize_u64<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'static>,
    {
        visitor.visit_u64(unsafe { Number::new(self.env, self.value)?.into_u64()? })
    }

    fn deserialize_u128<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'static>,
    {
        visitor.visit_u128(unsafe { Number::new(self.env, self.value)?.into_u128()? })
    }

    fn deserialize_f32<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'static>,
    {
        visitor.visit_f32(unsafe { sys::get_value_double(self.env, self.value)? as f32 })
    }

    fn deserialize_f64<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'static>,
    {
        visitor.visit_f64(unsafe { sys::get_value_double(self.env, self.value)? })
    }

    fn deserialize_char<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'static>,
    {
        self.deserialize_string(visitor)
    }

    fn deserialize_str<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'static>,
    {
        self.deserialize_string(visitor)
    }

    fn deserialize_string<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'static>,
    {
        visitor.visit_string(unsafe { sys::get_value_string(self.env, self.value)? })
    }

    fn deserialize_bytes<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'static>,
    {
        self.deserialize_byte_buf(visitor)
    }

    fn deserialize_byte_buf<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'static>,
    {
        match unsafe { sys::get_value_arraybuffer(self.env, self.value) } {
            Ok(v) => visitor.visit_byte_buf(v),
            Err(err) if err == sys::Status::InvalidArg => {
                visitor.visit_byte_buf(unsafe { sys::get_value_arrayview(self.env, self.value)? })
            }
            Err(err) => Err(err.into()),
        }
    }

    fn deserialize_option<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'static>,
    {
        match unsafe { sys::typeof_value(self.env, self.value)? } {
            sys::ValueType::Null | sys::ValueType::Undefined => visitor.visit_none(),
            _ => visitor.visit_some(self),
        }
    }

    fn deserialize_unit<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'static>,
    {
        visitor.visit_unit()
    }

    fn deserialize_unit_struct<V>(
        self,
        _name: &'static str,
        visitor: V,
    ) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'static>,
    {
        visitor.visit_unit()
    }

    fn deserialize_newtype_struct<V>(
        self,
        _name: &'static str,
        visitor: V,
    ) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'static>,
    {
        visitor.visit_newtype_struct(self)
    }

    fn deserialize_seq<V>(self, _visitor: V) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'static>,
    {
        Err(Error::FallbackJson)
    }

    fn deserialize_tuple<V>(self, _len: usize, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'static>,
    {
        self.deserialize_seq(visitor)
    }

    fn deserialize_tuple_struct<V>(
        self,
        _name: &'static str,
        _len: usize,
        visitor: V,
    ) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'static>,
    {
        self.deserialize_seq(visitor)
    }

    fn deserialize_map<V>(self, _visitor: V) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'static>,
    {
        Err(Error::FallbackJson)
    }

    fn deserialize_struct<V>(
        self,
        _name: &'static str,
        _fields: &'static [&'static str],
        _visitor: V,
    ) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'static>,
    {
        Err(Error::FallbackJson)
    }

    fn deserialize_enum<V>(
        self,
        _name: &'static str,
        _variants: &'static [&'static str],
        visitor: V,
    ) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'static>,
    {
        // No-value enums are serialized as `string`
        if let Ok(s) = unsafe { sys::get_value_string(self.env, self.value) } {
            visitor.visit_enum(de::IntoDeserializer::into_deserializer(s))
        } else {
            Err(Error::FallbackJson)
        }
    }

    fn deserialize_identifier<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'static>,
    {
        self.deserialize_string(visitor)
    }

    fn deserialize_ignored_any<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'static>,
    {
        visitor.visit_unit()
    }
}
