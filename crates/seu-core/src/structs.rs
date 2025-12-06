use core::marker::PhantomData;

use crate::fields::FieldsProduct;
use crate::variants::VariantFlavour;

/// Generic struct representation
#[repr(transparent)]
pub struct Struct<Flavour: VariantFlavour, Fields: FieldsProduct> {
    /// Generic representation of fields
    pub fields: Fields,
    flavour: PhantomData<Flavour>,
}

impl<Flavour: VariantFlavour, Fields: FieldsProduct> Struct<Flavour, Fields> {
    #[inline]
    pub fn new(fields: Fields) -> Self {
        Self {
            fields,
            flavour: PhantomData,
        }
    }
}
