use std::{collections::HashMap, io::Write};

use anyhow::{anyhow, bail, Context, Result};
use regex::Regex;
use serde::Serialize;

fn main() -> Result<()> {
    let output = std::process::Command::new("rustfmt")
        .arg("--help=config")
        .stdout(std::process::Stdio::piped())
        .output()?;
    let output = String::from_utf8(output.stdout)?;

    let regex =
        Regex::new(r#"^([\d\w_]+)\s+(\[(.*?)\])?(<(.*?)>)?(\s*Default:\s*(.*))?$"#).unwrap();

    let items = output
        .trim()
        .trim_start_matches("Configuration Options:")
        .trim_start()
        .replace('\r', "")
        .split("\n\n")
        .map(|s| to_item(s, &regex))
        .collect::<Result<HashMap<String, Item>>>()?;

    let mut stdout = std::io::stdout();
    serde_json::to_writer_pretty(
        &mut stdout,
        &Item {
            schema: Some("http://json-schema.org/draft-07/schema#"),
            title: Some("rustfmt schema".into()),
            r#type: Some(Type::Object),
            description: Some("https://rust-lang.github.io/rustfmt".into()),
            properties: Some(items),
            ..Item::default()
        },
    )?;
    writeln!(stdout)?;

    Ok(())
}

#[derive(Default, Serialize)]
struct Item {
    #[serde(rename = "$schema")]
    #[serde(skip_serializing_if = "Option::is_none")]
    schema: Option<&'static str>,

    #[serde(skip_serializing_if = "Option::is_none")]
    title: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    r#type: Option<Type>,

    #[serde(skip_serializing_if = "Option::is_none")]
    description: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    default: Option<Value>,

    #[serde(skip_serializing_if = "Option::is_none")]
    r#enum: Option<Vec<Value>>,

    #[serde(skip_serializing_if = "Option::is_none")]
    properties: Option<HashMap<String, Item>>,
}

#[derive(Serialize, Debug)]
#[serde(untagged)]
enum Value {
    Integer(i64),
    Boolean(bool),
    String(String),
}

#[derive(Serialize, Debug, Copy, Clone)]
#[serde(rename_all = "snake_case")]
enum Type {
    Integer,
    Boolean,
    String,
    Object,
}

impl Type {
    fn parse(self, s: &str) -> Result<Value> {
        Ok(match self {
            Type::Integer => Value::Integer(s.parse()?),
            Type::Boolean => Value::Boolean(match s {
                "true" => true,
                "false" => false,
                s => bail!("Unknown boolean literal {:?}", s),
            }),
            Type::String => Value::String(s.into()),
            t => bail!("Can't parse this type: {:?}", t),
        })
    }
}

fn to_item(input: &str, regex: &Regex) -> Result<(String, Item)> {
    let mut lines = input.lines();
    let fst = lines
        .next()
        .with_context(|| anyhow!("First line missing: {:?}", input))?
        .trim();
    let desc = lines
        .next()
        .with_context(|| anyhow!("Second line missing: {:?}", input))?
        .trim();
    if lines.next().is_some() {
        bail!("Unexpected third line {:?}", input);
    }

    let captures = regex
        .captures(fst)
        .with_context(|| anyhow!("String could not be parsed: {:?}", fst))?;

    let key = captures[1].into();

    let r#type: Type = match captures.get(5).map(|m| m.as_str()) {
        Some("integer") | Some("unsigned integer") => Type::Integer,
        Some("boolean") => Type::Boolean,
        Some(s) => bail!("Unknown type {:?}", s),
        None => Type::String,
    };

    let r#enum = captures
        .get(3)
        .map(|s| {
            s.as_str()
                .split('|')
                .map(|s| Type::String.parse(s))
                .collect::<Result<Vec<_>>>()
        })
        .transpose()?;

    let default = captures
        .get(7)
        .map(|s| r#type.parse(s.as_str()))
        .transpose()?;

    Ok((
        key,
        Item {
            description: Some(desc.into()),
            r#enum,
            default,
            r#type: Some(r#type),
            ..Item::default()
        },
    ))
}
