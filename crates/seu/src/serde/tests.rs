use serde::Deserialize;
use serde::Serialize;
use serde_json::json;

use crate::DataType;
use crate::serde::DeserializeViaSeu;
use crate::serde::SerializeViaSeu;

fn deserialize_like<T>(_: &T, value: serde_json::Value) -> T
where
    T: crate::core::datatypes::DataType,
    DeserializeViaSeu<T>: serde::de::DeserializeOwned,
{
    serde_json::from_value::<DeserializeViaSeu<T>>(value)
        .unwrap()
        .into_data()
}

fn deserialize<T>(value: serde_json::Value) -> serde_json::Result<T>
where
    T: crate::core::datatypes::DataType,
    DeserializeViaSeu<T>: serde::de::DeserializeOwned,
{
    serde_json::from_value::<DeserializeViaSeu<T>>(value).map(DeserializeViaSeu::into_data)
}

macro_rules! assert_json {
    ($value:expr) => {{
        let value = $value;

        let got = serde_json::to_value(SerializeViaSeu::new(&value)).unwrap();
        let expected = serde_json::to_value(&value).unwrap();
        assert_eq!(got, expected);

        let got = deserialize_like(&value, expected);
        assert_eq!(got, value);
    }};
}

#[derive(Debug, PartialEq, DataType, Serialize, Deserialize)]
#[datatype(crate = crate)]
struct UnitStruct;

#[derive(Debug, PartialEq, DataType, Serialize, Deserialize)]
#[datatype(crate = crate)]
struct EmptyTupleStruct();

#[derive(Debug, PartialEq, DataType, Serialize, Deserialize)]
#[datatype(crate = crate)]
struct TupleStruct(u8, bool, String);

#[derive(Debug, PartialEq, DataType, Serialize, Deserialize)]
#[datatype(crate = crate)]
struct EmptyNamedStruct {}

#[derive(Debug, PartialEq, DataType, Serialize, Deserialize)]
#[datatype(crate = crate)]
struct NamedStruct {
    first: u8,
    second: bool,
    label: String,
}

#[derive(Debug, PartialEq, DataType, Serialize, Deserialize)]
#[datatype(crate = crate)]
struct GenericStruct<T> {
    value: T,
    items: Vec<T>,
}

#[derive(Debug, PartialEq, DataType, Serialize, Deserialize)]
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

#[test]
fn deserializes_named_structs_from_positional_fields() {
    let got = deserialize::<NamedStruct>(json!([1, true, "named"])).unwrap();

    assert_eq!(
        got,
        NamedStruct {
            first: 1,
            second: true,
            label: "named".to_owned(),
        }
    );
}

#[test]
fn deserializes_empty_shapes_from_unit_or_empty_fields() {
    assert_eq!(deserialize::<UnitStruct>(json!(null)).unwrap(), UnitStruct);
    assert_eq!(
        deserialize::<EmptyTupleStruct>(json!([])).unwrap(),
        EmptyTupleStruct()
    );
    assert_eq!(
        deserialize::<EmptyNamedStruct>(json!({})).unwrap(),
        EmptyNamedStruct {}
    );
}

#[test]
fn rejects_unit_for_non_empty_tuple_structs() {
    let err = deserialize::<TupleStruct>(json!(null)).unwrap_err();

    assert!(err.to_string().contains("invalid type: null"));
}

#[test]
fn deserializes_named_structs_ignoring_unknown_fields() {
    let got = deserialize::<NamedStruct>(json!({
        "ignored": "value",
        "first": 1,
        "second": true,
        "label": "named",
    }))
    .unwrap();

    assert_eq!(
        got,
        NamedStruct {
            first: 1,
            second: true,
            label: "named".to_owned(),
        }
    );
}

#[test]
fn rejects_duplicate_named_struct_fields() {
    let err = serde_json::from_str::<DeserializeViaSeu<NamedStruct>>(
        r#"{"first":1,"first":2,"second":true,"label":"named"}"#,
    )
    .err()
    .unwrap();

    assert!(err.to_string().contains("duplicate field `first`"));
}

#[test]
fn rejects_missing_named_struct_fields() {
    let err = deserialize::<NamedStruct>(json!({
        "first": 1,
        "second": true,
    }))
    .unwrap_err();

    assert!(err.to_string().contains("missing field `label`"));
}

#[test]
fn rejects_tuple_structs_with_wrong_length() {
    let too_short = deserialize::<TupleStruct>(json!([1, true])).unwrap_err();
    assert!(too_short.to_string().contains("invalid length 2"));

    let too_long = deserialize::<TupleStruct>(json!([1, true, "tuple", "extra"])).unwrap_err();
    assert!(too_long.to_string().contains("invalid length 4"));
}

#[test]
fn rejects_unknown_enum_variants() {
    let err = deserialize::<EnumShapes<u8>>(json!({
        "Unknown": null,
    }))
    .unwrap_err();

    assert!(err.to_string().contains("unknown variant"));
}

#[test]
fn rejects_duplicate_struct_variant_fields() {
    let err = serde_json::from_str::<DeserializeViaSeu<EnumShapes<u8>>>(
        r#"{"Struct":{"first":1,"first":2,"second":true,"label":"struct"}}"#,
    )
    .err()
    .unwrap();

    assert!(err.to_string().contains("duplicate field `first`"));
}

#[test]
fn rejects_missing_struct_variant_fields() {
    let err = deserialize::<EnumShapes<u8>>(json!({
        "Struct": {
            "first": 1,
            "second": true,
        },
    }))
    .unwrap_err();

    assert!(err.to_string().contains("missing field `label`"));
}
