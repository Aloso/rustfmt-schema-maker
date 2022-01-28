use std::collections::BTreeMap;

use serde::Serialize;

use crate::typ::Type;

#[derive(Default, Serialize)]
pub struct Item {
    #[serde(rename = "$schema")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub schema: Option<&'static str>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub r#type: Option<Type>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub default: Option<Value>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub r#enum: Option<Vec<Value>>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub properties: Option<BTreeMap<String, Item>>,
}

#[derive(Serialize, Debug)]
#[serde(untagged)]
pub enum Value {
    Integer(i64),
    Boolean(bool),
    String(String),
    Array(Vec<Value>),
}
