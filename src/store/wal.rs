use std::collections::HashSet;

use bigdecimal::BigDecimal;
use serde::{Serialize, Deserialize, Serializer, Deserializer};

#[derive(Debug, Default, Serialize, Deserialize, Clone)]
pub struct State {
  #[serde(serialize_with = "serialize_bigdecimal", deserialize_with = "deserialize_bigdecimal")]
  balance: BigDecimal,
  input_hashs: HashSet<String>,
}

impl State {
  pub fn new(balance: BigDecimal, input_hashs: HashSet<String>) -> Self {
    Self { balance, input_hashs }
  }

  pub fn update(&mut self, balance: BigDecimal, input_hashs: HashSet<String>) {
    self.balance = balance;
    self.input_hashs.extend(input_hashs);
  }
}

fn serialize_bigdecimal<S>(value: &BigDecimal, serializer: S) -> Result<S::Ok, S::Error>
where
  S: Serializer,
{
  let s = value.to_string();
  serializer.serialize_str(&s)
}

fn deserialize_bigdecimal<'de, D>(deserializer: D) -> Result<BigDecimal, D::Error>
where
  D: Deserializer<'de>,
{
  let s = String::deserialize(deserializer)?;
  s.parse::<BigDecimal>().map_err(serde::de::Error::custom)
}
