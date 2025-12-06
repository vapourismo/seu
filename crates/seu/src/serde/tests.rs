use serde::Serialize;

use crate::DataType;
use crate::serde::SerializeViaSeu;

macro_rules! assert_json {
    ($value:expr $(,)?) => {{
        let value = $value;
        let got = serde_json::to_value(SerializeViaSeu::new(&value)).unwrap();
        let expected = serde_json::to_value(&value).unwrap();
        assert_eq!(got, expected);
    }};
}

#[derive(DataType, Serialize)]
#[datatype(crate = crate)]
struct UnitStruct;

#[derive(DataType, Serialize)]
#[datatype(crate = crate)]
struct EmptyTupleStruct();

#[derive(DataType, Serialize)]
#[datatype(crate = crate)]
struct TupleStruct(u8, bool, String);

#[derive(DataType, Serialize)]
#[datatype(crate = crate)]
struct EmptyNamedStruct {}

#[derive(DataType, Serialize)]
#[datatype(crate = crate)]
struct NamedStruct {
    first: u8,
    second: bool,
    label: String,
}

#[derive(DataType, Serialize)]
#[datatype(crate = crate)]
struct GenericStruct<T> {
    value: T,
    items: Vec<T>,
}

#[derive(DataType, Serialize)]
#[datatype(crate = crate)]
enum EnumShapes<T> {
    Unit,
    EmptyTuple(),
    Tuple(u8, bool, String),
    EmptyStruct {},
    Struct {
        first: u8,
        second: bool,
        label: String,
    },
    Generic {
        value: T,
        items: Vec<T>,
    },
}

#[test]
fn serializes_unit_struct() {
    assert_json!(UnitStruct);
}

#[test]
fn serializes_tuple_structs() {
    assert_json!(EmptyTupleStruct());
    assert_json!(TupleStruct(1, true, "tuple".to_owned()));
}

#[test]
fn serializes_named_structs() {
    assert_json!(EmptyNamedStruct {});
    assert_json!(NamedStruct {
        first: 1,
        second: true,
        label: "named".to_owned(),
    });
}

#[test]
fn serializes_generic_struct_fields() {
    assert_json!(GenericStruct {
        value: 1,
        items: vec![2, 3],
    });
}

#[test]
fn serializes_unit_enum_variant() {
    assert_json!(EnumShapes::<u8>::Unit);
}

#[test]
fn serializes_empty_tuple_enum_variant() {
    assert_json!(EnumShapes::<u8>::EmptyTuple());
}

#[test]
fn serializes_tuple_enum_variant() {
    assert_json!(EnumShapes::<u8>::Tuple(1, true, "tuple".to_owned()));
}

#[test]
fn serializes_empty_struct_enum_variant() {
    assert_json!(EnumShapes::<u8>::EmptyStruct {});
}

#[test]
fn serializes_struct_enum_variant() {
    assert_json!(EnumShapes::<u8>::Struct {
        first: 1,
        second: true,
        label: "struct".to_owned(),
    });
}

#[test]
fn serializes_generic_struct_enum_variant() {
    assert_json!(EnumShapes::Generic {
        value: 1,
        items: vec![2, 3],
    });
}
