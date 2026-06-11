use core::marker::PhantomData;

use crate::fields::FieldsProduct;
use crate::names::HasName;
use crate::variants::VariantFlavour;

/// Generic struct representation
#[repr(transparent)]
pub struct Struct<Name: HasName, Flavour: VariantFlavour, Fields: FieldsProduct> {
    /// Generic representation of fields
    pub fields: Fields,
    name: PhantomData<Name>,
    flavour: PhantomData<Flavour>,
}

impl<Name: HasName, Flavour: VariantFlavour, Fields: FieldsProduct> Struct<Name, Flavour, Fields> {
    #[inline]
    pub fn new(fields: Fields) -> Self {
        Self {
            fields,
            name: PhantomData,
            flavour: PhantomData,
        }
    }
}
