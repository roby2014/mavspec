MAVSpec
=======

Rust library for parsing [MAVLink](https://mavlink.io/en/) XML definitions.

> ### WARNING!!!
> 
> This project is intended to be used with other [Mavka](https://gitlab.com/mavka) tools. For now its API is still
> relatively unstable. Once the library will be successfully consumed by these projects, API will be stabilised.

Documentation can be found [here](https://docs.rs/mavspec/latest/mavspec/).

Usage
-----

Parse standard and custom XML definitions from [`./message_definitions`](./message_definitions):

```rust
use std::env;

use mavspec::parser::XMLInspector;

fn main() {
    // Instantiate inspector and load list of XML definitions
    let inspector = XMLInspector::builder()
        .set_sources(vec![
            "./message_definitions/standard".to_string(),
            "./message_definitions/extra".to_string(),
        ])
        .build()
        .unwrap();
    
    // Parse all XML definitions
    let protocol = inspector.parse().unwrap();
    
    // Get `crazyflight` custom dialect
    let crazyflight = protocol.dialects().get("crazyflight").unwrap();
    
    // Get `DUMMYFLIGHT_OUTCRY` message
    let outcry_message = crazyflight.messages().get(&54000u32).unwrap();
    assert_eq!(outcry_message.name(), "CRAZYFLIGHT_OUTCRY");
    println!("\n`CRAZYFLIGHT_OUTCRY` message: {:#?}", outcry_message);
}
```

See [examples](#examples) for advanced usage.

Examples
--------

- [`parser`](./examples/parser.rs) — parse XML definitions.
  ```shell
  cargo run --example parser --features=serde
  ```

Roadmap
-------

**Basics** (required before publishing non-alpha version)

- [x] Add standalone examples.
- [x] Accept multiple folders with XML definitions.
- [x] Implement CRC calculation.
- [x] Add more utility functions for fields management.
- [ ] Add more examples to docs.

**Advanced** (considered for the next milestone)

- [ ] Create abstractions for [MAVLink commands](https://mavlink.io/en/services/command.html).

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
