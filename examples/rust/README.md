MAVSpec Rust Example
=======================

A simple Rust library which contains auto-generated MAVLink dialects.

# Usage

Run with default dialects:

```shell
cargo run --package mavspec_rust_example --bin rust_example
```

Run with all dialects generated:

```shell
cargo run --package mavspec_rust_example --all-features --bin rust_example
```

Run with only `common` dialect generated:

```shell
cargo run --package mavspec_rust_example --features common --bin rust_example
```

# Configuration

Dialects are generated in [`build.rs`](build.rs). MAVSpec [`Builder`](../../mavspec/src/rust/build_helper.rs) scans
package [`Cargo.toml`](Cargo.toml) for metadata and generates only specified `messages`. The `all_enums` key is
responsible for enabling all enums regardless of which messages are generated.

```toml
[package.metadata.mavspec]
messages = ["HEARTBEAT", "PROTOCOL_VERSION", "MAV_INSPECT_V1", "COMMAND_INT", "COMMAND_LONG"]
all_enums = false
```
