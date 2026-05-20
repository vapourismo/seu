use alloc::fmt;
use core::cell::RefCell;
use core::marker::PhantomData;

use serde::Deserializer;
use serde::Serializer;
use serde::de::EnumAccess;
use serde::de::Visitor;
use seu_core::names::HasNames;

use crate::core::datatypes::DataType;
use crate::core::enums::Enum;
use crate::core::product::Nil;
use crate::core::structs::Struct;
use crate::core::variants::StructVariant;
use crate::core::variants::TupleVariant;
use crate::core::variants::UnitVariant;
use crate::serde::fields::DeserializeStructFields;
use crate::serde::fields::DeserializeTupleFields;
use crate::serde::fields::SerializeStructFields;
use crate::serde::fields::SerializeTupleStructFields;
use crate::serde::fields::StructFieldsVisitor;
use crate::serde::fields::TupleFieldsVisitor;
use crate::serde::variants::DeserializeVariants;
use crate::serde::variants::SerializeVariants;
use crate::serde::variants::VariantIdx;

pub trait SerializeRepr<Parent: DataType> {
    fn serialize_repr<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error>;
}

pub trait DeserializeRepr<'de, Parent: DataType>: Sized {
    fn deserialize_repr<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error>;
}

impl<Parent, VariantNames, Variants> SerializeRepr<Parent> for Enum<VariantNames, Variants>
where
    Parent: DataType,
    VariantNames: HasNames,
    Variants: SerializeVariants<Parent>,
{
    fn serialize_repr<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let serializer = RefCell::new(Some(serializer));
        let handlers = Variants::eager_handlers::<S>(&serializer);
        self.variant.reduce_ref(handlers)
    }
}

impl<'de, Parent, VariantNames, Variants> DeserializeRepr<'de, Parent>
    for Enum<VariantNames, Variants>
where
    Parent: DataType,
    VariantNames: HasNames,
    Variants: DeserializeVariants<'de, Parent, Variants>,
{
    fn deserialize_repr<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        struct EnumVisitor<Parent, VariantNames, Variants>(
            PhantomData<Parent>,
            PhantomData<VariantNames>,
            PhantomData<Variants>,
        );

        impl<'de, Parent, VariantNames, Variants> Visitor<'de>
            for EnumVisitor<Parent, VariantNames, Variants>
        where
            Parent: DataType,
            VariantNames: HasNames,
            Variants: DeserializeVariants<'de, Parent, Variants>,
        {
            type Value = Enum<VariantNames, Variants>;

            fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
                formatter.write_str(Parent::TYPE_NAME)
            }

            fn visit_enum<A: EnumAccess<'de>>(self, data: A) -> Result<Self::Value, A::Error> {
                let (variant, access) = data.variant::<VariantIdx>()?;
                Variants::deserialize_variant(variant, access).map(Enum::new)
            }
        }

        deserializer.deserialize_enum(
            Parent::TYPE_NAME,
            VariantNames::NAMES,
            EnumVisitor::<Parent, VariantNames, Variants>(PhantomData, PhantomData, PhantomData),
        )
    }
}

impl<Parent, Fields> SerializeRepr<Parent> for Struct<TupleVariant, Fields>
where
    Parent: DataType,
    Fields: SerializeTupleStructFields,
{
    fn serialize_repr<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let serializer = serializer.serialize_tuple_struct(Parent::TYPE_NAME, Fields::LEN)?;
        self.fields.serialize_tuple_struct_fields(serializer)
    }
}

impl<'de, Parent, Fields> DeserializeRepr<'de, Parent> for Struct<TupleVariant, Fields>
where
    Parent: DataType,
    Fields: DeserializeTupleFields<'de>,
{
    fn deserialize_repr<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        deserializer
            .deserialize_tuple_struct(
                Parent::TYPE_NAME,
                Fields::LEN,
                TupleFieldsVisitor::<Fields>::new(Parent::TYPE_NAME),
            )
            .map(Struct::new)
    }
}

impl<Parent, Names, Fields> SerializeRepr<Parent> for Struct<StructVariant<Names>, Fields>
where
    Parent: DataType,
    Names: HasNames,
    Fields: SerializeStructFields,
{
    fn serialize_repr<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let serializer = serializer.serialize_struct(Parent::TYPE_NAME, Fields::LEN)?;
        self.fields.serialize_struct_fields(serializer)
    }
}

impl<'de, Parent, Names, Fields> DeserializeRepr<'de, Parent>
    for Struct<StructVariant<Names>, Fields>
where
    Parent: DataType,
    Names: HasNames,
    Fields: DeserializeStructFields<'de>,
{
    fn deserialize_repr<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        deserializer
            .deserialize_struct(
                Parent::TYPE_NAME,
                Names::NAMES,
                StructFieldsVisitor::<Fields>::new(Parent::TYPE_NAME),
            )
            .map(Struct::new)
    }
}

impl<Parent: DataType> SerializeRepr<Parent> for Struct<UnitVariant, Nil> {
    fn serialize_repr<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_unit_struct(Parent::TYPE_NAME)
    }
}

impl<'de, Parent: DataType> DeserializeRepr<'de, Parent> for Struct<UnitVariant, Nil> {
    fn deserialize_repr<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        struct UnitStructVisitor<Parent: DataType>(PhantomData<Parent>);

        impl<'de, Parent: DataType> Visitor<'de> for UnitStructVisitor<Parent> {
            type Value = Struct<UnitVariant, Nil>;

            fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
                formatter.write_str(Parent::TYPE_NAME)
            }

            fn visit_unit<E: serde::de::Error>(self) -> Result<Self::Value, E> {
                Ok(Struct::new(Nil))
            }
        }

        deserializer
            .deserialize_unit_struct(Parent::TYPE_NAME, UnitStructVisitor::<Parent>(PhantomData))
    }
}
