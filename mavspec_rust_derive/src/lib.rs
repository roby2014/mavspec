//! MAVSpec Rust Procedural Macros

#![warn(missing_docs)]
#![deny(rustdoc::broken_intra_doc_links)]
#![doc(
    html_logo_url = "https://gitlab.com/mavka/libs/mavspec/-/raw/main/avatar.png?ref_type=heads",
    html_favicon_url = "https://gitlab.com/mavka/libs/mavspec/-/raw/main/avatar.png?ref_type=heads"
)]

use syn::DeriveInput;

pub(crate) mod consts;
pub(crate) mod errors;
pub(crate) mod field_types;
mod message_attributes;
pub(crate) mod message_field;

pub(crate) mod enums;
pub(crate) mod message;

/// Derive MAVLink message from struct.
///
/// # Usage
///
/// Basic usage:
///
/// ```rust
/// use mavspec::rust::derive::Message;
///
/// #[derive(Clone, Debug, Message)]
/// #[message_id(255)] // Specify message ID
/// #[crc_extra(32)]   // Set CRC_EXTRA byte
/// struct CustomMessage {
///     scalar_u8: u8,
///     array_u8_4: [u8; 4],
///     #[extension] // This marks an extension fields
///     ext_array_u32_4: [u32; 4],
/// }
/// ```
///
/// ## Enums
///
/// It is possible to use custom enums as message fields types with `#[derive(Enum)]`. Each enum should have a numeric
/// representation and implement [`Default`] trait.
///
/// Fields with custom types should be attributed with `base_type` attribute (i.e. `#[base_type(u16)]`) that specifies
/// actual base type of a field. For arrays base type is a type of their elements.
///
/// It is possible to use larger base types with smaller enum representation types. In such case you have to specify the
/// representation type of an enum with `repr_type` attribute (i.e. `#[repr_type(u8)]`).
///
/// It is forbidden to use larger representation types with smaller base types.
///
/// ```rust
/// use mavspec::rust::derive::{Enum, Message};
///
/// #[repr(u8)]
/// #[derive(Clone, Copy, Debug, Default, Enum)]
/// enum Variants {
///     #[default]
///     OptionA = 0,
///     OptionB = 1,
///     OptionC = 2,
/// }
///
/// #[derive(Clone, Debug, Message)]
/// #[message_id(255)]
/// #[crc_extra(32)]
/// struct CustomMessage {
///     #[base_type(u8)]
///     scalar_u8: Variants,
///
///     #[base_type(u8)]
///     array_u8_4: [Variants; 4],
///
///     #[base_type(u16)]
///     #[repr_type(u8)]
///     large_scalar_u16: Variants,
///
///     #[base_type(u16)]
///     #[repr_type(u8)]
///     large_array_u16_4: [Variants; 4],
/// }
/// ```
///
/// ## Bitmasks
///
/// For bitmasks you can use native [bitflags](https://crates.io/crates/bitflags) flags. In such case you have to
/// attribute corresponding fields with `#[bitmask]` attribute. In the same spirit as for [enums](#enums) you have to
/// set `base_type` type and `repr_type` attributes.
///
/// ```rust
/// use mavspec::rust::derive::Message;
/// use bitflags::bitflags;
///
/// bitflags! {
///     #[derive(Clone, Copy, Debug, Default)]
///     struct Flags: u8 {
///         const FLAG_8 = 1;
///         const FLAG_7 = 1 << 1;
///         const FLAG_6 = 1 << 2;
///         const FLAG_5 = 1 << 3;
///         const FLAG_4 = 1 << 4;
///         const FLAG_3 = 1 << 5;
///         const FLAG_2 = 1 << 6;
///         const FLAG_1 = 1 << 7;
///     }
/// }
///
/// #[derive(Clone, Debug, Message)]
/// #[message_id(255)]
/// #[crc_extra(32)]
/// struct CustomMessage {
///     #[bitmask]
///     #[base_type(u8)]
///     scalar_u8: Flags,
///
///     #[bitmask]
///     #[base_type(u8)]
///     array_u8_4: [Flags; 4],
///
///     #[bitmask]
///     #[base_type(u16)]
///     #[repr_type(u8)]
///     large_scalar_u16: Flags,
///
///     #[bitmask]
///     #[base_type(u16)]
///     #[repr_type(u8)]
///     large_array_u16_4: [Flags; 4],
/// }
/// ```
#[proc_macro_derive(
    Message,
    attributes(message_id, crc_extra, extension, base_type, repr_type, bitmask)
)]
pub fn derive_mavlink_message(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input: DeriveInput = syn::parse(input).unwrap();

    let message = match message::Message::try_from(input) {
        Ok(message) => message,
        Err(err) => panic!("{}", err),
    };

    proc_macro::TokenStream::from(message.to_token_stream())
}

/// Derive MAvLink enum from enum.
///
/// # Usage
///
/// Basic usage:
///
/// ```rust
/// use mavspec::rust::derive::Enum;
///
/// #[repr(u8)]
/// #[derive(Clone, Copy, Debug, Default, Enum)]
/// enum CustomEnum {
///     #[default]
///     OptionA = 0,
///     OptionB = 1,
///     OptionC = 2,
/// }
/// ```
///
/// It is possible to use constants in variant discriminants:
///
/// ```rust
/// use mavspec::rust::derive::Enum;
///
/// const ONE: u8 = 1;
/// const TWO: u8 = 2;
///
/// #[derive(Enum)]
/// #[repr(u8)]
/// #[derive(Copy, Clone, Debug, Default)]
/// enum CustomEnum {
///     #[default]
///     OptionA = 0,
///     OptionB = ONE, // Constants are supported
///     OptionC = TWO, //
/// }
/// ```
///
#[proc_macro_derive(Enum)]
pub fn derive_mavlink_enum(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input: DeriveInput = syn::parse(input).unwrap();

    let mav_enum = match enums::Enum::try_from(input) {
        Ok(mav_enum) => mav_enum,
        Err(err) => panic!("{}", err),
    };

    proc_macro::TokenStream::from(mav_enum.to_token_stream())
}
