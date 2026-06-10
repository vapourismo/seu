use alloc::borrow::ToOwned;
use alloc::fmt;
use alloc::string::String;
use alloc::vec::Vec;
use core::cell::RefCell;

use serde::Deserialize;
use serde::Serializer;
use serde::de::VariantAccess;
use serde::de::Visitor;
use seu_core::datatypes::DataType;
use seu_core::fields::FieldsProduct;
use seu_core::names::HasName;
use seu_core::names::HasNames;
use seu_core::nums::Index;
use seu_core::nums::Num;
use seu_core::product::Cons;
use seu_core::product::Nil;
use seu_core::sum::ReducerRef;
use seu_core::sum::Sum;
use seu_core::variants::StructVariant;
use seu_core::variants::TupleVariant;
use seu_core::variants::UnitVariant;
use seu_core::variants::Variant;
use seu_core::variants::VariantFlavour;
use seu_core::variants::VariantsProduct;

use crate::fields::DeserializeStructFields;
use crate::fields::DeserializeTupleFields;
use crate::fields::SerializeStructVariantFields;
use crate::fields::SerializeTupleVariantFields;
use crate::fields::StructFieldsVisitor;
use crate::fields::TupleFieldsVisitor;

pub trait DeserializeVariants<'de, Parent: DataType, TopVariants: VariantsProduct>:
    VariantsProduct + Sized
{
    fn deserialize_variant<A: VariantAccess<'de>>(
        var: VariantIdx,
        access: A,
    ) -> Result<Sum<TopVariants>, A::Error>;
}

impl<'de, Parent: DataType, TopVariants: VariantsProduct>
    DeserializeVariants<'de, Parent, TopVariants> for Nil
{
    fn deserialize_variant<A: VariantAccess<'de>>(
        var: VariantIdx,
        _access: A,
    ) -> Result<Sum<TopVariants>, A::Error> {
        Err(serde::de::Error::custom(format_args!(
            "unknown variant {var:?}"
        )))
    }
}

impl<'de, Name, Idx, Flavour, Fields, Tail, Parent, TopVariants>
    DeserializeVariants<'de, Parent, TopVariants>
    for Cons<Variant<Name, Idx, Flavour, Fields>, Tail>
where
    Name: HasName,
    Idx: Index<TopVariants, Selected = Variant<Name, Idx, Flavour, Fields>>,
    Flavour: VariantFlavour,
    Fields: FieldsProduct,
    Variant<Name, Idx, Flavour, Fields>: DeserializeVariant<'de>,
    Parent: DataType,
    Tail: DeserializeVariants<'de, Parent, TopVariants>,
    TopVariants: VariantsProduct,
{
    fn deserialize_variant<A: VariantAccess<'de>>(
        var: VariantIdx,
        access: A,
    ) -> Result<Sum<TopVariants>, A::Error> {
        match &var {
            VariantIdx::Index(idx) if *idx == Idx::NUM => {}
            VariantIdx::String(name) if name == Name::NAME => {}
            VariantIdx::Bytes(name) if name == Name::NAME.as_bytes() => {}
            _ => return Tail::deserialize_variant(var, access),
        }

        let variant = Variant::deserialize_variant(access)?;
        Ok(Sum::new(Idx::default(), variant))
    }
}

pub trait DeserializeVariant<'de>: Sized {
    fn deserialize_variant<A: VariantAccess<'de>>(access: A) -> Result<Self, A::Error>;
}

impl<'de, Name, Idx, Fields> DeserializeVariant<'de> for Variant<Name, Idx, TupleVariant, Fields>
where
    Name: HasName,
    Idx: Num,
    Fields: DeserializeTupleFields<'de>,
{
    fn deserialize_variant<A: VariantAccess<'de>>(access: A) -> Result<Self, A::Error> {
        access
            .tuple_variant(Fields::LEN, TupleFieldsVisitor::<Fields>::new(Name::NAME))
            .map(Variant::new)
    }
}

impl<'de, Name, Idx, Names, Fields> DeserializeVariant<'de>
    for Variant<Name, Idx, StructVariant<Names>, Fields>
where
    Name: HasName,
    Idx: Num,
    Names: HasNames,
    Fields: DeserializeStructFields<'de>,
{
    fn deserialize_variant<A: VariantAccess<'de>>(access: A) -> Result<Self, A::Error> {
        access
            .struct_variant(Names::NAMES, StructFieldsVisitor::<Fields>::new(Name::NAME))
            .map(Variant::new)
    }
}

impl<'de, Name, Idx> DeserializeVariant<'de> for Variant<Name, Idx, UnitVariant, Nil>
where
    Name: HasName,
    Idx: Num,
{
    fn deserialize_variant<A: VariantAccess<'de>>(access: A) -> Result<Self, A::Error> {
        access.unit_variant()?;
        Ok(Variant::new(Nil))
    }
}

pub trait SerializeVariants<Parent: DataType>: VariantsProduct + Sized {
    fn eager_handlers<'a, S: Serializer>(
        serializer: &RefCell<Option<S>>,
    ) -> impl ReducerRef<'a, Self, Result<S::Ok, S::Error>>
    where
        Self: 'a;
}

impl<Parent: DataType> SerializeVariants<Parent> for Nil {
    #[inline]
    fn eager_handlers<'a, S: Serializer>(
        _serializer: &RefCell<Option<S>>,
    ) -> impl ReducerRef<'a, Self, Result<S::Ok, S::Error>>
    where
        Self: 'a,
    {
        Nil
    }
}

impl<Variant, Parent, Tail> SerializeVariants<Parent> for Cons<Variant, Tail>
where
    Variant: SerializeVariant<Parent>,
    Parent: DataType,
    Tail: SerializeVariants<Parent>,
    Self: VariantsProduct,
{
    #[inline]
    fn eager_handlers<'a, S: Serializer>(
        serializer: &RefCell<Option<S>>,
    ) -> impl ReducerRef<'a, Self, Result<S::Ok, S::Error>>
    where
        Self: 'a,
    {
        Cons(
            |input: &'a Variant| {
                let serializer = serializer
                    .take()
                    .expect("No other variant should have taken the serializer");
                input.serialize_variant(serializer)
            },
            Tail::eager_handlers(serializer),
        )
    }
}

pub trait SerializeVariant<Parent: DataType> {
    fn serialize_variant<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error>;
}

impl<Name, Idx, Fields, Parent> SerializeVariant<Parent>
    for Variant<Name, Idx, TupleVariant, Fields>
where
    Name: HasName,
    Idx: Num,
    Fields: SerializeTupleVariantFields,
    Parent: DataType,
{
    fn serialize_variant<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let serializer = serializer.serialize_tuple_variant(
            Parent::TYPE_NAME,
            Idx::NUM as u32,
            Name::NAME,
            Fields::LEN,
        )?;
        self.fields.serialize_tuple_fields(serializer)
    }
}

impl<Name, Idx, Names, Fields, Parent> SerializeVariant<Parent>
    for Variant<Name, Idx, StructVariant<Names>, Fields>
where
    Name: HasName,
    Idx: Num,
    Names: HasNames,
    Fields: SerializeStructVariantFields,
    Parent: DataType,
{
    fn serialize_variant<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let serializer = serializer.serialize_struct_variant(
            Parent::TYPE_NAME,
            Idx::NUM as u32,
            Name::NAME,
            Fields::LEN,
        )?;
        self.fields.serialize_struct_fields(serializer)
    }
}

impl<Name, Idx, Parent> SerializeVariant<Parent> for Variant<Name, Idx, UnitVariant, Nil>
where
    Name: HasName,
    Idx: Num,
    Parent: DataType,
{
    fn serialize_variant<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_unit_variant(Parent::TYPE_NAME, Idx::NUM as u32, Name::NAME)
    }
}

#[derive(Debug, Clone)]
pub enum VariantIdx {
    Index(u64),
    String(String),
    Bytes(Vec<u8>),
}

impl<'de> Deserialize<'de> for VariantIdx {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        struct Visit;

        impl Visitor<'_> for Visit {
            type Value = VariantIdx;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("variant")
            }

            fn visit_u64<E>(self, v: u64) -> Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                Ok(VariantIdx::Index(v))
            }

            fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                Ok(VariantIdx::String(v.to_owned()))
            }

            fn visit_bytes<E>(self, v: &[u8]) -> Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                Ok(VariantIdx::Bytes(v.to_owned()))
            }
        }

        deserializer.deserialize_identifier(Visit)
    }
}
