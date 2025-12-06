mod fields;
mod repr;
mod variants;

use serde::Serialize;
use serde::Serializer;

use crate::core::datatypes::DataType;
use crate::serde::repr::SerializeRepr;

pub struct SerializeViaSeu<'a, T: DataType + 'a>(pub T::ReprRef<'a>);

impl<'a, T: DataType + 'a> SerializeViaSeu<'a, T> {
    pub fn new(repr: &'a T) -> Self {
        Self(repr.as_repr())
    }
}

impl<'a, T> Serialize for SerializeViaSeu<'a, T>
where
    T: DataType,
    <T as DataType>::ReprRef<'a>: SerializeRepr<T>,
{
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        self.0.serialize_repr(serializer)
    }
}

#[cfg(test)]
mod tests;
