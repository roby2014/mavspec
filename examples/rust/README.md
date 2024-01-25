MAVSpec Rust Example
=======================

A simple Rust library which contains auto-generated MAVLink dialects.

Usage
-----

Run with default dialects:

```shell
cargo run --package mavspec_examples_rust --bin mavspec_examples_rust
```

Run with all dialects generated:

```shell
cargo run --package mavspec_examples_rust --all-features --bin mavspec_examples_rust
```

Run with only `common` dialect generated:

```shell
cargo run --package mavspec_examples_rust --features common --bin mavspec_examples_rust
```

Configuration
-------------

Dialects are generated in [`build.rs`](build.rs). MAVSpec [`BuildeHelper`](../../mavspec_rust_gen/src/build_helper.rs)
scans package [`Cargo.toml`](Cargo.toml) for metadata and generates only specified MAVLink entities.

```toml
[package.metadata.mavspec]
microservices = ["HEARTBEAT", "MISSION"]
messages = ["PROTOCOL_VERSION", "MAV_INSPECT_V1", "PING"]
enums = ["STORAGE_STATUS", "GIMBAL_*"]
commands = ["MAV_CMD_DO_CHANGE_SPEED", "MAV_CMD_DO_SET_ROI*"]
generate_tests = false
```

Caveats
-------

Note that [`message_definitions`](../../message_definitions) was symlinked into package directory. It is necessary
to make `cargo publish` work properly. Although we don't publish example packages, we want to keep them as close as
possible to what people might use in production.
