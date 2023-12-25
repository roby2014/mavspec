MAVSpec Rust Example
=======================

A simple Rust library which contains auto-generated MAVLink dialects.

# Usage

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

# Configuration

Dialects are generated in [`build.rs`](build.rs). MAVSpec [`BuildeHelper`](../../mavspec_rust_gen/src/build_helper.rs)
scans package [`Cargo.toml`](Cargo.toml) for metadata and generates only specified `messages`. The `all_enums` key is
responsible for enabling all enums regardless of which messages are generated.

```toml
[package.metadata.mavspec]
messages = ["HEARTBEAT", "PROTOCOL_VERSION", "MAV_INSPECT_V1", "COMMAND_INT", "COMMAND_LONG"]
all_enums = false
```
