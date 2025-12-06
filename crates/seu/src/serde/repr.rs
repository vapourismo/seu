use core::cell::RefCell;

use serde::Serializer;
use seu_core::names::HasNames;

use crate::core::datatypes::DataType;
use crate::core::enums::Enum;
use crate::core::fields::FieldsProduct;
use crate::core::structs::Struct;
use crate::core::variants::StructVariant;
use crate::core::variants::TupleVariant;
use crate::core::variants::UnitVariant;
use crate::serde::fields::SerializeStructFields;
use crate::serde::fields::SerializeTupleStructFields;
use crate::serde::variants::SerializeVariants;

pub trait SerializeRepr<Parent: DataType> {
    fn serialize_repr<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error>;
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

impl<Parent, Fields> SerializeRepr<Parent> for Struct<UnitVariant, Fields>
where
    Parent: DataType,
    Fields: FieldsProduct,
{
    fn serialize_repr<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_unit_struct(Parent::TYPE_NAME)
    }
}
