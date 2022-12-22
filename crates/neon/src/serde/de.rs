//! Implements a serde deserializer for JavaScript values

use std::slice;

use serde::de::{
    self, DeserializeSeed, EnumAccess, IntoDeserializer, MapAccess, SeqAccess, VariantAccess,
    Visitor,
};

use super::{sys, Error};

#[derive(Debug)]
/// High level deserializer for all JavaScript
pub(super) struct Deserializer {
    env: sys::Env,
    value: sys::Value,
}

impl Deserializer {
    pub(super) unsafe fn new(env: sys::Env, value: sys::Value) -> Self {
        Deserializer { env, value }
    }
}

#[derive(Debug)]
/// Specialized deserializer for `Array`
pub(super) struct ArrayAccessor {
    env: sys::Env,
    array: sys::Value,
    len: u32,
    index: u32,
}

impl ArrayAccessor {
    unsafe fn new(env: sys::Env, array: sys::Value) -> Result<Self, Error> {
        Ok(Self::new_with_length(
            env,
            array,
            sys::get_array_length(env, array)?,
        ))
    }

    unsafe fn new_with_length(env: sys::Env, array: sys::Value, len: u32) -> Self {
        Self {
            env,
            array,
            len,
            index: 0,
        }
    }

    unsafe fn next(&mut self) -> Result<Option<sys::Value>, Error> {
        if self.index >= self.len {
            return Ok(None);
        }

        let element = sys::get_element(self.env, self.array, self.index)?;

        self.index += 1;

        Ok(Some(element))
    }
}

#[derive(Debug)]
/// Specialized deserializer for generic `Object`
/// Only enumerable keys are read
pub(super) struct ObjectAccessor {
    env: sys::Env,
    object: sys::Value,
    keys: ArrayAccessor,
    // Store the most recent key for reading the next value
    next: Option<sys::Value>,
}

impl ObjectAccessor {
    unsafe fn new(env: sys::Env, object: sys::Value) -> Result<Self, Error> {
        let keys = sys::get_property_names(env, object)?;
        let keys = ArrayAccessor::new(env, keys)?;

        Ok(Self {
            env,
            object,
            keys,
            next: None,
        })
    }
}

#[derive(Debug)]
/// Specialized deserializer for `Object` with known keys
struct StructObjectAccessor {
    env: sys::Env,
    object: sys::Value,
    keys: slice::Iter<'static, &'static str>,
    // Store the most recent key for reading the next value
    next: Option<&'static str>,
}

impl StructObjectAccessor {
    unsafe fn new(env: sys::Env, object: sys::Value, keys: &'static [&'static str]) -> Self {
        Self {
            env,
            object,
            keys: keys.iter(),
            next: None,
        }
    }
}

impl de::Deserializer<'static> for Deserializer {
    type Error = Error;

    // JavaScript is a self describing format, allowing us to provide a deserialization
    // implementation without prior knowledge of the schema. This is useful for types
    // like `serde_json::Value`.
    fn deserialize_any<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'static>,
    {
        match unsafe { sys::typeof_value(self.env, self.value)? } {
            sys::ValueType::Undefined | sys::ValueType::Null => visitor.visit_unit(),
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
            sys::ValueType::Object => {
                if unsafe { sys::is_array(self.env, self.value)? } {
                    visitor.visit_seq(unsafe { ArrayAccessor::new(self.env, self.value)? })
                } else {
                    visitor.visit_map(unsafe { ObjectAccessor::new(self.env, self.value)? })
                }
            }
            typ => Err(Error::unsupported_type(typ)),
        }
    }

    fn deserialize_bool<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'static>,
    {
        visitor.visit_bool(unsafe { sys::get_value_bool(self.env, self.value)? })
    }

    // XXX: JavaScript only provides an `f64` number type. All integer types
    // will truncate fractional values when deserializing.
    fn deserialize_i8<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'static>,
    {
        let n = unsafe { sys::get_value_double(self.env, self.value)? };

        visitor.visit_i8(n as i8)
    }

    fn deserialize_i16<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'static>,
    {
        let n = unsafe { sys::get_value_double(self.env, self.value)? };

        visitor.visit_i16(n as i16)
    }

    fn deserialize_i32<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'static>,
    {
        let n = unsafe { sys::get_value_double(self.env, self.value)? };

        visitor.visit_i32(n as i32)
    }

    fn deserialize_i64<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'static>,
    {
        let n = unsafe { sys::get_value_double(self.env, self.value)? };

        visitor.visit_i64(n as i64)
    }

    // XXX: Deserializing a negative number as unsigned will wrap
    fn deserialize_u8<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'static>,
    {
        let n = unsafe { sys::get_value_double(self.env, self.value)? };

        visitor.visit_u8(n as u8)
    }

    fn deserialize_u16<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'static>,
    {
        let n = unsafe { sys::get_value_double(self.env, self.value)? };

        visitor.visit_u16(n as u16)
    }

    fn deserialize_u32<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'static>,
    {
        let n = unsafe { sys::get_value_double(self.env, self.value)? };

        visitor.visit_u32(n as u32)
    }

    fn deserialize_u64<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'static>,
    {
        let n = unsafe { sys::get_value_double(self.env, self.value)? };

        visitor.visit_u64(n as u64)
    }

    fn deserialize_f32<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'static>,
    {
        let n = unsafe { sys::get_value_double(self.env, self.value)? };

        visitor.visit_f32(n as f32)
    }

    fn deserialize_f64<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'static>,
    {
        let n = unsafe { sys::get_value_double(self.env, self.value)? };

        visitor.visit_f64(n)
    }

    // `char` are serialized as a single character `string`
    fn deserialize_char<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'static>,
    {
        self.deserialize_string(visitor)
    }

    fn deserialize_str<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'static>,
    {
        self.deserialize_string(visitor)
    }

    fn deserialize_string<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'static>,
    {
        visitor.visit_string(unsafe { sys::get_value_string(self.env, self.value)? })
    }

    fn deserialize_bytes<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'static>,
    {
        self.deserialize_byte_buf(visitor)
    }

    // Bytes are serialized as the idiomatic `ArrayBuffer` JavaScript type
    // FIXME: This should support array views
    fn deserialize_byte_buf<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'static>,
    {
        match unsafe { sys::get_value_arraybuffer(self.env, self.value) } {
            Ok(v) => visitor.visit_byte_buf(v),
            Err(err) if err == sys::Status::InvalidArg => {
                visitor.visit_byte_buf(unsafe { sys::get_value_arrayview(self.env, self.value)? })
            }
            Err(err) => Err(err.into()),
        }
    }

    // `None` are serialized as `null`, but when deserializing `undefined` is
    // also accepted.
    fn deserialize_option<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'static>,
    {
        match unsafe { sys::typeof_value(self.env, self.value)? } {
            sys::ValueType::Null | sys::ValueType::Undefined => visitor.visit_none(),
            _ => visitor.visit_some(self),
        }
    }

    fn deserialize_unit<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'static>,
    {
        // Since this is a transcoder, we need to do anything to consume
        visitor.visit_unit()
    }

    fn deserialize_unit_struct<V>(
        self,
        _name: &'static str,
        visitor: V,
    ) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'static>,
    {
        visitor.visit_unit()
    }

    fn deserialize_newtype_struct<V>(
        self,
        _name: &'static str,
        visitor: V,
    ) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'static>,
    {
        visitor.visit_newtype_struct(self)
    }

    // `Array` is used since it is the only sequence type in JavaScript
    fn deserialize_seq<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'static>,
    {
        match unsafe { ArrayAccessor::new(self.env, self.value) } {
            Ok(accessor) => visitor.visit_seq(accessor),
            Err(err) if err.is_array_expected() => self.deserialize_any(visitor),
            Err(err) => Err(err),
        }
    }

    // `Array` are used to serialize tuples; this is a common pattern, especially in TypeScript
    fn deserialize_tuple<V>(self, _len: usize, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'static>,
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
        V: Visitor<'static>,
    {
        self.deserialize_seq(visitor)
    }

    // Generic `Object` are used to serialize map
    fn deserialize_map<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'static>,
    {
        // FIXME: Optimize this for `Object`
        self.deserialize_any(visitor)
    }

    fn deserialize_struct<V>(
        self,
        _name: &'static str,
        _fields: &'static [&'static str],
        visitor: V,
    ) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'static>,
    {
        // FIXME: Optimize this for known fields
        self.deserialize_map(visitor)
    }

    fn deserialize_enum<V>(
        self,
        _name: &'static str,
        _variants: &'static [&'static str],
        visitor: V,
    ) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'static>,
    {
        // No-value enums are serialized as `string`
        if let Ok(s) = unsafe { sys::get_value_string(self.env, self.value) } {
            visitor.visit_enum(s.into_deserializer())
        } else {
            visitor.visit_enum(self)
        }
    }

    fn deserialize_identifier<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'static>,
    {
        self.deserialize_string(visitor)
    }

    fn deserialize_ignored_any<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'static>,
    {
        visitor.visit_unit()
    }
}

impl SeqAccess<'static> for ArrayAccessor {
    type Error = Error;

    // This will have unpredictable results if the `Array` has a getter that mutates
    // the object. It should be _safe_ and return an `Error`, but hopefully users
    // don't do this.
    fn next_element_seed<T>(&mut self, seed: T) -> Result<Option<T::Value>, Self::Error>
    where
        T: DeserializeSeed<'static>,
    {
        unsafe { self.next()? }
            .map(|v| seed.deserialize(unsafe { Deserializer::new(self.env, v) }))
            .transpose()
    }

    // We can efficiently provide a size hint since `Array` have known length
    fn size_hint(&self) -> Option<usize> {
        Some((self.len - self.index) as usize)
    }
}

impl MapAccess<'static> for ObjectAccessor {
    type Error = Error;

    // This will have unpredictable results if the `Object` has a getter that mutates
    // the object. It should be _safe_ and return an `Error`, but hopefully users
    // don't do this on serializable types.
    fn next_key_seed<K>(&mut self, seed: K) -> Result<Option<K::Value>, Self::Error>
    where
        K: DeserializeSeed<'static>,
    {
        // Store the next `key` for deserializing the value in `next_value_seed`
        self.next = unsafe { self.keys.next()? };
        self.next
            .map(|v| seed.deserialize(unsafe { Deserializer::new(self.env, v) }))
            .transpose()
    }

    fn next_value_seed<V>(&mut self, seed: V) -> Result<V::Value, Self::Error>
    where
        V: DeserializeSeed<'static>,
    {
        // `Error::missing_key` should only happen in a buggy serde implementation
        let key = self.next.ok_or_else(Error::missing_key)?;
        let value = unsafe { sys::get_property(self.env, self.object, key)? };

        seed.deserialize(unsafe { Deserializer::new(self.env, value) })
    }

    // We can efficiently provide a size hint since we fetch all keys ahead of time
    fn size_hint(&self) -> Option<usize> {
        self.keys.size_hint()
    }
}

impl MapAccess<'static> for StructObjectAccessor {
    type Error = Error;

    fn next_key_seed<K>(&mut self, seed: K) -> Result<Option<K::Value>, Self::Error>
    where
        K: DeserializeSeed<'static>,
    {
        // Store the next `key` for deserializing the value in `next_value_seed`
        self.next = self.keys.next().copied();
        self.next
            .map(|v| seed.deserialize(de::value::StrDeserializer::new(v)))
            .transpose()
    }

    fn next_value_seed<V>(&mut self, seed: V) -> Result<V::Value, Self::Error>
    where
        V: DeserializeSeed<'static>,
    {
        // `Error::missing_key` should only happen in a buggy serde implementation
        let key = self.next.ok_or_else(Error::missing_key)?;
        let value = unsafe { sys::get_named_property(self.env, self.object, key)? };

        seed.deserialize(unsafe { Deserializer::new(self.env, value) })
    }

    // We can efficiently provide a size hint since we fetch all keys ahead of time
    fn size_hint(&self) -> Option<usize> {
        self.keys.size_hint().1
    }
}

impl EnumAccess<'static> for Deserializer {
    type Error = Error;
    type Variant = Self;

    // Enums are serialized as `{ [type]: value }`
    fn variant_seed<V>(self, seed: V) -> Result<(V::Value, Self::Variant), Self::Error>
    where
        V: DeserializeSeed<'static>,
    {
        let keys = unsafe { sys::get_property_names(self.env, self.value)? };
        let key = unsafe { sys::get_element(self.env, keys, 0)? };
        let value = unsafe { sys::get_property(self.env, self.value, key)? };
        let deserializer = unsafe { Deserializer::new(self.env, value) };
        let key = seed.deserialize(unsafe { Deserializer::new(self.env, key) })?;

        Ok((key, deserializer))
    }
}

// Externally tagged enum can be treated equivalent to the enclosed type
impl VariantAccess<'static> for Deserializer {
    type Error = Error;

    fn unit_variant(self) -> Result<(), Self::Error> {
        Ok(())
    }

    fn newtype_variant_seed<T>(self, seed: T) -> Result<T::Value, Self::Error>
    where
        T: DeserializeSeed<'static>,
    {
        seed.deserialize(self)
    }

    fn tuple_variant<V>(self, len: usize, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'static>,
    {
        visitor
            .visit_seq(unsafe { ArrayAccessor::new_with_length(self.env, self.value, len as u32) })
    }

    fn struct_variant<V>(
        self,
        fields: &'static [&'static str],
        visitor: V,
    ) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'static>,
    {
        visitor.visit_map(unsafe { StructObjectAccessor::new(self.env, self.value, fields) })
    }
}
