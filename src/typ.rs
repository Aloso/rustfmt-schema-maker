use std::str::FromStr;

use anyhow::{bail, Result};
use serde::Serialize;

use crate::item::Value;

#[derive(Serialize, Debug, Copy, Clone, Eq, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum Type {
    Integer,
    Boolean,
    String,
    Object,
    Array,
}

impl Type {
    pub fn parse(&self, s: &str) -> Result<Value> {
        Ok(match self {
            Type::Integer => Value::Integer(s.parse()?),
            Type::Boolean => Value::Boolean(match s {
                "true" => true,
                "false" => false,
                s => bail!("Unknown boolean literal {:?}", s),
            }),
            Type::String => Value::String(s.into()),
            Type::Array if s == "[]" => Value::Array(vec![]),
            t => bail!("Can't parse this type: {:?}", t),
        })
    }
}

impl FromStr for Type {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(match s {
            "integer" | "unsigned integer" => Type::Integer,
            "boolean" => Type::Boolean,
            "string" => Type::String,
            _ => bail!("Unknown type {:?}", s),
        })
    }
}
