//! Implements a serde serializer for JavaScript values

use serde::{ser, Serialize};

use super::{sys, Error};

#[derive(Clone, Copy)]
#[repr(transparent)]
/// High level deserializer for all JavaScript values
pub(super) struct Serializer {
    env: sys::Env,
}

impl Serializer {
    pub(super) unsafe fn new(env: sys::Env) -> Self {
        Self { env }
    }
}

// Specialized serializer for writing to an `Array`
pub(super) struct ArraySerializer {
    serializer: Serializer,
    value: sys::Value,
    offset: usize,
}

impl ArraySerializer {
    unsafe fn new(serializer: Serializer, value: sys::Value) -> Self {
        Self {
            serializer,
            value,
            offset: 0,
        }
    }
}

// `Array` serializer for externally tagged enum `{ [key]: value }`
pub(super) struct WrappedArraySerializer {
    serializer: ArraySerializer,
    value: sys::Value,
}

impl WrappedArraySerializer {
    unsafe fn new(serializer: ArraySerializer, value: sys::Value) -> Self {
        Self { serializer, value }
    }
}

// Specialized serializer for writing to a generic `Object`
pub(super) struct ObjectSerializer {
    serializer: Serializer,
    value: sys::Value,
    key: Option<sys::Value>,
}

impl ObjectSerializer {
    unsafe fn new(serializer: Serializer, value: sys::Value) -> Self {
        Self {
            serializer,
            value,
            key: None,
        }
    }
}

// `Object` serializer for externally tagged enum `{ [key]: value }`
pub(super) struct WrappedObjectSerializer {
    serializer: ObjectSerializer,
    value: sys::Value,
}

impl WrappedObjectSerializer {
    unsafe fn new(serializer: ObjectSerializer, value: sys::Value) -> Self {
        Self { serializer, value }
    }
}

// Specialized serializer for maps with known fields
pub(super) struct StructSerializer {
    serializer: Serializer,
    value: sys::Value,
}

impl StructSerializer {
    unsafe fn new(serializer: Serializer, value: sys::Value) -> Self {
        Self { serializer, value }
    }
}

// Specialized serializer that understands valid key types
struct KeySerializer {
    serializer: Serializer,
}

impl KeySerializer {
    unsafe fn new(serializer: Serializer) -> Self {
        Self { serializer }
    }
}

impl ser::Serializer for Serializer {
    type Ok = sys::Value;
    type Error = Error;

    // Limited JavaScript types require sequences and tuples to both use `Array`
    type SerializeSeq = ArraySerializer;
    type SerializeTuple = ArraySerializer;
    type SerializeTupleStruct = ArraySerializer;
    type SerializeTupleVariant = WrappedArraySerializer;
    type SerializeMap = ObjectSerializer;
    type SerializeStruct = StructSerializer;
    type SerializeStructVariant = WrappedObjectSerializer;

    fn serialize_bool(self, v: bool) -> Result<Self::Ok, Self::Error> {
        Ok(unsafe { sys::create_bool(self.env, v)? })
    }

    // All numeric types are serialized into `f64`
    fn serialize_i8(self, v: i8) -> Result<Self::Ok, Self::Error> {
        Ok(unsafe { sys::create_double(self.env, v)? })
    }

    fn serialize_i16(self, v: i16) -> Result<Self::Ok, Self::Error> {
        Ok(unsafe { sys::create_double(self.env, v)? })
    }

    fn serialize_i32(self, v: i32) -> Result<Self::Ok, Self::Error> {
        Ok(unsafe { sys::create_double(self.env, v)? })
    }

    // XXX: Precision loss. Support `BigInt`?
    fn serialize_i64(self, v: i64) -> Result<Self::Ok, Self::Error> {
        Ok(unsafe { sys::create_double(self.env, v as f64)? })
    }

    fn serialize_u8(self, v: u8) -> Result<Self::Ok, Self::Error> {
        Ok(unsafe { sys::create_double(self.env, v)? })
    }

    fn serialize_u16(self, v: u16) -> Result<Self::Ok, Self::Error> {
        Ok(unsafe { sys::create_double(self.env, v)? })
    }

    fn serialize_u32(self, v: u32) -> Result<Self::Ok, Self::Error> {
        Ok(unsafe { sys::create_double(self.env, v)? })
    }

    // XXX: Precision loss. Support `BigInt`?
    fn serialize_u64(self, v: u64) -> Result<Self::Ok, Self::Error> {
        Ok(unsafe { sys::create_double(self.env, v as f64)? })
    }

    fn serialize_f32(self, v: f32) -> Result<Self::Ok, Self::Error> {
        Ok(unsafe { sys::create_double(self.env, v)? })
    }

    fn serialize_f64(self, v: f64) -> Result<Self::Ok, Self::Error> {
        Ok(unsafe { sys::create_double(self.env, v)? })
    }

    // `char` are serialized as single character string
    fn serialize_char(self, v: char) -> Result<Self::Ok, Self::Error> {
        Ok(unsafe { sys::create_string(self.env, v.to_string())? })
    }

    fn serialize_str(self, v: &str) -> Result<Self::Ok, Self::Error> {
        Ok(unsafe { sys::create_string(self.env, v)? })
    }

    // Bytes are serialized as `ArrayBuffer`
    fn serialize_bytes(self, v: &[u8]) -> Result<Self::Ok, Self::Error> {
        Ok(unsafe { sys::create_arraybuffer(self.env, v)? })
    }

    // `None` is serialized as a `null`
    fn serialize_none(self) -> Result<Self::Ok, Self::Error> {
        self.serialize_unit()
    }

    // Serialized as the value with no wrapper
    fn serialize_some<T>(self, value: &T) -> Result<Self::Ok, Self::Error>
    where
        T: ?Sized + Serialize,
    {
        value.serialize(self)
    }

    // JavaScript does not have a unit type; `null` is used instead
    fn serialize_unit(self) -> Result<Self::Ok, Self::Error> {
        Ok(unsafe { sys::get_null(self.env)? })
    }

    fn serialize_unit_struct(self, _name: &'static str) -> Result<Self::Ok, Self::Error> {
        self.serialize_unit()
    }

    // Data-less enum are serialized as `string`
    fn serialize_unit_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        variant: &'static str,
    ) -> Result<Self::Ok, Self::Error> {
        self.serialize_str(variant)
    }

    // New-type struct do not include a wrapper; serialized as the inner type
    fn serialize_newtype_struct<T>(
        self,
        _name: &'static str,
        value: &T,
    ) -> Result<Self::Ok, Self::Error>
    where
        T: ?Sized + Serialize,
    {
        value.serialize(self)
    }

    // Serialize as `{ [variant name]: [value] }`
    fn serialize_newtype_variant<T>(
        self,
        _name: &'static str,
        _variant_index: u32,
        variant: &'static str,
        value: &T,
    ) -> Result<Self::Ok, Self::Error>
    where
        T: ?Sized + Serialize,
    {
        unsafe {
            let o = sys::create_object(self.env)?;
            let v = value.serialize(self)?;

            sys::set_named_property(self.env, o, variant, v)?;

            Ok(o)
        }
    }

    fn serialize_seq(self, len: Option<usize>) -> Result<Self::SerializeSeq, Self::Error> {
        let len = len.unwrap_or_default();

        unsafe {
            let value = sys::create_array_with_length(self.env, len)?;

            Ok(ArraySerializer::new(self, value))
        }
    }

    // Tuple `(a, b, ...)` are serialized as array `[a, b, ...]`
    fn serialize_tuple(self, len: usize) -> Result<Self::SerializeTuple, Self::Error> {
        self.serialize_seq(Some(len))
    }

    fn serialize_tuple_struct(
        self,
        _name: &'static str,
        len: usize,
    ) -> Result<Self::SerializeTupleStruct, Self::Error> {
        self.serialize_seq(Some(len))
    }

    // Externally tagged enum; `{ [variant]: value }`
    fn serialize_tuple_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        variant: &'static str,
        len: usize,
    ) -> Result<Self::SerializeTupleVariant, Self::Error> {
        unsafe {
            let env = self.env;
            let wrapper = sys::create_object(env)?;
            let arr = sys::create_array_with_length(env, len)?;
            let serializer = ArraySerializer::new(self, arr);

            sys::set_named_property(env, wrapper, variant, arr)?;

            Ok(WrappedArraySerializer::new(serializer, wrapper))
        }
    }

    fn serialize_map(self, _len: Option<usize>) -> Result<Self::SerializeMap, Self::Error> {
        unsafe {
            let value = sys::create_object(self.env)?;
            Ok(ObjectSerializer::new(self, value))
        }
    }

    fn serialize_struct(
        self,
        _name: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeStruct, Self::Error> {
        unsafe {
            let value = sys::create_object(self.env)?;
            Ok(StructSerializer::new(self, value))
        }
    }

    // Externally tagged enum; `{ [variant]: value }`
    fn serialize_struct_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        variant: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeStructVariant, Self::Error> {
        unsafe {
            let env = self.env;
            let wrapper = sys::create_object(env)?;
            let value = sys::create_object(env)?;
            let serializer = ObjectSerializer::new(self, value);

            sys::set_named_property(env, wrapper, variant, value)?;

            Ok(WrappedObjectSerializer::new(serializer, wrapper))
        }
    }
}

impl ser::SerializeSeq for ArraySerializer {
    type Ok = sys::Value;
    type Error = Error;

    // XXX: Silently truncates with more than 2^32 elements
    fn serialize_element<T>(&mut self, value: &T) -> Result<(), Self::Error>
    where
        T: ?Sized + Serialize,
    {
        let value = value.serialize(self.serializer)?;
        let k = self.offset as u32;

        unsafe { sys::set_element(self.serializer.env, self.value, k, value)? };
        self.offset += 1;

        Ok(())
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        Ok(self.value)
    }
}

impl ser::SerializeTuple for ArraySerializer {
    type Ok = sys::Value;
    type Error = Error;

    fn serialize_element<T>(&mut self, value: &T) -> Result<(), Self::Error>
    where
        T: ?Sized + Serialize,
    {
        ser::SerializeSeq::serialize_element(self, value)
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        ser::SerializeSeq::end(self)
    }
}

impl ser::SerializeTupleStruct for ArraySerializer {
    type Ok = sys::Value;
    type Error = Error;

    fn serialize_field<T>(&mut self, value: &T) -> Result<(), Self::Error>
    where
        T: ?Sized + Serialize,
    {
        ser::SerializeSeq::serialize_element(self, value)
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        ser::SerializeSeq::end(self)
    }
}

impl ser::SerializeTupleVariant for WrappedArraySerializer {
    type Ok = sys::Value;
    type Error = Error;

    fn serialize_field<T>(&mut self, value: &T) -> Result<(), Self::Error>
    where
        T: ?Sized + Serialize,
    {
        ser::SerializeSeq::serialize_element(&mut self.serializer, value)
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        Ok(self.value)
    }
}

impl ser::SerializeMap for ObjectSerializer {
    type Ok = sys::Value;
    type Error = Error;

    fn serialize_key<T>(&mut self, key: &T) -> Result<(), Self::Error>
    where
        T: ?Sized + Serialize,
    {
        self.key = Some(key.serialize(unsafe { KeySerializer::new(self.serializer) })?);
        Ok(())
    }

    fn serialize_value<T>(&mut self, value: &T) -> Result<(), Self::Error>
    where
        T: ?Sized + Serialize,
    {
        let k = self.key.ok_or_else(Error::missing_key)?;
        let v = value.serialize(self.serializer)?;

        unsafe { sys::set_property(self.serializer.env, self.value, k, v)? };

        Ok(())
    }

    fn serialize_entry<K, V>(&mut self, key: &K, value: &V) -> Result<(), Self::Error>
    where
        K: ?Sized + Serialize,
        V: ?Sized + Serialize,
    {
        let k = key.serialize(unsafe { KeySerializer::new(self.serializer) })?;
        let v = value.serialize(self.serializer)?;

        unsafe { sys::set_property(self.serializer.env, self.value, k, v)? };

        Ok(())
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        Ok(self.value)
    }
}

impl ser::SerializeStruct for ObjectSerializer {
    type Ok = sys::Value;
    type Error = Error;

    fn serialize_field<T>(&mut self, key: &'static str, value: &T) -> Result<(), Self::Error>
    where
        T: ?Sized + Serialize,
    {
        ser::SerializeMap::serialize_entry(self, key, value)
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        Ok(self.value)
    }
}

impl ser::SerializeStruct for StructSerializer {
    type Ok = sys::Value;
    type Error = Error;

    fn serialize_field<T>(&mut self, key: &'static str, value: &T) -> Result<(), Self::Error>
    where
        T: ?Sized + Serialize,
    {
        let k = unsafe { sys::get_interned_string(self.serializer.env, key)? };
        let v = value.serialize(self.serializer)?;

        unsafe { sys::set_property(self.serializer.env, self.value, k, v)? };

        Ok(())
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        Ok(self.value)
    }
}

impl ser::SerializeStructVariant for WrappedObjectSerializer {
    type Ok = sys::Value;
    type Error = Error;

    fn serialize_field<T>(&mut self, key: &'static str, value: &T) -> Result<(), Self::Error>
    where
        T: ?Sized + Serialize,
    {
        ser::SerializeMap::serialize_entry(&mut self.serializer, key, value)
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        Ok(self.value)
    }
}

impl ser::Serializer for KeySerializer {
    type Ok = sys::Value;
    type Error = Error;

    type SerializeSeq = ser::Impossible<sys::Value, Error>;
    type SerializeTuple = ser::Impossible<sys::Value, Error>;
    type SerializeTupleStruct = ser::Impossible<sys::Value, Error>;
    type SerializeTupleVariant = ser::Impossible<sys::Value, Error>;
    type SerializeMap = ser::Impossible<sys::Value, Error>;
    type SerializeStruct = ser::Impossible<sys::Value, Error>;
    type SerializeStructVariant = ser::Impossible<sys::Value, Error>;

    fn serialize_bool(self, _v: bool) -> Result<Self::Ok, Self::Error> {
        Err(Error::unsupported_key_type("bool"))
    }

    fn serialize_i8(self, _v: i8) -> Result<Self::Ok, Self::Error> {
        Err(Error::unsupported_key_type("i8"))
    }

    fn serialize_i16(self, _v: i16) -> Result<Self::Ok, Self::Error> {
        Err(Error::unsupported_key_type("i16"))
    }

    fn serialize_i32(self, _v: i32) -> Result<Self::Ok, Self::Error> {
        Err(Error::unsupported_key_type("i32"))
    }

    fn serialize_i64(self, _v: i64) -> Result<Self::Ok, Self::Error> {
        Err(Error::unsupported_key_type("i64"))
    }

    fn serialize_u8(self, _v: u8) -> Result<Self::Ok, Self::Error> {
        Err(Error::unsupported_key_type("u8"))
    }

    fn serialize_u16(self, _v: u16) -> Result<Self::Ok, Self::Error> {
        Err(Error::unsupported_key_type("u16"))
    }

    fn serialize_u32(self, _v: u32) -> Result<Self::Ok, Self::Error> {
        Err(Error::unsupported_key_type("u32"))
    }

    fn serialize_u64(self, _v: u64) -> Result<Self::Ok, Self::Error> {
        Err(Error::unsupported_key_type("u64"))
    }

    fn serialize_f32(self, _v: f32) -> Result<Self::Ok, Self::Error> {
        Err(Error::unsupported_key_type("f32"))
    }

    fn serialize_f64(self, _v: f64) -> Result<Self::Ok, Self::Error> {
        Err(Error::unsupported_key_type("f64"))
    }

    fn serialize_char(self, v: char) -> Result<Self::Ok, Self::Error> {
        self.serializer.serialize_char(v)
    }

    fn serialize_str(self, v: &str) -> Result<Self::Ok, Self::Error> {
        self.serializer.serialize_str(v)
    }

    fn serialize_bytes(self, _v: &[u8]) -> Result<Self::Ok, Self::Error> {
        Err(Error::unsupported_key_type("&[u8]"))
    }

    fn serialize_none(self) -> Result<Self::Ok, Self::Error> {
        Err(Error::unsupported_key_type("none"))
    }

    fn serialize_some<T>(self, _value: &T) -> Result<Self::Ok, Self::Error>
    where
        T: ?Sized + Serialize,
    {
        Err(Error::unsupported_key_type("none"))
    }

    fn serialize_unit(self) -> Result<Self::Ok, Self::Error> {
        Err(Error::unsupported_key_type("()"))
    }

    fn serialize_unit_struct(self, _name: &'static str) -> Result<Self::Ok, Self::Error> {
        Err(Error::unsupported_key_type("()"))
    }

    fn serialize_unit_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        variant: &'static str,
    ) -> Result<Self::Ok, Self::Error> {
        self.serializer.serialize_str(variant)
    }

    fn serialize_newtype_struct<T>(
        self,
        _name: &'static str,
        value: &T,
    ) -> Result<Self::Ok, Self::Error>
    where
        T: ?Sized + Serialize,
    {
        value.serialize(self)
    }

    fn serialize_newtype_variant<T>(
        self,
        _name: &'static str,
        _variant_index: u32,
        _variant: &'static str,
        _value: &T,
    ) -> Result<Self::Ok, Self::Error>
    where
        T: ?Sized + Serialize,
    {
        Err(Error::unsupported_key_type("newtype_variant"))
    }

    fn serialize_seq(self, _len: Option<usize>) -> Result<Self::SerializeSeq, Self::Error> {
        Err(Error::unsupported_key_type("seq"))
    }

    fn serialize_tuple(self, _len: usize) -> Result<Self::SerializeTuple, Self::Error> {
        Err(Error::unsupported_key_type("seq"))
    }

    fn serialize_tuple_struct(
        self,
        _name: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeTupleStruct, Self::Error> {
        Err(Error::unsupported_key_type("tuple struct"))
    }

    fn serialize_tuple_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        _variant: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeTupleVariant, Self::Error> {
        Err(Error::unsupported_key_type("tuple struct variant"))
    }

    fn serialize_map(self, _len: Option<usize>) -> Result<Self::SerializeMap, Self::Error> {
        Err(Error::unsupported_key_type("map"))
    }

    fn serialize_struct(
        self,
        _name: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeStruct, Self::Error> {
        Err(Error::unsupported_key_type("struct"))
    }

    fn serialize_struct_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        _variant: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeStructVariant, Self::Error> {
        Err(Error::unsupported_key_type("struct variant"))
    }
}
