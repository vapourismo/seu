use serde::Serialize;
use serde::ser::SerializeStruct;
use serde::ser::SerializeStructVariant;
use serde::ser::SerializeTupleStruct;
use serde::ser::SerializeTupleVariant;

use crate::core::fields::Field;
use crate::core::fields::FieldsProduct;
use crate::core::names::HasName;
use crate::core::names::Unnamed;
use crate::core::product::Cons;
use crate::core::product::Nil;

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
