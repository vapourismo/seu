#![cfg_attr(not(feature = "std"), no_std)]

extern crate alloc;

mod fields;
mod repr;
mod variants;

use serde::Deserialize;
use serde::Deserializer;
use serde::Serialize;
use serde::Serializer;
use seu_core::datatypes::DataType;

use crate::repr::DeserializeRepr;
use crate::repr::SerializeRepr;

pub struct SerializeViaSeu<'a, T: DataType + 'a>(pub T::ReprRef<'a>);

impl<'a, T: DataType + 'a> SerializeViaSeu<'a, T> {
    pub fn new(data: &'a T) -> Self {
        Self(data.as_repr())
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

pub struct DeserializeViaSeu<T: DataType>(pub T::Repr);

impl<T: DataType> DeserializeViaSeu<T> {
    pub fn into_data(self) -> T {
        T::from_repr(self.0)
    }
}

impl<'de, T> Deserialize<'de> for DeserializeViaSeu<T>
where
    T: DataType,
    <T as DataType>::Repr: DeserializeRepr<'de, T>,
{
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        T::Repr::deserialize_repr(deserializer).map(Self)
    }
}

#[cfg(test)]
mod tests;
