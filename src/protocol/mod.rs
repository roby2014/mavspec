//! MAVLink protocol entities.
//!
//! Internal entities are useful for message manipulations in Rust libraries which use MAVSpec.
#![warn(missing_docs)]

#[allow(clippy::module_inception)]
mod protocol;
pub use protocol::Protocol;

mod dialect;
pub use dialect::Dialect;

mod enums;
pub use enums::{Enum, EnumEntry, EnumEntryMavCmdFlags, EnumEntryMavCmdParam};

mod messages;
pub use messages::{
    Message, MessageBuilder, MessageField, MessageFieldBuilder, MessageFieldInvalidValue, MessageId,
};

mod common;
pub use common::deprecated::{Deprecated, DeprecatedSince};
pub use common::mav_type::MavType;
pub use common::units::Units;
pub use common::value::Value;

// Errors
pub mod errors;

/// Builders.
///
/// Builders available for [`protocol`](self) entities.
///
/// See: [`Builder`](traits::Builder).
pub mod builders {
    pub use super::enums::{
        EnumBuilder, EnumEntryBuilder, EnumEntryMavCmdFlagsBuilder, EnumEntryMavCmdParamBuilder,
    };
    pub use super::messages::{MessageBuilder, MessageFieldBuilder};
}
