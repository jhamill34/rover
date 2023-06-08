use serde::{de::Visitor, Deserialize, Serialize, ser::{SerializeSeq, SerializeMap}};

#[derive(Clone, Debug, PartialEq, Eq)] 
pub enum Value {
    Null, 
    Bool(bool),
    String(String),
    Number(serde_json::Number),
    Array(Vec<Value>),
    Object(indexmap::IndexMap<String, Value>),
}

struct ValueVisitor;
impl <'de> Visitor<'de> for ValueVisitor {
    type Value = Value;

    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        formatter.write_str("a valid JSON value")
    }

    fn visit_unit<E>(self) -> Result<Self::Value, E>
        where E: serde::de::Error, {
        Ok(Value::Null)
    }

    fn visit_bool<E>(self, value: bool) -> Result<Self::Value, E> {
        Ok(Value::Bool(value))
    }

    fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
        where E: serde::de::Error, {
        self.visit_string(v.to_owned())
    }

    fn visit_u64<E>(self, v: u64) -> Result<Self::Value, E>
        where E: serde::de::Error, {
        Ok(Value::Number(v.into()))
    }

    fn visit_i64<E>(self, v: i64) -> Result<Self::Value, E>
        where E: serde::de::Error, {
        Ok(Value::Number(v.into()))
    }

    fn visit_f64<E>(self, v: f64) -> Result<Self::Value, E>
        where E: serde::de::Error, {
        Ok(serde_json::Number::from_f64(v).map_or(Value::Null, Value::Number))
    }

    fn visit_string<E>(self, v: String) -> Result<Self::Value, E>
        where E: serde::de::Error, {
        Ok(Value::String(v))
    }

    fn visit_none<E>(self) -> Result<Self::Value, E>
        where E: serde::de::Error, {
        Ok(Value::Null)
    }

    fn visit_some<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
        where D: serde::Deserializer<'de>, {
        Deserialize::deserialize(deserializer)
    }

    fn visit_seq<V>(self, mut visitor: V) -> Result<Self::Value, V::Error>
        where V: serde::de::SeqAccess<'de>, {
        let mut vec = Vec::new();
        while let Some(elem) = visitor.next_element()? {
            vec.push(elem);
        }
        Ok(Value::Array(vec))
    }

    fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
        where A: serde::de::MapAccess<'de>, {
        let mut values = indexmap::IndexMap::new();

        while let Some((key, value)) = map.next_entry()? {
            values.insert(key, value);
        }

        Ok(Value::Object(values))
    }
}

impl <'de> Deserialize<'de> for Value {
    #[inline]
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
        where D: serde::Deserializer<'de> {
        deserializer.deserialize_any(ValueVisitor)
    }
}

impl Serialize for Value {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
        where S: serde::Serializer {
        match *self {
            Value::Null => serializer.serialize_unit(),
            Value::Bool(v) => serializer.serialize_bool(v),
            Value::Number(ref v) => v.serialize(serializer),
            Value::String(ref v) => serializer.serialize_str(v),
            Value::Array(ref v) => {
                let mut seq = serializer.serialize_seq(Some(v.len()))?;
                for item in v {
                    seq.serialize_element(item)?;
                }
                seq.end()
            }
            Value::Object(ref v) => {
                let mut map = serializer.serialize_map(Some(v.len()))?;
                for (key, value) in v {
                    map.serialize_entry(key, value)?;
                }
                map.end()
            },
        }
    }
}


