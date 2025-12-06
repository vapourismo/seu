use core::marker::PhantomData;

use crate::names::HasNames;
use crate::sum::Sum;
use crate::variants::VariantsProduct;

/// Generic enum representation
#[repr(transparent)]
pub struct Enum<VariantNames: HasNames, Variants: VariantsProduct> {
    /// Active variant of the enum
    pub variant: Sum<Variants>,
    _variant_names: PhantomData<VariantNames>,
}

impl<VariantNames: HasNames, Variants: VariantsProduct> Enum<VariantNames, Variants> {
    /// Create a generic enum representation from a variant.
    #[inline]
    pub fn new(variant: Sum<Variants>) -> Self {
        Self {
            variant,
            _variant_names: PhantomData,
        }
    }
}
