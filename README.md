MAVCodeGen
==========

`MAVCodeGen` is a code-generator for [MAVLink](https://mavlink.io/en/) bindings based on
[`MAVSpec`](https://gitlab.com/mavka/libs/mavspec).

> ### Note!
> 
> This project is intended to be used with other [Mavka](https://gitlab.com/mavka) tools. For now its API is moderately
> unstable. Once the library will be successfully consumed by these projects, API will be stabilised.

API documentation can be found [here](https://docs.rs/mavcodegen/latest/mavcodegen/).

Install
-------

Install MAVCodeGen with cargo:

```shell
cargo add mavcodegen
```

# Usage

> The following explains how to use library API, for command-line tool usage check [CLI](#cli) section.

Basic usage:

```rust
use std::path::PathBuf;
use mavcodegen::rust::{Generator, GeneratorParams};
use mavspec::parser::XMLInspector;

fn main() {
    // Parse XML definitions
    let protocol = XMLInspector::builder()
        // Paths to XML definitions directories
        .set_sources(vec![
            "./message_definitions/standard".to_string(),
            "./message_definitions/extra".to_string(),
        ])
        // Build configuration and parse dialects
        .build().unwrap()
        .parse().unwrap();

    // Generate MAVLink dialects
    let generator = Generator::new(
        protocol,
        PathBuf::from("./tmp/mavlink"),
        GeneratorParams {
            serde: true,
            ..Default::default()
        },
    );
    generator.generate().unwrap();
}
```

### As build dependency

In most scenarios you'd like to generate code as a part of your build sequence. For such cases MAVCodeGen provides a
`Builder` helper. First, add MAVCodeGen as a build dependency to your `Cargo.toml`:

```toml
[build-dependencies]
# ...
mavcodegen = "0.1.0-alpha1"
# ...
```

If necessary, add optional section to your `Cargo.toml` to generate only specific messages:

```toml
[package.metadata.mavcodegen]
messages = ["HEARTBEAT", "PROTOCOL_VERSION", "MAV_SPEC_V1", "COMMAND_INT", "COMMAND_LONG"]
all_enums = false
```

This will greatly reduce compile time and may slightly reduce memory footprint. 

The `all_enum` key controls which enums will be generated. By default, only MAVLink enums required for selected messages
will be generated. Set `all_enums = true` to generate all enums. If `messages` key is not specified, then `all_enums`
won't have any effect.

Build MAVLink bindings in your `build.rs`:

```rust
use std::env::var;
use std::path::Path;

use mavcodegen::rust::BuildHelper;

fn main() {
    // Assume that your library and `message_definitions` are both in the root of your project.
    let sources = vec![
        Path::new("./message_definitions/standard"),
        Path::new("./message_definitions/extra"),
    ];
    // Output path
    let destination = Path::new(&var("OUT_DIR").unwrap()).join("mavlink");
    // Path to your `Cargo.toml` manifest
    let manifest_path = Path::new(env!("CARGO_MANIFEST_DIR")).join("Cargo.toml");

    BuildHelper::builder(&sources, &destination)
        .set_manifest_path(&manifest_path)
        .generate()
        .unwrap();
}
```

Finally, import generated code in your `lib.rs`:

```rust
mod mavlink {
    include!(concat!(env!("OUT_DIR"), "/mavlink/mod.rs"));
}
pub use mavlink::dialects;
```

Check [`examples/rust`](examples/rust/README.md) for a more elaborated example.

CLI
---

Parse XML definitions from [`./message_definitions/standard`](./message_definitions/standard) and generate dialects in
`tmp/mavlink` directory:

```shell
cargo run --bin mavcodegen -- --src message_definitions/standard --out tmp/mavlink rust
```

Print `mavcodegen` help for Rust code generator:

```shell
cargo run --bin mavcodegen -- rust -h
```

Examples
--------

- [`examples/rust`](examples/rust/README.md) — an example library with autogenerated code.
  ```shell
  cargo run --package mavcodegen_rust_example --bin rust_example
  ```

License
-------

> Here we simply comply with the suggested dual licensing according to
> [Rust API Guidelines](https://rust-lang.github.io/api-guidelines/about.html) (C-PERMISSIVE).

Licensed under either of

* Apache License, Version 2.0
  ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
* MIT license
  ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.

Contribution
------------

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in the work by you, as defined in the Apache-2.0 license, shall be
dual licensed as above, without any additional terms or conditions.
