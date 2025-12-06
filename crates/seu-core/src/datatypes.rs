/// Implementing types are data types (e.g. structs, enums)
pub trait DataType {
    /// Name of the data type's constructor
    const TYPE_NAME: &'static str;

    /// Owned generic representation
    ///
    /// In this representation, all fields are owned.
    type Repr;

    /// Immutably borrowing generic representation
    ///
    /// In this representation, all fields are immutably borrowed.
    type ReprRef<'a>
    where
        Self: 'a;

    /// Mutably borrowing generic representation
    ///
    /// In this representation, all fields are mutably borrowed.
    type ReprMut<'a>
    where
        Self: 'a;

    /// Converts the owned generic representation.
    fn into_repr(self) -> Self::Repr;

    /// Immutably borrows this value as its shared generic representation.
    fn as_repr(&self) -> Self::ReprRef<'_>;

    /// Mutably borrows this value as its mutable generic representation.
    fn as_repr_mut(&mut self) -> Self::ReprMut<'_>;

    /// Construct a value from its owned generic representation.
    fn from_repr(repr: Self::Repr) -> Self;
}
