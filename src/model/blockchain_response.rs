use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum Either<T, E> {
    Right(T),
    Left(E),
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ResultError {
    pub value: Value,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct Value {
    #[serde(rename = "msg")]
    pub msg: String,
}

impl Value {
    pub fn is_not_found_err(&self) -> bool {
        self.msg.contains("not found")
    }
}
