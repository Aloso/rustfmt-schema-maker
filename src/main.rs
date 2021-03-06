use std::{collections::HashMap, io::Write, process::exit, str::FromStr};

use anyhow::{anyhow, bail, Context, Result};
use regex::Regex;
use serde::Serialize;

fn main() -> Result<()> {
    let output = std::process::Command::new("cargo")
        .args(&[
            "+nightly",
            "fmt",
            "--",
            "--unstable-features",
            "--help=config",
        ])
        .stdout(std::process::Stdio::piped())
        .output()?;
    if !output.status.success() {
        eprintln!("rustfmt exited with status code {}:", output.status);
        eprintln!("  {}", String::from_utf8_lossy(&output.stderr));
        exit(1);
    }

    let output = String::from_utf8(output.stdout)?;

    let regex = Regex::new(
        r#"^([\d\w_]+)\s+(\[(.*?)\])?(<(.*?)>)?(\s*Default:\s*(.*?))?( \(unstable\))?$"#,
    )
    .unwrap();

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
    Array(Vec<Value>),
}

#[derive(Serialize, Debug, Copy, Clone, Eq, PartialEq)]
#[serde(rename_all = "snake_case")]
enum Type {
    Integer,
    Boolean,
    String,
    Object,
    Array,
}

impl Type {
    fn parse(&self, s: &str) -> Result<Value> {
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

    let mut r#type = match captures.get(5).map(|m| m.as_str()) {
        Some(s) => s.parse::<Type>()?,
        None => Type::String,
    };

    let mut r#enum = if let Some(s) = captures.get(3) {
        let s = s.as_str();
        if s.starts_with('<') && s.ends_with(">,..") {
            s[1..s.len() - 4].parse::<Type>()?;
            r#type = Type::Array;
            None
        } else {
            let items = s
                .split('|')
                .map(|s| Type::String.parse(s))
                .collect::<Result<Vec<_>>>()?;
            Some(items)
        }
    } else {
        None
    };

    if r#enum.is_none() && r#type == Type::Boolean {
        r#enum = Some(vec![Value::Boolean(true), Value::Boolean(false)]);
    }

    let default = captures
        .get(7)
        .map(|s| r#type.parse(s.as_str()))
        .transpose()?;

    let unstable = captures.get(8).is_some();
    let desc = if unstable {
        String::from(desc) + "\n\n### Unstable\nThis option requires Nightly Rust."
    } else {
        String::from(desc)
    };

    Ok((
        key,
        Item {
            description: Some(desc),
            r#enum,
            default,
            r#type: Some(r#type),
            ..Item::default()
        },
    ))
}
