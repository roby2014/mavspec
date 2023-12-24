MAVSpec: Rust Generation Tools
==============================

Rust code generation module for MAVSpec. Supports `no-std` (including `no-alloc`) targets. Provides optional
[Serde](https://serde.rs) support.

[`repository`](https://gitlab.com/mavka/libs/mavspec)
[`crates.io`](https://crates.io/crates/mavspec)
[`API docs`](https://docs.rs/mavspec/latest/mavspec/)
[`issues`](https://gitlab.com/mavka/libs/mavspec/-/issues)

This crate is a part of [MAVSpec](https://gitlab.com/mavka/libs/mavinspect) code generation toolchain for
[MAVLink](https://mavlink.io/en/). While nothing prevents from using this crate directly, such approach
usage is not encouraged (and not documented). We suggest to import this module from Mavspec as `mavspec::rust::gen`.

This project is a member of [Mavka](https://mavka.gitlab.io/home/) family.
