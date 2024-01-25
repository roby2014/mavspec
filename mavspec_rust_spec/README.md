MAVSpec: Rust Specification
===========================

Core interfaces for [MAVSpec](https://gitlab.com/mavka/libs/mavspec)'s Rust code generation toolchain. Supports
`no-std` (including `no-alloc`) targets. Provides optional [Serde](https://serde.rs) support.

[`repository`](https://gitlab.com/mavka/libs/mavspec)
[`crates.io`](https://crates.io/crates/mavspec)
[`API docs`](https://docs.rs/mavspec/latest/mavspec/)
[`issues`](https://gitlab.com/mavka/libs/mavspec/-/issues)

This crate is a part of [MAVSpec](https://gitlab.com/mavka/libs/mavinspect) code generation toolchain for
[MAVLink](https://mavlink.io/en/). While nothing prevents from using this crate directly, such approach
is not encouraged (and not documented). We suggest to import this module from Mavspec as `mavspec::rust::gen`.

This project is a member of [Mavka](https://mavka.gitlab.io/home/) family.

# Cargo Features

* `alloc` — enables global allocator. Incompatible with `no-alloc` targets.  
* `std` — enables Rust standard library. Enables `alloc`. Incompatible with `no-std` targets.  
* `serde` — enables [Serde](https://serde.rs) support. It will be included with corresponding `std`/`alloc`
  features (or without them).

License
-------

> Here we simply comply with the suggested dual licensing according to
> [Rust API Guidelines](https://rust-lang.github.io/api-guidelines/about.html) (C-PERMISSIVE).

Licensed under either of

* Apache License, Version 2.0
  ([LICENSE-APACHE](../LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
* MIT license
  ([LICENSE-MIT](../LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.

Contribution
------------

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in the work by you, as defined in the Apache-2.0 license, shall be
dual licensed as above, without any additional terms or conditions.
