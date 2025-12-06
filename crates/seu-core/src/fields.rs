use core::marker::PhantomData;

use crate::names::HasName;
use crate::names::HasOptionalName;
use crate::product::Cons;
use crate::product::Nil;
use crate::product::Product;

/// Generic field representation
#[repr(transparent)]
pub struct Field<Name: HasOptionalName, Type> {
    /// Runtime value of the field
    pub value: Type,
    name: PhantomData<Name>,
}

impl<Name: HasOptionalName, Type> Field<Name, Type> {
    /// Construct a generic field representation for `Name` using its runtime value.
    #[inline]
    pub fn new(value: Type) -> Self {
        Self {
            value,
            name: PhantomData,
        }
    }
}

impl<Name: HasName, Type> HasName for Field<Name, Type> {
    const NAME: &'static str = Name::NAME;
}

/// Product which consist of field representations
pub trait FieldsProduct: Product {}

impl FieldsProduct for Nil {}

impl<Name, Type, Tail> FieldsProduct for Cons<Field<Name, Type>, Tail>
where
    Name: HasOptionalName,
    Tail: FieldsProduct,
{
}
