MAVSpec: Rust Generation Tools
==============================

Rust code generation module for MAVSpec. Supports `no-std` (including `no-alloc`) targets. Provides optional
[Serde](https://serde.rs) support.

<span style="font-size:24px">[🇺🇦](https://mavka.gitlab.io/home/a_note_on_the_war_in_ukraine/)</span>
[![`repository`](https://img.shields.io/gitlab/pipeline-status/mavka/libs/mavspec.svg?branch=main&label=repository)](https://gitlab.com/mavka/libs/mavspec)
[![`crates.io`](https://img.shields.io/crates/v/mavspec.svg)](https://crates.io/crates/mavspec)
[![`docs.rs`](https://img.shields.io/docsrs/mavspec.svg?label=docs.rs)](https://docs.rs/mavspec/latest/mavspec/)
[![`issues`](https://img.shields.io/gitlab/issues/open/mavka/libs/mavspec.svg)](https://gitlab.com/mavka/libs/mavspec/-/issues/)

This crate is a part of [MAVSpec](https://gitlab.com/mavka/libs/mavspec) code generation toolchain for
[MAVLink](https://mavlink.io/en/). While nothing prevents from using this crate directly, such approach
is not encouraged (and not documented). We suggest to import this module from Mavspec as `mavspec::rust::gen`.

This project is a member of [Mavka](https://mavka.gitlab.io/home/) family.

`License
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
dual licensed as above, without any additional terms or conditions.`
