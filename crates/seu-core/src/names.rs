/// Types that represent a name (e.g. of a field, variant)
pub trait HasName: HasOptionalName {
    const NAME: &'static str;
}

/// Types that represent an optional name
pub trait HasOptionalName {
    const OPT_NAME: Option<&'static str>;
}

impl<T: HasName> HasOptionalName for T {
    const OPT_NAME: Option<&'static str> = Some(T::NAME);
}

/// Marker for named things (e.g. fields of a struct)
pub struct Named<Name: HasName>(pub Name);

impl<Name: HasName> HasName for Named<Name> {
    const NAME: &'static str = Name::NAME;
}

/// Marker for unnamed things (e.g. fields of a tuple struct or variant)
pub struct Unnamed;

impl HasOptionalName for Unnamed {
    const OPT_NAME: Option<&'static str> = None;
}

/// Something that contains a sequence of names
pub trait HasNames {
    const NAMES: &'static [&'static str];
}
