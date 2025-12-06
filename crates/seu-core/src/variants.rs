use core::marker::PhantomData;

use crate::fields::FieldsProduct;
use crate::names::HasName;
use crate::names::HasNames;
use crate::nums::Num;
use crate::product::Cons;
use crate::product::Nil;
use crate::sum::SumVariants;

/// [`VariantFlavour`] for tuple structs and tuple variants
pub struct TupleVariant;

/// [`VariantFlavour`] for structs and struct variants
pub struct StructVariant<Names: HasNames>(PhantomData<Names>);

/// [`VariantFlavour`] for unit structs and unit variants
pub struct UnitVariant;

/// Marker trait for variant flavours (tuple, struct, unit)
pub trait VariantFlavour {}

impl VariantFlavour for TupleVariant {}

impl<Names: HasNames> VariantFlavour for StructVariant<Names> {}

impl VariantFlavour for UnitVariant {}

/// Generic variant representation
pub struct Variant<Name, Idx, Flavour, Fields>
where
    Name: HasName,
    Idx: Num,
    Flavour: VariantFlavour,
    Fields: FieldsProduct,
{
    pub fields: Fields,
    idx: PhantomData<Idx>,
    flavour: PhantomData<Flavour>,
    name: PhantomData<Name>,
}

impl<Name, Idx, Flavour, Fields> Variant<Name, Idx, Flavour, Fields>
where
    Name: HasName,
    Idx: Num,
    Flavour: VariantFlavour,
    Fields: FieldsProduct,
{
    /// Construct a generic variant representation.
    #[inline]
    pub fn new(fields: Fields) -> Self {
        Self {
            fields,
            idx: PhantomData,
            flavour: PhantomData,
            name: PhantomData,
        }
    }
}

/// Products that consist of variant information types (e.g. [`Variant`])
pub trait VariantsProduct: SumVariants {}

impl VariantsProduct for Nil {}

impl<Name, Idx, Flavour, Fields, Tail> VariantsProduct
    for Cons<Variant<Name, Idx, Flavour, Fields>, Tail>
where
    Name: HasName,
    Idx: Num,
    Flavour: VariantFlavour,
    Fields: FieldsProduct,
    Tail: VariantsProduct,
{
}
