use std::collections::BTreeMap;
use std::io::Write;
use std::process::exit;

use anyhow::{anyhow, bail, Context, Result};
use regex::Regex;

use item::{Item, Value};
use typ::Type;

mod item;
mod typ;

fn main() -> Result<()> {
    let regex = get_regex();

    let output_nightly = run_rustfmt(&[
        "+nightly",
        "fmt",
        "--",
        "--unstable-features",
        "--help=config",
    ])?;
    let output_stable = run_rustfmt(&["fmt", "--", "--help=config"])?;
    let mut items_nightly = parse_rustfmt_output(&output_nightly, &regex)?;
    let items_stable = parse_rustfmt_output(&output_stable, &regex)?;

    for (k, item) in items_nightly.iter_mut() {
        if !items_stable.contains_key(k) {
            let desc = item.description.as_mut().unwrap();
            desc.push_str("\n\n### Unstable\nThis option requires Nightly Rust.");
        }
    }

    let mut stdout = std::io::stdout();
    serde_json::to_writer_pretty(
        &mut stdout,
        &Item {
            schema: Some("http://json-schema.org/draft-07/schema#"),
            title: Some("rustfmt schema".into()),
            r#type: Some(Type::Object),
            description: Some("https://rust-lang.github.io/rustfmt".into()),
            properties: Some(items_nightly),
            ..Item::default()
        },
    )?;
    writeln!(stdout)?;

    Ok(())
}

fn run_rustfmt(args: &[&str]) -> Result<String> {
    let output = std::process::Command::new("cargo")
        .args(args)
        .stdout(std::process::Stdio::piped())
        .output()?;

    if !output.status.success() {
        eprintln!("rustfmt exited with status code {}:", output.status);
        eprintln!("  {}", String::from_utf8_lossy(&output.stderr));
        exit(1);
    }

    let output = String::from_utf8(output.stdout)?;
    Ok(output)
}

fn parse_rustfmt_output(output: &str, regex: &Regex) -> Result<BTreeMap<String, Item>> {
    output
        .trim()
        .trim_start_matches("Configuration Options:")
        .trim_start()
        .replace('\r', "")
        .split("\n\n")
        .map(|s| to_item(s, regex))
        .collect()
}

fn get_regex() -> Regex {
    Regex::new(r#"^([\d\w_]+)\s+(\[(.*?)\])?(<(.*?)>)?(\s*Default:\s*(.*?))?( \(unstable\))?$"#)
        .unwrap()
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

    let key: String = captures[1].into();

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

    let desc = String::from(desc)
        + "\n\n[Documentation](https://rust-lang.github.io/rustfmt/#"
        + &key
        + ")";

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
