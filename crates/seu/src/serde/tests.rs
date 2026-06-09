use serde::Deserialize;
use serde::Serialize;
use serde::de::DeserializeSeed;
use serde::de::EnumAccess;
use serde::de::IntoDeserializer;
use serde::de::MapAccess;
use serde::de::VariantAccess;
use serde::de::Visitor;
use serde::ser::Impossible;
use serde::ser::SerializeStructVariant;
use serde::ser::SerializeTupleVariant;
use serde_json::json;

use crate::DataType;
use crate::serde::DeserializeViaSeu;
use crate::serde::SerializeViaSeu;
use crate::serde::fields::FieldIdx;
use crate::serde::variants::VariantIdx;

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

#[derive(Debug)]
struct TestError(String);

impl std::fmt::Display for TestError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(&self.0)
    }
}

impl std::error::Error for TestError {}

impl serde::de::Error for TestError {
    fn custom<T: std::fmt::Display>(msg: T) -> Self {
        Self(msg.to_string())
    }
}

impl serde::ser::Error for TestError {
    fn custom<T: std::fmt::Display>(msg: T) -> Self {
        Self(msg.to_string())
    }
}

struct BytesKeyMapDeserializer {
    entries: std::vec::IntoIter<(&'static [u8], serde_json::Value)>,
    value: Option<serde_json::Value>,
}

impl BytesKeyMapDeserializer {
    fn new(entries: Vec<(&'static [u8], serde_json::Value)>) -> Self {
        Self {
            entries: entries.into_iter(),
            value: None,
        }
    }
}

impl<'de> serde::Deserializer<'de> for BytesKeyMapDeserializer {
    type Error = TestError;

    fn deserialize_any<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value, Self::Error> {
        visitor.visit_map(self)
    }

    fn deserialize_struct<V: Visitor<'de>>(
        self,
        _name: &'static str,
        _fields: &'static [&'static str],
        visitor: V,
    ) -> Result<V::Value, Self::Error> {
        visitor.visit_map(self)
    }

    serde::forward_to_deserialize_any! {
        bool i8 i16 i32 i64 u8 u16 u32 u64 f32 f64 char str string bytes
        byte_buf option unit unit_struct newtype_struct seq tuple tuple_struct map
        enum identifier ignored_any
    }
}

impl<'de> MapAccess<'de> for BytesKeyMapDeserializer {
    type Error = TestError;

    fn next_key_seed<K: DeserializeSeed<'de>>(
        &mut self,
        seed: K,
    ) -> Result<Option<K::Value>, Self::Error> {
        let Some((key, value)) = self.entries.next() else {
            return Ok(None);
        };

        self.value = Some(value);
        let key = serde::de::value::BorrowedBytesDeserializer::<TestError>::new(key);
        seed.deserialize(key).map(Some)
    }

    fn next_value_seed<V: DeserializeSeed<'de>>(
        &mut self,
        seed: V,
    ) -> Result<V::Value, Self::Error> {
        seed.deserialize(self.value.take().unwrap())
            .map_err(serde::de::Error::custom)
    }
}

struct IndexedEnumDeserializer {
    index: u64,
    value: serde_json::Value,
}

impl<'de> serde::Deserializer<'de> for IndexedEnumDeserializer {
    type Error = serde_json::Error;

    fn deserialize_any<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value, Self::Error> {
        visitor.visit_enum(self)
    }

    fn deserialize_enum<V: Visitor<'de>>(
        self,
        _name: &'static str,
        _variants: &'static [&'static str],
        visitor: V,
    ) -> Result<V::Value, Self::Error> {
        visitor.visit_enum(self)
    }

    serde::forward_to_deserialize_any! {
        bool i8 i16 i32 i64 u8 u16 u32 u64 f32 f64 char str string bytes
        byte_buf option unit unit_struct newtype_struct seq tuple tuple_struct map
        struct identifier ignored_any
    }
}

impl<'de> EnumAccess<'de> for IndexedEnumDeserializer {
    type Error = serde_json::Error;
    type Variant = JsonVariantAccess;

    fn variant_seed<V: DeserializeSeed<'de>>(
        self,
        seed: V,
    ) -> Result<(V::Value, Self::Variant), Self::Error> {
        let index = serde::de::value::U64Deserializer::<serde_json::Error>::new(self.index);
        let variant = seed.deserialize(index)?;
        Ok((variant, JsonVariantAccess(self.value)))
    }
}

struct JsonVariantAccess(serde_json::Value);

impl<'de> VariantAccess<'de> for JsonVariantAccess {
    type Error = serde_json::Error;

    fn unit_variant(self) -> Result<(), Self::Error> {
        Deserialize::deserialize(self.0)
    }

    fn newtype_variant_seed<T: DeserializeSeed<'de>>(
        self,
        seed: T,
    ) -> Result<T::Value, Self::Error> {
        seed.deserialize(self.0)
    }

    fn tuple_variant<V: Visitor<'de>>(
        self,
        _len: usize,
        visitor: V,
    ) -> Result<V::Value, Self::Error> {
        let serde_json::Value::Array(items) = self.0 else {
            return Err(serde::de::Error::custom("expected tuple variant array"));
        };

        visitor.visit_seq(
            serde::de::value::SeqDeserializer::<_, serde_json::Error>::new(items.into_iter()),
        )
    }

    fn struct_variant<V: Visitor<'de>>(
        self,
        _fields: &'static [&'static str],
        visitor: V,
    ) -> Result<V::Value, Self::Error> {
        let serde_json::Value::Object(fields) = self.0 else {
            return Err(serde::de::Error::custom("expected struct variant object"));
        };

        visitor.visit_map(
            serde::de::value::MapDeserializer::<_, serde_json::Error>::new(fields.into_iter()),
        )
    }
}

struct BytesEnumDeserializer {
    name: &'static [u8],
    value: serde_json::Value,
}

impl<'de> serde::Deserializer<'de> for BytesEnumDeserializer {
    type Error = serde_json::Error;

    fn deserialize_any<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value, Self::Error> {
        visitor.visit_enum(self)
    }

    fn deserialize_enum<V: Visitor<'de>>(
        self,
        _name: &'static str,
        _variants: &'static [&'static str],
        visitor: V,
    ) -> Result<V::Value, Self::Error> {
        visitor.visit_enum(self)
    }

    serde::forward_to_deserialize_any! {
        bool i8 i16 i32 i64 u8 u16 u32 u64 f32 f64 char str string bytes
        byte_buf option unit unit_struct newtype_struct seq tuple tuple_struct map
        struct identifier ignored_any
    }
}

impl<'de> EnumAccess<'de> for BytesEnumDeserializer {
    type Error = serde_json::Error;
    type Variant = JsonVariantAccess;

    fn variant_seed<V: DeserializeSeed<'de>>(
        self,
        seed: V,
    ) -> Result<(V::Value, Self::Variant), Self::Error> {
        let name = serde::de::value::BorrowedBytesDeserializer::<serde_json::Error>::new(self.name);
        let variant = seed.deserialize(name)?;
        Ok((variant, JsonVariantAccess(self.value)))
    }
}

struct ExpectingDeserializer;

impl<'de> serde::Deserializer<'de> for ExpectingDeserializer {
    type Error = TestError;

    fn deserialize_any<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value, Self::Error> {
        Err(serde::de::Error::invalid_type(
            serde::de::Unexpected::Bool(false),
            &visitor,
        ))
    }

    serde::forward_to_deserialize_any! {
        bool i8 i16 i32 i64 u8 u16 u32 u64 f32 f64 char str string bytes
        byte_buf option unit unit_struct newtype_struct seq tuple tuple_struct map
        struct enum identifier ignored_any
    }
}

struct UnitTupleStructDeserializer;

impl<'de> serde::Deserializer<'de> for UnitTupleStructDeserializer {
    type Error = TestError;

    fn deserialize_tuple_struct<V: Visitor<'de>>(
        self,
        _name: &'static str,
        _len: usize,
        visitor: V,
    ) -> Result<V::Value, Self::Error> {
        visitor.visit_unit()
    }

    fn deserialize_any<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value, Self::Error> {
        self.deserialize_tuple_struct("", 0, visitor)
    }

    serde::forward_to_deserialize_any! {
        bool i8 i16 i32 i64 u8 u16 u32 u64 f32 f64 char str string bytes
        byte_buf option unit unit_struct newtype_struct seq tuple map
        struct enum identifier ignored_any
    }
}

struct ExpectingIdentifierDeserializer;

impl<'de> serde::Deserializer<'de> for ExpectingIdentifierDeserializer {
    type Error = TestError;

    fn deserialize_identifier<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value, Self::Error> {
        Err(serde::de::Error::invalid_type(
            serde::de::Unexpected::Bool(false),
            &visitor,
        ))
    }

    fn deserialize_any<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value, Self::Error> {
        self.deserialize_identifier(visitor)
    }

    serde::forward_to_deserialize_any! {
        bool i8 i16 i32 i64 u8 u16 u32 u64 f32 f64 char str string bytes
        byte_buf option unit unit_struct newtype_struct seq tuple tuple_struct map
        struct enum ignored_any
    }
}

struct VariantIndexSerializer;

impl serde::Serializer for VariantIndexSerializer {
    type Ok = u32;
    type Error = TestError;
    type SerializeSeq = Impossible<u32, TestError>;
    type SerializeTuple = Impossible<u32, TestError>;
    type SerializeTupleStruct = Impossible<u32, TestError>;
    type SerializeTupleVariant = CaptureTupleVariant;
    type SerializeMap = Impossible<u32, TestError>;
    type SerializeStruct = Impossible<u32, TestError>;
    type SerializeStructVariant = CaptureStructVariant;

    fn serialize_unit_variant(
        self,
        _name: &'static str,
        variant_index: u32,
        _variant: &'static str,
    ) -> Result<Self::Ok, Self::Error> {
        Ok(variant_index)
    }

    fn serialize_tuple_variant(
        self,
        _name: &'static str,
        variant_index: u32,
        _variant: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeTupleVariant, Self::Error> {
        Ok(CaptureTupleVariant(variant_index))
    }

    fn serialize_struct_variant(
        self,
        _name: &'static str,
        variant_index: u32,
        _variant: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeStructVariant, Self::Error> {
        Ok(CaptureStructVariant(variant_index))
    }

    fn serialize_bool(self, _v: bool) -> Result<Self::Ok, Self::Error> {
        unreachable!()
    }

    fn serialize_i8(self, _v: i8) -> Result<Self::Ok, Self::Error> {
        unreachable!()
    }

    fn serialize_i16(self, _v: i16) -> Result<Self::Ok, Self::Error> {
        unreachable!()
    }

    fn serialize_i32(self, _v: i32) -> Result<Self::Ok, Self::Error> {
        unreachable!()
    }

    fn serialize_i64(self, _v: i64) -> Result<Self::Ok, Self::Error> {
        unreachable!()
    }

    fn serialize_u8(self, _v: u8) -> Result<Self::Ok, Self::Error> {
        unreachable!()
    }

    fn serialize_u16(self, _v: u16) -> Result<Self::Ok, Self::Error> {
        unreachable!()
    }

    fn serialize_u32(self, _v: u32) -> Result<Self::Ok, Self::Error> {
        unreachable!()
    }

    fn serialize_u64(self, _v: u64) -> Result<Self::Ok, Self::Error> {
        unreachable!()
    }

    fn serialize_f32(self, _v: f32) -> Result<Self::Ok, Self::Error> {
        unreachable!()
    }

    fn serialize_f64(self, _v: f64) -> Result<Self::Ok, Self::Error> {
        unreachable!()
    }

    fn serialize_char(self, _v: char) -> Result<Self::Ok, Self::Error> {
        unreachable!()
    }

    fn serialize_str(self, _v: &str) -> Result<Self::Ok, Self::Error> {
        unreachable!()
    }

    fn serialize_bytes(self, _v: &[u8]) -> Result<Self::Ok, Self::Error> {
        unreachable!()
    }

    fn serialize_none(self) -> Result<Self::Ok, Self::Error> {
        unreachable!()
    }

    fn serialize_some<T: ?Sized + Serialize>(self, _value: &T) -> Result<Self::Ok, Self::Error> {
        unreachable!()
    }

    fn serialize_unit(self) -> Result<Self::Ok, Self::Error> {
        unreachable!()
    }

    fn serialize_unit_struct(self, _name: &'static str) -> Result<Self::Ok, Self::Error> {
        unreachable!()
    }

    fn serialize_newtype_struct<T: ?Sized + Serialize>(
        self,
        _name: &'static str,
        _value: &T,
    ) -> Result<Self::Ok, Self::Error> {
        unreachable!()
    }

    fn serialize_newtype_variant<T: ?Sized + Serialize>(
        self,
        _name: &'static str,
        _variant_index: u32,
        _variant: &'static str,
        _value: &T,
    ) -> Result<Self::Ok, Self::Error> {
        unreachable!()
    }

    fn serialize_seq(self, _len: Option<usize>) -> Result<Self::SerializeSeq, Self::Error> {
        unreachable!()
    }

    fn serialize_tuple(self, _len: usize) -> Result<Self::SerializeTuple, Self::Error> {
        unreachable!()
    }

    fn serialize_tuple_struct(
        self,
        _name: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeTupleStruct, Self::Error> {
        unreachable!()
    }

    fn serialize_map(self, _len: Option<usize>) -> Result<Self::SerializeMap, Self::Error> {
        unreachable!()
    }

    fn serialize_struct(
        self,
        _name: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeStruct, Self::Error> {
        unreachable!()
    }
}

struct CaptureTupleVariant(u32);

impl SerializeTupleVariant for CaptureTupleVariant {
    type Ok = u32;
    type Error = TestError;

    fn serialize_field<T: ?Sized + Serialize>(&mut self, _value: &T) -> Result<(), Self::Error> {
        Ok(())
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        Ok(self.0 + 100)
    }
}

struct CaptureStructVariant(u32);

impl SerializeStructVariant for CaptureStructVariant {
    type Ok = u32;
    type Error = TestError;

    fn serialize_field<T: ?Sized + Serialize>(
        &mut self,
        _key: &'static str,
        _value: &T,
    ) -> Result<(), Self::Error> {
        Ok(())
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        Ok(self.0 + 200)
    }
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
fn deserializes_named_structs_from_numeric_field_indices() {
    let fields = [
        (0_u64.into_deserializer(), json!(1)),
        (1_u64.into_deserializer(), json!(true)),
        (2_u64.into_deserializer(), json!("named")),
    ];
    let deserializer = serde::de::value::MapDeserializer::new(fields.into_iter());

    let got = DeserializeViaSeu::<NamedStruct>::deserialize(deserializer)
        .unwrap()
        .into_data();

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
fn rejects_unit_tuple_struct_from_tuple_struct_visitor() {
    let err = DeserializeViaSeu::<TupleStruct>::deserialize(UnitTupleStructDeserializer)
        .err()
        .unwrap();

    assert!(err.to_string().contains("invalid type: unit value"));
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
fn rejects_duplicate_named_struct_fields_by_index() {
    let fields = [
        (0_u64.into_deserializer(), json!(1)),
        (0_u64.into_deserializer(), json!(2)),
        (1_u64.into_deserializer(), json!(true)),
        (2_u64.into_deserializer(), json!("named")),
    ];
    let deserializer = serde::de::value::MapDeserializer::new(fields.into_iter());

    let err = DeserializeViaSeu::<NamedStruct>::deserialize(deserializer)
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
fn rejects_unit_for_non_empty_named_structs() {
    let err = deserialize::<NamedStruct>(json!(null)).unwrap_err();

    assert!(err.to_string().contains("invalid type: null"));
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
fn deserializes_named_structs_from_byte_field_names() {
    let deserializer = BytesKeyMapDeserializer::new(vec![
        (b"first", json!(1)),
        (b"second", json!(true)),
        (b"label", json!("named")),
    ]);

    let got = DeserializeViaSeu::<NamedStruct>::deserialize(deserializer)
        .unwrap()
        .into_data();

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
fn deserializes_enum_variants_from_numeric_and_byte_identifiers() {
    let tuple = DeserializeViaSeu::<EnumShapes<u8>>::deserialize(IndexedEnumDeserializer {
        index: 2,
        value: json!([1, true, "tuple"]),
    })
    .unwrap()
    .into_data();
    assert_eq!(tuple, EnumShapes::Tuple(1, true, "tuple".to_owned()));

    let strukt = DeserializeViaSeu::<EnumShapes<u8>>::deserialize(BytesEnumDeserializer {
        name: b"Struct",
        value: json!({
            "first": 1,
            "second": true,
            "label": "struct",
        }),
    })
    .unwrap()
    .into_data();
    assert_eq!(
        strukt,
        EnumShapes::Struct {
            first: 1,
            second: true,
            label: "struct".to_owned(),
        }
    );
}

#[test]
fn serializes_variant_indices() {
    let tuple = EnumShapes::<u8>::Tuple(1, true, "tuple".to_owned());
    assert_eq!(
        SerializeViaSeu::new(&tuple)
            .serialize(VariantIndexSerializer)
            .unwrap(),
        102
    );

    let strukt = EnumShapes::<u8>::Struct {
        first: 1,
        second: true,
        label: "struct".to_owned(),
    };
    assert_eq!(
        SerializeViaSeu::new(&strukt)
            .serialize(VariantIndexSerializer)
            .unwrap(),
        204
    );
}

#[test]
fn reports_expected_visitor_names() {
    let tuple_err = DeserializeViaSeu::<TupleStruct>::deserialize(ExpectingDeserializer)
        .err()
        .unwrap();
    assert!(tuple_err.to_string().contains("TupleStruct"));

    let named_err = DeserializeViaSeu::<NamedStruct>::deserialize(ExpectingDeserializer)
        .err()
        .unwrap();
    assert!(named_err.to_string().contains("NamedStruct"));

    let unit_err = DeserializeViaSeu::<UnitStruct>::deserialize(ExpectingDeserializer)
        .err()
        .unwrap();
    assert!(unit_err.to_string().contains("UnitStruct"));

    let enum_err = DeserializeViaSeu::<EnumShapes<u8>>::deserialize(ExpectingDeserializer)
        .err()
        .unwrap();
    assert!(enum_err.to_string().contains("EnumShapes"));

    let field_err = FieldIdx::deserialize(ExpectingIdentifierDeserializer).unwrap_err();
    assert!(field_err.to_string().contains("field"));

    let variant_err = VariantIdx::deserialize(ExpectingIdentifierDeserializer).unwrap_err();
    assert!(variant_err.to_string().contains("variant"));
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

#[test]
fn deserializes_field_identifiers_from_supported_forms() {
    let index: serde::de::value::U64Deserializer<serde::de::value::Error> =
        1_u64.into_deserializer();
    let field = FieldIdx::deserialize(index).unwrap();
    assert!(matches!(field, FieldIdx::Index(1)));

    let name: serde::de::value::StrDeserializer<serde::de::value::Error> =
        "first".into_deserializer();
    let field = FieldIdx::deserialize(name).unwrap();
    assert!(matches!(field, FieldIdx::String(name) if name == "first"));

    let bytes =
        serde::de::value::BorrowedBytesDeserializer::<serde::de::value::Error>::new(b"second");
    let field = FieldIdx::deserialize(bytes).unwrap();
    assert!(matches!(field, FieldIdx::Bytes(name) if name == b"second"));
}

#[test]
fn deserializes_variant_identifiers_from_supported_forms() {
    let index: serde::de::value::U64Deserializer<serde::de::value::Error> =
        2_u64.into_deserializer();
    let variant = VariantIdx::deserialize(index).unwrap();
    assert!(matches!(variant, VariantIdx::Index(2)));

    let name: serde::de::value::StrDeserializer<serde::de::value::Error> =
        "Tuple".into_deserializer();
    let variant = VariantIdx::deserialize(name).unwrap();
    assert!(matches!(variant, VariantIdx::String(name) if name == "Tuple"));

    let bytes =
        serde::de::value::BorrowedBytesDeserializer::<serde::de::value::Error>::new(b"Struct");
    let variant = VariantIdx::deserialize(bytes).unwrap();
    assert!(matches!(variant, VariantIdx::Bytes(name) if name == b"Struct"));
}
