use std::collections::HashSet;

use crate::library::common::from_ivec;
use bigdecimal::BigDecimal;
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use sled::IVec;

#[derive(Eq, PartialEq, Debug, Default, Serialize, Deserialize, Clone)]
pub struct State {
    #[serde(
        serialize_with = "serialize_bigdecimal",
        deserialize_with = "deserialize_bigdecimal"
    )]
    pub balance: BigDecimal,
    pub input_hashs: HashSet<String>,
}

impl State {
    pub fn new(balance: BigDecimal, input_hashs: HashSet<String>) -> Self {
        Self {
            balance,
            input_hashs,
        }
    }
    pub fn new_with_iterable<I: IntoIterator<Item = String>>(
        balance: BigDecimal,
        input_hashs: I,
    ) -> Self {
        let input_hashs: HashSet<String> = input_hashs.into_iter().collect();
        Self {
            balance,
            input_hashs,
        }
    }

    pub fn update<I: IntoIterator<Item = String>>(&mut self, balance: BigDecimal, input_hashs: I) {
        self.balance = balance;
        self.input_hashs.extend(input_hashs);
    }

    pub fn merge(&mut self, other_state: Self) {
        self.balance = other_state.balance;
        self.input_hashs.extend(other_state.input_hashs);
    }

    pub fn from<K, V>(key: &IVec, value: &IVec) -> (K, V)
    where
        K: for<'a> serde::Deserialize<'a> + Default,
        V: for<'a> serde::Deserialize<'a> + Default,
    {
        (from_ivec::<K>(key), from_ivec::<V>(value))
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
