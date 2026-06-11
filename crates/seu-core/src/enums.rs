use core::marker::PhantomData;

use crate::names::HasName;
use crate::names::HasNames;
use crate::sum::Sum;
use crate::variants::VariantsProduct;

/// Generic enum representation
#[repr(transparent)]
pub struct Enum<Name: HasName, VariantNames: HasNames, Variants: VariantsProduct> {
    /// Active variant of the enum
    pub variant: Sum<Variants>,
    _name: PhantomData<Name>,
    _variant_names: PhantomData<VariantNames>,
}

impl<Name, VariantNames, Variants> Enum<Name, VariantNames, Variants>
where
    Name: HasName,
    VariantNames: HasNames,
    Variants: VariantsProduct,
{
    /// Create a generic enum representation from a variant.
    #[inline]
    pub fn new(variant: Sum<Variants>) -> Self {
        Self {
            variant,
            _name: PhantomData,
            _variant_names: PhantomData,
        }
    }
}
