MAVCodeGen
==========

`MAVCodeGen` is a code-generator based on MAVLink protocol XML definitions.

> ### WARNING!!!
> 
> This project is intended to be used with other [Mavka](https://gitlab.com/mavka) tools. For now its API is still
> unstable. Once the library will be successfully consumed by these projects, API will be stabilised.

Documentation can be found [here](https://docs.rs/mavspec/latest/mavspec/).

Usage
-----

Parse standard and custom XML definitions from [`./message_definitions`](./message_definitions):

```rust
use mavspec::parser::XMLInspector;

fn main() {
  // Instantiate inspector and load list of XML definitions
  let inspector = XMLInspector::new(vec![
    "./message_definitions/standard".to_string(),
    "./message_definitions/extra".to_string(),
  ])
  .unwrap();
  
  // Parse all XML definitions
  let protocol = inspector.parse().unwrap();
}
```

See [examples](#examples) for advanced usage.

CLI
---

Generate `Rust` bindings for an example [`mavlib`](examples/mavlib) library:

```shell
cargo run --bin mavcodegen --features rust -- \
    --src message_definitions/extra \
    --out examples/mavlib/src/mavlink \
    rust \
        --module-path "mavlib::mavlink"
```

Examples
--------

- [`mavlib/basic`](examples/mavlib/examples/basic.rs) — basic.
  ```shell
  cargo run --package mavlib --example basic
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
