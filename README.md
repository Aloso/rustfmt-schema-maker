# rustfmt-schema-maker

Generates a JSON schema for `rustfmt.toml` files. This can be used with [taplo](https://taplo.tamasfe.dev).

### Installation

You need Cargo and rustfmt installed. Installation of Cargo is explained [here](https://www.rust-lang.org/tools/install). You can install rustfmt with

```fish
rustup component add rustfmt
```

Then you can install rustfmt-schema-maker with

```fish
cargo install --git https://github.com/Aloso/rustfmt-schema-maker
```

### Usage

```fish
rustfmt-schema-maker > rustfmt_schema.json
```
