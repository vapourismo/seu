use alloc::fmt;
use core::cell::RefCell;
use core::marker::PhantomData;

use serde::Deserializer;
use serde::Serializer;
use serde::de::EnumAccess;
use serde::de::Visitor;
use seu_core::enums::Enum;
use seu_core::names::HasName;
use seu_core::names::HasNames;
use seu_core::product::Nil;
use seu_core::structs::Struct;
use seu_core::variants::StructVariant;
use seu_core::variants::TupleVariant;
use seu_core::variants::UnitVariant;

use crate::fields::DeserializeStructFields;
use crate::fields::DeserializeTupleFields;
use crate::fields::SerializeStructFields;
use crate::fields::SerializeTupleStructFields;
use crate::fields::StructFieldsVisitor;
use crate::fields::TupleFieldsVisitor;
use crate::variants::DeserializeVariants;
use crate::variants::SerializeVariants;
use crate::variants::VariantIdx;

pub trait SerializeRepr {
    fn serialize_repr<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error>;
}

pub trait DeserializeRepr<'de>: Sized {
    fn deserialize_repr<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error>;
}

impl<Name, VariantNames, Variants> SerializeRepr for Enum<Name, VariantNames, Variants>
where
    Name: HasName,
    VariantNames: HasNames,
    Variants: SerializeVariants,
{
    fn serialize_repr<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let serializer = RefCell::new(Some(serializer));
        let handlers = Variants::eager_handlers::<S>(&serializer);
        self.variant.reduce_ref(handlers)
    }
}

impl<'de, Name, VariantNames, Variants> DeserializeRepr<'de> for Enum<Name, VariantNames, Variants>
where
    Name: HasName,
    VariantNames: HasNames,
    Variants: DeserializeVariants<'de, Variants>,
{
    fn deserialize_repr<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        struct EnumVisitor<Name, VariantNames, Variants>(
            PhantomData<Name>,
            PhantomData<VariantNames>,
            PhantomData<Variants>,
        );

        impl<'de, Name, VariantNames, Variants> Visitor<'de> for EnumVisitor<Name, VariantNames, Variants>
        where
            Name: HasName,
            VariantNames: HasNames,
            Variants: DeserializeVariants<'de, Variants>,
        {
            type Value = Enum<Name, VariantNames, Variants>;

            fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
                formatter.write_str(Name::NAME)
            }

            fn visit_enum<A: EnumAccess<'de>>(self, data: A) -> Result<Self::Value, A::Error> {
                let (variant, access) = data.variant::<VariantIdx>()?;
                Variants::deserialize_variant(variant, access).map(Enum::new)
            }
        }

        deserializer.deserialize_enum(
            Name::NAME,
            VariantNames::NAMES,
            EnumVisitor::<Name, VariantNames, Variants>(PhantomData, PhantomData, PhantomData),
        )
    }
}

impl<Name, Fields> SerializeRepr for Struct<Name, TupleVariant, Fields>
where
    Name: HasName,
    Fields: SerializeTupleStructFields,
{
    fn serialize_repr<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let serializer = serializer.serialize_tuple_struct(Name::NAME, Fields::LEN)?;
        self.fields.serialize_tuple_struct_fields(serializer)
    }
}

impl<'de, Name, Fields> DeserializeRepr<'de> for Struct<Name, TupleVariant, Fields>
where
    Name: HasName,
    Fields: DeserializeTupleFields<'de>,
{
    fn deserialize_repr<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        deserializer
            .deserialize_tuple_struct(
                Name::NAME,
                Fields::LEN,
                TupleFieldsVisitor::<Name, Fields>::new(),
            )
            .map(Struct::new)
    }
}

impl<Name, FieldNames, Fields> SerializeRepr for Struct<Name, StructVariant<FieldNames>, Fields>
where
    Name: HasName,
    FieldNames: HasNames,
    Fields: SerializeStructFields,
{
    fn serialize_repr<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let serializer = serializer.serialize_struct(Name::NAME, Fields::LEN)?;
        self.fields.serialize_struct_fields(serializer)
    }
}

impl<'de, Name, FieldNames, Fields> DeserializeRepr<'de>
    for Struct<Name, StructVariant<FieldNames>, Fields>
where
    Name: HasName,
    FieldNames: HasNames,
    Fields: DeserializeStructFields<'de>,
{
    fn deserialize_repr<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        deserializer
            .deserialize_struct(
                Name::NAME,
                FieldNames::NAMES,
                StructFieldsVisitor::<Name, Fields>::new(),
            )
            .map(Struct::new)
    }
}

impl<Name: HasName> SerializeRepr for Struct<Name, UnitVariant, Nil> {
    fn serialize_repr<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_unit_struct(Name::NAME)
    }
}

impl<'de, Name: HasName> DeserializeRepr<'de> for Struct<Name, UnitVariant, Nil> {
    fn deserialize_repr<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        struct UnitStructVisitor<Name: HasName>(PhantomData<Name>);

        impl<'de, Name: HasName> Visitor<'de> for UnitStructVisitor<Name> {
            type Value = Struct<Name, UnitVariant, Nil>;

            fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
                formatter.write_str(Name::NAME)
            }

            fn visit_unit<E: serde::de::Error>(self) -> Result<Self::Value, E> {
                Ok(Struct::new(Nil))
            }
        }

        deserializer.deserialize_unit_struct(Name::NAME, UnitStructVisitor::<Name>(PhantomData))
    }
}
