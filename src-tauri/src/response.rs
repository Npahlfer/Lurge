use serde::ser::{Serialize, SerializeStruct, Serializer};

pub enum ResponseValue {
    String(String),
    Bool(bool),
}

impl Serialize for ResponseValue {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match self {
            ResponseValue::String(s) => serializer.serialize_str(s),
            ResponseValue::Bool(b) => serializer.serialize_bool(*b),
        }
    }
}

pub struct ResponseResult<T> {
    pub success: bool,
    pub result: T,
    pub error: String,
}

impl<T: Serialize> Serialize for ResponseResult<T> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut s = serializer.serialize_struct("ResponseResult", 3)?;
        s.serialize_field("success", &self.success)?;
        s.serialize_field("error", &self.error)?;
        s.serialize_field("result", &self.result)?;
        s.end()
    }
}
