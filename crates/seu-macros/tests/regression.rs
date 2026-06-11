use seu::DataType;
use seu::core::Product;
use seu::core::enums::Enum;
use seu::core::fields::Field;
use seu::core::names::Named;
use seu::core::names::Unnamed;
use seu::core::structs::Struct;
use seu::core::variants::StructVariant;
use seu::core::variants::TupleVariant;
use seu::core::variants::UnitVariant;
use seu::core::variants::Variant;
use seu::macros::Num;

/// Helper trait that can be used to express a type equality constraint
#[diagnostic::on_unimplemented(message = "Type-level assertion failed: {Self} == {T}")]
pub trait Same<T: Same<Self> + ?Sized> {}

impl<T> Same<T> for T {}

/// Helper macro that asserts type equality
macro_rules! assert_ty {
    ($lhs:ty, $rhs:ty) => {
        const _: () = {
            const fn assert_ty<A, B>()
            where
                A: Same<B>,
                B: Same<A>,
            {
            }

            assert_ty::<$lhs, $rhs>()
        };
    };
}

/// Helper macro that asserts the representation type of a data type
macro_rules! assert_repr {
    ($ty:ty, $repr:ty) => {
        assert_ty!(<$ty as seu::core::datatypes::DataType>::Repr, $repr);
    };
}

/// Helper macro that asserts the shared representation type of a data type
macro_rules! assert_repr_ref {
    ($ty:ty, $repr:ty) => {
        assert_ty!(
            <$ty as seu::core::datatypes::DataType>::ReprRef<'static>,
            $repr
        );
    };
}

/// Helper macro that asserts the mutable representation type of a data type
macro_rules! assert_repr_mut {
    ($ty:ty, $repr:ty) => {
        assert_ty!(
            <$ty as seu::core::datatypes::DataType>::ReprMut<'static>,
            $repr
        );
    };
}

#[test]
fn main() {
    #[derive(DataType)]
    struct UnitStruct;

    assert_repr!(UnitStruct, Struct<_, UnitVariant, Product![]>);
    assert_repr_ref!(UnitStruct, Struct<_, UnitVariant, Product![]>);
    assert_repr_mut!(UnitStruct, Struct<_, UnitVariant, Product![]>);

    #[derive(DataType)]
    struct EmptyTupleStruct();

    assert_repr!(EmptyTupleStruct, Struct<_, TupleVariant, Product![]>);
    assert_repr_ref!(EmptyTupleStruct, Struct<_, TupleVariant, Product![]>);
    assert_repr_mut!(EmptyTupleStruct, Struct<_, TupleVariant, Product![]>);

    #[derive(DataType)]
    struct TupleStruct(u8, bool);

    assert_repr!(
        TupleStruct,
        Struct<_, TupleVariant, Product![Field<Unnamed, u8>, Field<Unnamed, bool>]>
    );
    assert_repr_ref!(
        TupleStruct,
        Struct<
            _,
            TupleVariant,
            Product![Field<Unnamed, &'static u8>, Field<Unnamed, &'static bool>]
        >
    );
    assert_repr_mut!(
        TupleStruct,
        Struct<
            _,
            TupleVariant,
            Product![
                Field<Unnamed, &'static mut u8>,
                Field<Unnamed, &'static mut bool>
            ]
        >
    );

    #[derive(DataType)]
    struct EmptyNamedStruct {}

    assert_repr!(EmptyNamedStruct, Struct<_, StructVariant<_>, Product![]>);
    assert_repr_ref!(EmptyNamedStruct, Struct<_, StructVariant<_>, Product![]>);
    assert_repr_mut!(EmptyNamedStruct, Struct<_, StructVariant<_>, Product![]>);

    #[derive(DataType)]
    struct NamedStruct {
        first: u8,
        second: bool,
    }

    assert_repr!(
        NamedStruct,
        Struct<_, StructVariant<_>, Product![Field<_, u8>, Field<_, bool>]>
    );
    assert_repr_ref!(
        NamedStruct,
        Struct<
            _,
            StructVariant<_>,
            Product![Field<Named<_>, &'static u8>, Field<Named<_>, &'static bool>],
        >
    );
    assert_repr_mut!(
        NamedStruct,
        Struct<
            _,
            StructVariant<_>,
            Product![
                Field<Named<_>, &'static mut u8>,
                Field<Named<_>, &'static mut bool>
            ],
        >
    );

    #[derive(DataType)]
    enum EmptyEnum {}

    assert_repr!(EmptyEnum, Enum<_, _, Product![]>);
    assert_repr_ref!(EmptyEnum, Enum<_, _, Product![]>);
    assert_repr_mut!(EmptyEnum, Enum<_, _, Product![]>);

    #[derive(DataType)]
    enum UnitEnum {
        Unit,
    }

    assert_repr!(
        UnitEnum,
        Enum<_, _, Product![Variant<_, Num!(0), UnitVariant, Product![]>]>
    );
    assert_repr_ref!(
        UnitEnum,
        Enum<_, _, Product![Variant<_, Num!(0), UnitVariant, Product![]>]>
    );
    assert_repr_mut!(
        UnitEnum,
        Enum<_, _, Product![Variant<_, Num!(0), UnitVariant, Product![]>]>
    );

    #[derive(DataType)]
    enum TupleEnum {
        EmptyTuple(),
        Tuple(u8, bool),
    }

    assert_repr!(
        TupleEnum,
        Enum<
            _,
            _,
            Product![
                Variant<_, Num!(0), TupleVariant, Product![]>,
                Variant<
                    _,
                    Num!(1),
                    TupleVariant,
                    Product![Field<Unnamed, u8>, Field<Unnamed, bool>]
                >
            ],
        >
    );
    assert_repr_ref!(
        TupleEnum,
        Enum<
            _,
            _,
            Product![
                Variant<_, Num!(0), TupleVariant, Product![]>,
                Variant<
                    _,
                    Num!(1),
                    TupleVariant,
                    Product![Field<Unnamed, &'static u8>, Field<Unnamed, &'static bool>]
                >
            ],
        >
    );
    assert_repr_mut!(
        TupleEnum,
        Enum<
            _,
            _,
            Product![
                Variant<_, Num!(0), TupleVariant, Product![]>,
                Variant<
                    _,
                    Num!(1),
                    TupleVariant,
                    Product![
                        Field<Unnamed, &'static mut u8>,
                        Field<Unnamed, &'static mut bool>
                    ]
                >
            ],
        >
    );

    #[derive(DataType)]
    enum StructEnum {
        EmptyStruct {},
        Struct { first: u8, second: bool },
    }

    assert_repr!(
        StructEnum,
        Enum<
            _,
            _,
            Product![
                Variant<_, Num!(0), StructVariant<_>, Product![]>,
                Variant<
                    _,
                    Num!(1),
                    StructVariant<_>,
                    Product![Field<Named<_>, u8>, Field<Named<_>, bool>]
                >
            ],
        >
    );
    assert_repr_ref!(
        StructEnum,
        Enum<
            _,
            _,
            Product![
                Variant<_, Num!(0), StructVariant<_>, Product![]>,
                Variant<
                    _,
                    Num!(1),
                    StructVariant<_>,
                    Product![Field<Named<_>, &'static u8>, Field<Named<_>, &'static bool>]
                >
            ],
        >
    );
    assert_repr_mut!(
        StructEnum,
        Enum<
            _,
            _,
            Product![
                Variant<_, Num!(0), StructVariant<_>, Product![]>,
                Variant<
                    _,
                    Num!(1),
                    StructVariant<_>,
                    Product![
                        Field<Named<_>, &'static mut u8>,
                        Field<Named<_>, &'static mut bool>
                    ]
                >
            ],
        >
    );

    #[derive(DataType)]
    enum EnumShapes {
        Unit,
        EmptyTuple(),
        Tuple(u8, bool),
        EmptyStruct {},
        Struct { first: u8, second: bool },
    }

    assert_repr!(
        EnumShapes,
        Enum<
            _,
            _,
            Product![
                Variant<_, Num!(0), UnitVariant, Product![]>,
                Variant<_, Num!(1), TupleVariant, Product![]>,
                Variant<
                    _,
                    Num!(2),
                    TupleVariant,
                    Product![Field<Unnamed, u8>, Field<Unnamed, bool>]
                >,
                Variant<_, Num!(3), StructVariant<_>, Product![]>,
                Variant<
                    _,
                    Num!(4),
                    StructVariant<_>,
                    Product![Field<Named<_>, u8>, Field<Named<_>, bool>]
                >
            ],
        >
    );
    assert_repr_ref!(
        EnumShapes,
        Enum<
            _,
            _,
            Product![
                Variant<_, Num!(0), UnitVariant, Product![]>,
                Variant<_, Num!(1), TupleVariant, Product![]>,
                Variant<
                    _,
                    Num!(2),
                    TupleVariant,
                    Product![Field<Unnamed, &'static u8>, Field<Unnamed, &'static bool>]
                >,
                Variant<_, Num!(3), StructVariant<_>, Product![]>,
                Variant<
                    _,
                    Num!(4),
                    StructVariant<_>,
                    Product![Field<Named<_>, &'static u8>, Field<Named<_>, &'static bool>]
                >
            ],
        >
    );
    assert_repr_mut!(
        EnumShapes,
        Enum<
            _,
            _,
            Product![
                Variant<_, Num!(0), UnitVariant, Product![]>,
                Variant<_, Num!(1), TupleVariant, Product![]>,
                Variant<
                    _,
                    Num!(2),
                    TupleVariant,
                    Product![
                        Field<Unnamed, &'static mut u8>,
                        Field<Unnamed, &'static mut bool>
                    ]
                >,
                Variant<_, Num!(3), StructVariant<_>, Product![]>,
                Variant<
                    _,
                    Num!(4),
                    StructVariant<_>,
                    Product![
                        Field<Named<_>, &'static mut u8>,
                        Field<Named<_>, &'static mut bool>
                    ]
                >
            ],
        >
    );
}
