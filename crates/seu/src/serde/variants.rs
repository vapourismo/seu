use core::cell::RefCell;

use serde::Serializer;
use seu_core::names::HasNames;

use crate::core::datatypes::DataType;
use crate::core::names::HasName;
use crate::core::nums::Num;
use crate::core::product::Cons;
use crate::core::product::Nil;
use crate::core::sum::ReducerRef;
use crate::core::variants::StructVariant;
use crate::core::variants::TupleVariant;
use crate::core::variants::UnitVariant;
use crate::core::variants::Variant;
use crate::core::variants::VariantsProduct;
use crate::serde::fields::SerializeStructVariantFields;
use crate::serde::fields::SerializeTupleVariantFields;

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
