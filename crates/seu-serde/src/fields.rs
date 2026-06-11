use alloc::borrow::ToOwned;
use alloc::fmt;
use alloc::string::String;
use alloc::vec::Vec;
use core::marker::PhantomData;

use serde::Deserialize;
use serde::Serialize;
use serde::de::IgnoredAny;
use serde::de::MapAccess;
use serde::de::SeqAccess;
use serde::de::Unexpected;
use serde::de::Visitor;
use serde::ser::SerializeStruct;
use serde::ser::SerializeStructVariant;
use serde::ser::SerializeTupleStruct;
use serde::ser::SerializeTupleVariant;
use seu_core::fields::Field;
use seu_core::fields::FieldsProduct;
use seu_core::names::HasName;
use seu_core::names::HasOptionalName;
use seu_core::names::Unnamed;
use seu_core::product::Cons;
use seu_core::product::Nil;
use seu_core::product::Product;

pub trait DeserializeTupleFields<'de>: FieldsProduct + Sized {
    fn deserialize_unit_fields<E: serde::de::Error>() -> Result<Self, E>;

    fn deserialize_tuple_fields<A: SeqAccess<'de>>(
        seq: &mut A,
        index: usize,
    ) -> Result<Self, A::Error>;
}

impl<'de> DeserializeTupleFields<'de> for Nil {
    #[inline]
    fn deserialize_unit_fields<E: serde::de::Error>() -> Result<Self, E> {
        Ok(Nil)
    }

    #[inline]
    fn deserialize_tuple_fields<A: SeqAccess<'de>>(
        seq: &mut A,
        index: usize,
    ) -> Result<Self, A::Error> {
        // There should be no further items in the sequence.
        if seq.next_element::<IgnoredAny>()?.is_some() {
            return Err(serde::de::Error::invalid_length(
                index + 1,
                &core::any::type_name::<Self>(),
            ));
        }

        Ok(Nil)
    }
}

impl<'de, MaybeName, Type, Tail> DeserializeTupleFields<'de> for Cons<Field<MaybeName, Type>, Tail>
where
    MaybeName: HasOptionalName,
    Type: Deserialize<'de>,
    Tail: DeserializeTupleFields<'de>,
{
    #[inline]
    fn deserialize_unit_fields<E: serde::de::Error>() -> Result<Self, E> {
        Err(serde::de::Error::invalid_type(
            Unexpected::Unit,
            &core::any::type_name::<Self>(),
        ))
    }

    #[inline]
    fn deserialize_tuple_fields<A: SeqAccess<'de>>(
        seq: &mut A,
        index: usize,
    ) -> Result<Self, A::Error> {
        let value = seq.next_element()?.ok_or_else(|| {
            serde::de::Error::invalid_length(index, &core::any::type_name::<Self>())
        })?;
        let tail = Tail::deserialize_tuple_fields(seq, index + 1)?;
        Ok(Cons(Field::new(value), tail))
    }
}

pub struct TupleFieldsVisitor<Name, Fields> {
    _name: PhantomData<Name>,
    _fields: PhantomData<Fields>,
}

impl<Name, Fields> TupleFieldsVisitor<Name, Fields> {
    pub fn new() -> Self {
        Self {
            _name: PhantomData,
            _fields: PhantomData,
        }
    }
}

impl<'de, Name: HasName, Fields: DeserializeTupleFields<'de>> Visitor<'de>
    for TupleFieldsVisitor<Name, Fields>
{
    type Value = Fields;

    fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(Name::NAME)
    }

    fn visit_seq<A: SeqAccess<'de>>(self, mut seq: A) -> Result<Self::Value, A::Error> {
        Fields::deserialize_tuple_fields(&mut seq, 0)
    }

    fn visit_unit<E: serde::de::Error>(self) -> Result<Self::Value, E> {
        Fields::deserialize_unit_fields()
    }
}

pub enum HandledField {
    Unknown,
    Handled,
}

pub trait DeserializeStructFields<'de>: DeserializeTupleFields<'de> {
    type Interim: Product;

    fn start() -> Self::Interim;

    fn finalize<E: serde::de::Error>(interim: Self::Interim) -> Result<Self, E>;

    fn handle_field<A: MapAccess<'de>>(
        interim: &mut Self::Interim,
        map: &mut A,
        field: &FieldIdx,
        idx: u64,
    ) -> Result<HandledField, A::Error>;
}

impl<'de> DeserializeStructFields<'de> for Nil {
    type Interim = Nil;

    #[inline]
    fn start() -> Self::Interim {
        Nil
    }

    #[inline]
    fn finalize<E: serde::de::Error>(Nil: Self::Interim) -> Result<Self, E> {
        Ok(Nil)
    }

    fn handle_field<A: MapAccess<'de>>(
        _interim: &mut Self::Interim,
        _map: &mut A,
        _field: &FieldIdx,
        _idx: u64,
    ) -> Result<HandledField, A::Error> {
        Ok(HandledField::Unknown)
    }
}

impl<'de, Name, Type, Tail> DeserializeStructFields<'de> for Cons<Field<Name, Type>, Tail>
where
    Name: HasName,
    Type: Deserialize<'de>,
    Tail: DeserializeStructFields<'de>,
{
    type Interim = Cons<Option<Type>, Tail::Interim>;

    #[inline]
    fn start() -> Self::Interim {
        Cons(None, Tail::start())
    }

    #[inline]
    fn finalize<E: serde::de::Error>(Cons(head, tail): Self::Interim) -> Result<Self, E> {
        let Some(head) = head else {
            return Err(serde::de::Error::missing_field(Name::NAME));
        };

        let tail = Tail::finalize(tail)?;
        let res = Cons(Field::new(head), tail);

        Ok(res)
    }

    fn handle_field<A: MapAccess<'de>>(
        Cons(head, tail): &mut Self::Interim,
        map: &mut A,
        field: &FieldIdx,
        idx: u64,
    ) -> Result<HandledField, A::Error> {
        match field {
            FieldIdx::Index(field_idx) if idx == *field_idx => {}
            FieldIdx::String(field_name) if field_name == Name::NAME => {}
            FieldIdx::Bytes(field_name) if field_name == Name::NAME.as_bytes() => {}
            _ => {
                return Tail::handle_field(tail, map, field, idx + 1);
            }
        }

        if head.is_some() {
            return Err(serde::de::Error::duplicate_field(Name::NAME));
        }

        let value: Type = map.next_value()?;
        *head = Some(value);

        Ok(HandledField::Handled)
    }
}

pub struct StructFieldsVisitor<Name, Fields> {
    _name: PhantomData<Name>,
    _fields: PhantomData<Fields>,
}

impl<Name, Fields> StructFieldsVisitor<Name, Fields> {
    pub fn new() -> Self {
        Self {
            _name: PhantomData,
            _fields: PhantomData,
        }
    }
}

impl<'de, Name: HasName, Fields: DeserializeStructFields<'de>> Visitor<'de>
    for StructFieldsVisitor<Name, Fields>
{
    type Value = Fields;

    fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(Name::NAME)
    }

    fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
    where
        A: SeqAccess<'de>,
    {
        Fields::deserialize_tuple_fields(&mut seq, 0)
    }

    fn visit_map<A: MapAccess<'de>>(self, mut map: A) -> Result<Self::Value, A::Error> {
        let mut interim = Fields::start();

        while let Some(field) = map.next_key()? {
            let res = Fields::handle_field(&mut interim, &mut map, &field, 0)?;

            if let HandledField::Unknown = res {
                map.next_value::<IgnoredAny>()?;
            }
        }

        Fields::finalize(interim)
    }
}

pub trait SerializeTupleVariantFields: FieldsProduct {
    fn serialize_tuple_fields<S: SerializeTupleVariant>(
        &self,
        serializer: S,
    ) -> Result<S::Ok, S::Error>;
}

impl SerializeTupleVariantFields for Nil {
    fn serialize_tuple_fields<S: SerializeTupleVariant>(
        &self,
        serializer: S,
    ) -> Result<S::Ok, S::Error> {
        serializer.end()
    }
}

impl<Type, Tail> SerializeTupleVariantFields for Cons<Field<Unnamed, Type>, Tail>
where
    Type: Serialize,
    Tail: SerializeTupleVariantFields,
{
    fn serialize_tuple_fields<S: SerializeTupleVariant>(
        &self,
        mut serializer: S,
    ) -> Result<S::Ok, S::Error> {
        serializer.serialize_field(&self.0.value)?;
        Tail::serialize_tuple_fields(&self.1, serializer)
    }
}

pub trait SerializeStructVariantFields: FieldsProduct {
    fn serialize_struct_fields<S: SerializeStructVariant>(
        &self,
        serializer: S,
    ) -> Result<S::Ok, S::Error>;
}

impl SerializeStructVariantFields for Nil {
    fn serialize_struct_fields<S: SerializeStructVariant>(
        &self,
        serializer: S,
    ) -> Result<S::Ok, S::Error> {
        serializer.end()
    }
}

impl<Name, Type, Tail> SerializeStructVariantFields for Cons<Field<Name, Type>, Tail>
where
    Name: HasName,
    Type: Serialize,
    Tail: SerializeStructVariantFields,
{
    fn serialize_struct_fields<S: SerializeStructVariant>(
        &self,
        mut serializer: S,
    ) -> Result<S::Ok, S::Error> {
        serializer.serialize_field(Name::NAME, &self.0.value)?;
        Tail::serialize_struct_fields(&self.1, serializer)
    }
}

pub trait SerializeTupleStructFields: FieldsProduct {
    fn serialize_tuple_struct_fields<S: SerializeTupleStruct>(
        &self,
        serializer: S,
    ) -> Result<S::Ok, S::Error>;
}

impl SerializeTupleStructFields for Nil {
    fn serialize_tuple_struct_fields<S: SerializeTupleStruct>(
        &self,
        serializer: S,
    ) -> Result<S::Ok, S::Error> {
        serializer.end()
    }
}

impl<Type, Tail> SerializeTupleStructFields for Cons<Field<Unnamed, Type>, Tail>
where
    Type: Serialize,
    Tail: SerializeTupleStructFields,
{
    fn serialize_tuple_struct_fields<S: SerializeTupleStruct>(
        &self,
        mut serializer: S,
    ) -> Result<S::Ok, S::Error> {
        serializer.serialize_field(&self.0.value)?;
        Tail::serialize_tuple_struct_fields(&self.1, serializer)
    }
}

pub trait SerializeStructFields: FieldsProduct {
    fn serialize_struct_fields<S: SerializeStruct>(&self, serializer: S)
    -> Result<S::Ok, S::Error>;
}

impl SerializeStructFields for Nil {
    fn serialize_struct_fields<S: SerializeStruct>(
        &self,
        serializer: S,
    ) -> Result<S::Ok, S::Error> {
        serializer.end()
    }
}

impl<Name, Type, Tail> SerializeStructFields for Cons<Field<Name, Type>, Tail>
where
    Name: HasName,
    Type: Serialize,
    Tail: SerializeStructFields,
{
    fn serialize_struct_fields<S: SerializeStruct>(
        &self,
        mut serializer: S,
    ) -> Result<S::Ok, S::Error> {
        serializer.serialize_field(Name::NAME, &self.0.value)?;
        Tail::serialize_struct_fields(&self.1, serializer)
    }
}

#[derive(Debug, Clone)]
pub enum FieldIdx {
    Index(u64),
    String(String),
    Bytes(Vec<u8>),
}

impl<'de> Deserialize<'de> for FieldIdx {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        struct Visit;

        impl Visitor<'_> for Visit {
            type Value = FieldIdx;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("field")
            }

            fn visit_u64<E>(self, v: u64) -> Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                Ok(FieldIdx::Index(v))
            }

            fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                Ok(FieldIdx::String(v.to_owned()))
            }

            fn visit_bytes<E>(self, v: &[u8]) -> Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                Ok(FieldIdx::Bytes(v.to_owned()))
            }
        }

        deserializer.deserialize_identifier(Visit)
    }
}
