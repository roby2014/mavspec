use quote::{format_ident, quote};

use crate::conventions::{
    dialect_mod_name, enum_rust_name, message_mod_name, message_raw_struct_name,
    message_struct_name, rust_default_value, rust_var_name, t_bytes_read_fn, t_bytes_write_fn,
};
use crate::specs::dialects::dialect::messages::{
    FieldSpec, MessageImplModuleSpec, MessageInheritedModuleSpec, MessagesRootModuleSpec,
};
use crate::specs::Spec;
use crate::templates::helpers::{make_serde_arrays_annotation, make_serde_derive_annotation};

/// Messages root module template.
pub(crate) fn messages_root_module(spec: &MessagesRootModuleSpec) -> syn::File {
    let module_doc_comment = format!(" MAVLink messages of `{}` dialect.", spec.dialect_name());

    let message_modules_and_imports = spec.messages().values().map(|msg| {
        let message_mod_name = format_ident!("{}", message_mod_name(msg.name().into()));
        let message_struct_name = format_ident!("{}", message_struct_name(msg.name().into()));
        quote! {
            pub mod #message_mod_name;
            pub use #message_mod_name::#message_struct_name;
        }
    });

    syn::parse2(quote! {
        #![doc = #module_doc_comment]

        #(#message_modules_and_imports)*
    })
    .unwrap()
}

pub(crate) fn message_module(spec: &MessageImplModuleSpec) -> syn::File {
    let module_doc_comment = format!(" # MAVLink `{}` message implementation.", spec.name());
    let message_id: syn::LitInt = syn::parse_str(format!("{}", spec.id()).as_str()).unwrap();
    let crc_extra: syn::LitInt = syn::parse_str(format!("{}", spec.crc_extra()).as_str()).unwrap();
    let message_leading_doc_comment = format!(" MAVLink `{}` message.", spec.name());
    let min_supported_mavlink_version_number = if spec.is_v1_compatible() { 1 } else { 2 };
    let min_supported_mavlink_version_doc_comment = format!(
        " Minimum supported MAVLink version is `MAVLink {min_supported_mavlink_version_number}`."
    );
    let description_doc_comments = spec.description().iter().map(|line| {
        quote! { #[doc = #line] }
    });
    let derive_serde = make_serde_derive_annotation(spec.params().serde);

    let message_struct_ident = spec.ident();
    let message_encode_decode_doc_comment =
        format!(" [`{message_struct_ident}`] (encoding) and [`IntoPayload`] (decoding) traits.");
    let message_fields = spec.fields().iter().map(|field| {
        let leading_doc_comment = format!(" MAVLink field `{}`.", field.name());
        let description_doc_comments = field.description().iter().map(|line| {
            quote! { #[doc = #line] }
        });
        let serde_arrays_attr = make_serde_arrays_annotation(field.requires_serde_arrays());
        let field_ident = format_ident!("{}", rust_var_name(field.name().into()));
        let field_rust_type: syn::Type =
            syn::parse_str(field.r#type().rust_type().as_str()).unwrap();
        let enum_ident = format_ident!("{}", enum_rust_name(field.enum_name().into()));
        let array_length = field.array_length();

        let field_definition = if field.is_enum() {
            if field.is_array() {
                quote! {
                    pub #field_ident: [super::super::enums::#enum_ident; #array_length],
                }
            } else {
                quote! {
                    pub #field_ident: super::super::enums::#enum_ident,
                }
            }
        } else {
            quote! {
                pub #field_ident: #field_rust_type,
            }
        };

        quote! {
            #[doc = #leading_doc_comment]
            ///
            #(#description_doc_comments)*
            #serde_arrays_attr
            #field_definition
        }
    });

    let message_raw_struct_ident = format_ident!("{}", message_raw_struct_name(spec.name().into()));
    let message_raw_leading_doc_comment = format!(" Raw representation of [`{message_raw_struct_ident}`](struct@self::{message_raw_struct_ident}) MAVLink message.");
    let message_raw_fields = spec.fields().iter().map(|field| {
        let leading_doc_comment = format!(" MAVLink field `{}`.", field.name());
        let serde_arrays_attr = make_serde_arrays_annotation(field.requires_serde_arrays());
        let field_ident = format_ident!("{}", rust_var_name(field.name().into()));
        let field_rust_type: syn::Type =
            syn::parse_str(field.r#type().rust_type().as_str()).unwrap();

        quote! {
            #[doc = #leading_doc_comment]
            #serde_arrays_attr
            pub #field_ident: #field_rust_type,
        }
    });

    let message_default_fields = spec.fields().iter().map(|field| {
        let field_ident = format_ident!("{}", rust_var_name(field.name().into()));
        let enum_ident = format_ident!("{}", enum_rust_name(field.enum_name().into()));
        let array_length = field.array_length();

        if field.is_enum() {
            if field.is_array() {
                quote! {
                    #field_ident: [super::super::enums::#enum_ident::default(); #array_length],
                }
            } else {
                quote! {
                    #field_ident: super::super::enums::#enum_ident::default(),
                }
            }
        } else {
            let default_value: syn::Expr =
                syn::parse_str(rust_default_value(field.r#type().clone()).as_str()).unwrap();
            quote! {
                #field_ident: #default_value,
            }
        }
    });

    let message_try_from_payload_doc_comment = format!(" Decodes [`Payload`] into [`{message_struct_ident}`](struct@self::{message_struct_ident}) according to [`MavLinkVersion`].");
    let message_try_from_payload_arm_v1 = if spec.is_v1_compatible() {
        quote! {
            MavLinkVersion::V1 => v1::decode(value.bytes()),
        }
    } else {
        quote! {
            version => {
                return Err(MessageError::UnsupportedMavLinkVersion {
                    actual: version,
                    minimal: MESSAGE_INFO.min_supported_mavlink_version(),
                });
            }
        }
    };

    let message_encode_doc_comment = format!(" Encodes [`{message_struct_ident}`](struct@self::{message_struct_ident}) into [`Payload`] according to [`MavLinkVersion`].");
    let message_encode_arm_v1 = if spec.is_v1_compatible() {
        quote! {
            MavLinkVersion::V1 => v1::encode(self)?,
        }
    } else {
        quote! {
            version => {
                return Err(MessageError::UnsupportedMavLinkVersion {
                    actual: version,
                    minimal: MESSAGE_INFO.min_supported_mavlink_version(),
                });
            }
        }
    };

    let message_to_message_raw_leading_doc_comment =
        format!(" Converts into raw [`{message_raw_struct_ident}`].");
    let message_to_message_raw_field_setters = spec
        .fields()
        .iter()
        .map(message_to_message_raw_field_setter);

    let message_raw_try_from_payload_doc_comment = format!(
        " Decodes [`Payload`] into [`{message_raw_struct_ident}`] according to [`MavLinkVersion`]."
    );
    let message_raw_try_from_payload_arm_v1 = if spec.is_v1_compatible() {
        quote! {
            MavLinkVersion::V1 => v1::decode_raw(value.bytes()),
        }
    } else {
        quote! {
            version => {
                return Err(MessageError::UnsupportedMavLinkVersion {
                    actual: version,
                    minimal: MESSAGE_INFO.min_supported_mavlink_version(),
                });
            }
        }
    };

    let message_raw_encode_doc_comment = format!(
        " Encodes [`{message_raw_struct_ident}`] into [`Payload`] according to [`MavLinkVersion`]."
    );
    let message_raw_encode_arm_v1 = if spec.is_v1_compatible() {
        quote! {
            MavLinkVersion::V1 => v1::encode_raw(self)?,
        }
    } else {
        quote! {
            version => {
                return Err(MessageError::UnsupportedMavLinkVersion {
                    actual: version,
                    minimal: MESSAGE_INFO.min_supported_mavlink_version(),
                });
            }
        }
    };

    let message_raw_to_message_doc_comment = format!(" Attempts to convert [`{message_raw_struct_ident}`] into [`{message_struct_ident}`](struct@self::{message_struct_ident}).");
    let message_raw_to_message_field_setters = spec
        .fields()
        .iter()
        .map(message_raw_to_message_field_setter);

    let payload_v2_size = spec.payload_v2_size();

    let mod_v2_leading_doc_comment = format!(" Encoding/decoding for [`{message_struct_ident}`](struct@self::{message_struct_ident}) within `MAVLink 2` protocol.");
    let payload_size_v2_doc_comment = format!(" Message [`{message_struct_ident}`](struct@self::{message_struct_ident}) payload size (non-truncated) according to `MAVLink 2` protocol.");
    let decode_raw_fn_leading_doc_comment =
        format!(" Decodes into [`{message_raw_struct_ident}`] message.");
    let decode_fn_leading_doc_comment = format!(
        " Decodes into [`{message_struct_ident}`](struct@self::{message_struct_ident}) message."
    );
    let encode_raw_fn_leading_doc_comment =
        format!(" Encodes [`{message_raw_struct_ident}`] message into MAVLink [`Payload`].");
    let encode_fn_leading_doc_comment = format!(" Encodes [`{message_struct_ident}`](struct@self::{message_struct_ident}) message into MAVLink [`Payload`].");

    let v2_decode_raw_field_readers = spec.fields_v2().iter().map(|field| {
        let field_ident = format_ident!("{}", rust_var_name(field.name().into()));
        let reader_fn = format_ident!("{}", t_bytes_read_fn(field.r#type().clone()));

        quote! {
            #field_ident: reader.#reader_fn()?,
        }
    });

    let v2_encode_raw_field_writers = spec.fields_v2().iter().map(|field| {
        let field_ident = format_ident!("{}", rust_var_name(field.name().into()));
        let write_fn = format_ident!("{}", t_bytes_write_fn(field.r#type().clone()));

        quote! {
            writer.#write_fn(message.#field_ident)?;
        }
    });

    let v1_encode_decode_mod = message_v1_encode_decode_mod(spec);

    let tests = generate_tests(spec);

    syn::parse2(quote! {
        #![doc = #module_doc_comment]

        use mavspec::rust::spec::{
            IntoPayload, MavLinkVersion, MessageError, MessageImpl, MessageInfo, MessageSpec, Payload,
        };
        use mavspec::rust::spec::types::{MessageId, CrcExtra};

        /// Message ID.
        pub(crate) const MESSAGE_ID: MessageId = #message_id;
        /// `CRC_EXTRA` calculated from message XML definition.
        pub(crate) const CRC_EXTRA: CrcExtra = #crc_extra;
        /// Generic message info that contains all message metadata.
        pub(crate) const MESSAGE_INFO: MessageInfo = MessageInfo::new(MESSAGE_ID, CRC_EXTRA);

        /// MAVLink message specification
        #[inline]
        pub const fn spec() -> &'static dyn MessageSpec {
            &MESSAGE_INFO
        }

        #[allow(rustdoc::bare_urls)]
        #[allow(rustdoc::broken_intra_doc_links)]
        #[allow(rustdoc::invalid_rust_codeblocks)]
        #[doc = #message_leading_doc_comment]
        ///
        #[doc = #min_supported_mavlink_version_doc_comment]
        ///
        /// # Description
        ///
        #(#description_doc_comments)*
        ///
        /// # Encoding/Decoding
        ///
        /// Message encoding/decoding are provided by implementing [`TryFrom<Payload>`] for
        #[doc = #message_encode_decode_doc_comment]
        #[derive(core::clone::Clone, core::fmt::Debug)]
        #derive_serde
        pub struct #message_struct_ident {
            #(#message_fields)*
        }

        #[doc = #message_raw_leading_doc_comment]
        ///
        #[doc = #min_supported_mavlink_version_doc_comment]
        #[derive(core::clone::Clone, core::fmt::Debug)]
        #derive_serde
        pub struct #message_raw_struct_ident {
            #(#message_raw_fields)*
        }

        impl MessageSpec for #message_struct_ident {
            /// MAVLink message ID.
            ///
            /// See [`MessageSpec::id`].
            #[inline]
            fn id(&self) -> MessageId {
                MESSAGE_ID
            }

            /// Minimum supported MAVLink version.
            ///
            /// See [`MessageSpec::min_supported_mavlink_version`].
            #[inline]
            fn min_supported_mavlink_version(&self) -> MavLinkVersion {
                MESSAGE_INFO.min_supported_mavlink_version()
            }

            /// Message `CRC_EXTRA` calculated from message XML definition.
            ///
            /// See: [`MessageSpec::crc_extra`].
            #[inline]
            fn crc_extra(&self) -> CrcExtra {
                CRC_EXTRA
            }
        }

        impl MessageSpec for #message_raw_struct_ident {
            /// MAVLink message ID.
            ///
            /// See [`MessageSpec::id`].
            #[inline]
            fn id(&self) -> MessageId {
                MESSAGE_ID
            }

            /// Minimum supported MAVLink version.
            ///
            /// See [`MessageSpec::min_supported_mavlink_version`].
            #[inline]
            fn min_supported_mavlink_version(&self) -> MavLinkVersion {
                MESSAGE_INFO.min_supported_mavlink_version()
            }

            /// Message `CRC_EXTRA` calculated from message XML definition.
            ///
            /// See: [`MessageSpec::crc_extra`].
            #[inline]
            fn crc_extra(&self) -> CrcExtra {
                CRC_EXTRA
            }
        }

        impl MessageImpl for #message_struct_ident {}
        impl MessageImpl for #message_raw_struct_ident {}

        #[allow(clippy::derivable_impls)]
        impl core::default::Default for #message_struct_ident {
            fn default() -> Self {
                Self {
                    #(#message_default_fields)*
                }
            }
        }

        #[allow(clippy::derivable_impls)]
        impl core::default::Default for #message_raw_struct_ident {
            fn default() -> Self {
                #message_struct_ident::default().into()
            }
        }

        impl core::convert::TryFrom<&Payload> for #message_struct_ident {
            type Error = MessageError;

            #[inline]
            fn try_from(value: &Payload) -> Result<Self, Self::Error> {
                Self::try_from_payload(value)
            }
        }

        impl core::convert::TryFrom<&Payload> for #message_raw_struct_ident {
            type Error = MessageError;

            #[inline]
            fn try_from(value: &Payload) -> Result<Self, Self::Error> {
                Self::try_from_payload(value)
            }
        }

        impl IntoPayload for #message_struct_ident {
            #[inline]
            fn encode(
                &self,
                version: MavLinkVersion,
            ) -> Result<Payload, MessageError> {
                self.encode(version)
            }
        }

        impl IntoPayload for #message_raw_struct_ident {
            #[inline]
            fn encode(
                &self,
                version: MavLinkVersion,
            ) -> Result<Payload, MessageError> {
                self.encode(version)
            }
        }

        impl #message_struct_ident {
            #[doc = #message_try_from_payload_doc_comment]
            pub fn try_from_payload(value: &Payload) -> Result<Self, MessageError> {
                match value.version() {
                    MavLinkVersion::V2 => v2::decode(value.bytes()),
                    #message_try_from_payload_arm_v1
                }
            }

            #[doc = #message_encode_doc_comment]
            pub fn encode(
                &self,
                version: MavLinkVersion,
            ) -> Result<Payload, MessageError> {
                Ok(match version {
                    MavLinkVersion::V2 => v2::encode(self)?,
                    #message_encode_arm_v1
                })
            }

            #[doc = #message_to_message_raw_leading_doc_comment]
            pub fn to_raw_message(&self) -> #message_raw_struct_ident {
                #message_raw_struct_ident {
                    #(#message_to_message_raw_field_setters)*
                }
            }
        }

        impl #message_raw_struct_ident {
            #[doc = #message_raw_try_from_payload_doc_comment]
            pub fn try_from_payload(value: &Payload) -> Result<Self, MessageError> {
                match value.version() {
                    MavLinkVersion::V2 => v2::decode_raw(value.bytes()),
                    #message_raw_try_from_payload_arm_v1
                }
            }

            #[doc = #message_raw_encode_doc_comment]
            pub fn encode(
                &self,
                version: MavLinkVersion,
            ) -> Result<Payload, MessageError> {
                Ok(match version {
                    MavLinkVersion::V2 => v2::encode_raw(self)?,
                    #message_raw_encode_arm_v1
                })
            }

            #[doc = #message_raw_to_message_doc_comment]
            pub fn try_to_message(&self) -> Result<#message_struct_ident, MessageError> {
                Ok(#message_struct_ident {
                    #(#message_raw_to_message_field_setters)*
                })
            }
        }

        impl core::convert::TryFrom<#message_raw_struct_ident> for #message_struct_ident {
            type Error = MessageError;

            fn try_from(value: #message_raw_struct_ident) -> Result<Self, Self::Error> {
                value.try_to_message()
            }
        }

        impl core::convert::From<#message_struct_ident> for #message_raw_struct_ident {
            fn from(value: #message_struct_ident) -> Self {
                value.to_raw_message()
            }
        }

        #[doc = #mod_v2_leading_doc_comment]
        ///
        /// See [MAVLink 2](https://mavlink.io/en/guide/mavlink_2.html).
        pub mod v2 {
            use mavspec::rust::spec::{Payload, MavLinkVersion, MessageError};
            use mavspec::rust::spec::tbytes::{TBytesWriterFor, TBytesReader, TBytesReaderFor, TBytesWriter};

            use super::{ #message_struct_ident, #message_raw_struct_ident, MESSAGE_ID };

            #[doc = #payload_size_v2_doc_comment]
            pub const PAYLOAD_SIZE: usize = #payload_v2_size;

            #[doc = #decode_raw_fn_leading_doc_comment]
            ///
            /// If `payload` is less than expected, the remaining elements will be considered to be zeros.
            /// See [MAVLink 2 payload truncation](https://mavlink.io/en/guide/serialization.html#payload_truncation).
            ///
            /// Fields are reordered according to [MAVLink specification](https://mavlink.io/en/guide/serialization.html#field_reordering).
            ///
            /// # Errors
            ///
            /// * Returns [`MessageError::BufferError`] in case of malformed `payload`.
            pub fn decode_raw(payload: &[u8]) -> Result<#message_raw_struct_ident, MessageError> {
                let mut full_payload = [0u8; PAYLOAD_SIZE];
                full_payload[0..payload.len()].copy_from_slice(payload);

                let reader = TBytesReader::from(full_payload.as_slice());

                Ok(#message_raw_struct_ident {
                    #(#v2_decode_raw_field_readers)*
                })
            }

            #[doc = #decode_fn_leading_doc_comment]
            ///
            /// If `payload` is less than expected, the remaining elements will be considered to be zeros.
            /// See [MAVLink 2 payload truncation](https://mavlink.io/en/guide/serialization.html#payload_truncation).
            ///
            /// Fields are reordered according to [MAVLink specification](https://mavlink.io/en/guide/serialization.html#field_reordering).
            ///
            /// # Errors
            ///
            /// * Returns [`MessageError::BufferError`] in case of malformed `payload`.
            /// * Returns [`MessageError::InvalidEnumValue`] if invalid value was provided for MAVLink enum.
            pub fn decode(payload: &[u8]) -> Result<#message_struct_ident, MessageError> {
                decode_raw(payload)?.try_into()
            }

            #[doc = #encode_raw_fn_leading_doc_comment]
            ///
            /// Fields are reordered according to [MAVLink specification](https://mavlink.io/en/guide/serialization.html#field_reordering).
            ///
            /// Zero trailing bytes will be truncated.
            /// See [MAVLink 2 payload truncation](https://mavlink.io/en/guide/serialization.html#payload_truncation).
            ///
            /// # Errors
            ///
            /// This function does not returns errors at the moment. The [`Result`] returning type is
            /// reserved for future implementations where such errors may happen.
            pub fn encode_raw(
                message: &#message_raw_struct_ident
            ) -> Result<Payload, MessageError> {
                let mut buf = [0u8; PAYLOAD_SIZE];
                let mut writer = TBytesWriter::from(buf.as_mut_slice());

                #(#v2_encode_raw_field_writers)*

                let payload = Payload::new(MESSAGE_ID, buf.as_slice(), MavLinkVersion::V2);
                Ok(payload)
            }

            #[doc = #encode_fn_leading_doc_comment]
            ///
            /// Fields are reordered according to [MAVLink specification](https://mavlink.io/en/guide/serialization.html#field_reordering).
            ///
            /// Zero trailing bytes will be truncated.
            /// See [MAVLink 2 payload truncation](https://mavlink.io/en/guide/serialization.html#payload_truncation).
            ///
            /// # Errors
            ///
            /// This function does not returns errors at the moment. The [`Result`] returning type is
            /// reserved for future implementations where such errors may happen.
            pub fn encode(
                message: &#message_struct_ident
            ) -> Result<Payload, MessageError> {
                encode_raw(&message.to_raw_message())
            }
        }

        #v1_encode_decode_mod

        #tests
    })
    .unwrap()
}

fn message_v1_encode_decode_mod(spec: &MessageImplModuleSpec) -> syn::File {
    if !spec.is_v1_compatible() {
        return syn::parse2(quote!()).unwrap();
    }

    let message_struct_ident = spec.ident();
    let message_raw_struct_ident = format_ident!("{}", message_raw_struct_name(spec.name()));

    let mod_v1_leading_doc_comment = format!(" Encoding/decoding for [`{message_struct_ident}`](struct@self::{message_struct_ident}) within `MAVLink 1` protocol.");
    let payload_size_v1_doc_comment = format!(" Message [`{message_struct_ident}`](struct@self::{message_struct_ident}) payload size (non-truncated) according to `MAVLink 2` protocol.");
    let decode_raw_fn_leading_doc_comment =
        format!(" Decodes into [`{message_raw_struct_ident}`] message.");
    let decode_fn_leading_doc_comment = format!(
        " Decodes into [`{message_struct_ident}`](struct@self::{message_struct_ident}) message."
    );
    let encode_raw_fn_leading_doc_comment =
        format!(" Encodes [`{message_raw_struct_ident}`] message into MAVLink [`Payload`].");
    let encode_fn_leading_doc_comment = format!(" Encodes [`{message_struct_ident}`](struct@self::{message_struct_ident}) message into MAVLink [`Payload`].");

    let payload_v1_size = spec.payload_v1_size();

    let v1_decode_raw_field_readers = spec.fields_v1().iter().map(|field| {
        let field_ident = format_ident!("{}", rust_var_name(field.name().into()));
        let reader_fn = format_ident!("{}", t_bytes_read_fn(field.r#type().clone()));

        quote! {
            #field_ident: reader.#reader_fn()?,
        }
    });

    let v1_decode_raw_default_fields = if spec.has_extension_fields() {
        quote! {
            .. core::default::Default::default()
        }
    } else {
        quote!()
    };

    let v1_encode_raw_field_writers = spec.fields_v1().iter().map(|field| {
        let field_ident = format_ident!("{}", rust_var_name(field.name().into()));
        let writer_fn = format_ident!("{}", t_bytes_write_fn(field.r#type().clone()));

        quote! {
            writer.#writer_fn(message.#field_ident)?;
        }
    });

    syn::parse2(quote! {
        #[doc = #mod_v1_leading_doc_comment]
        ///
        /// See [MAVLink versions](https://mavlink.io/en/guide/mavlink_version.html).
        pub mod v1 {
            use mavspec::rust::spec::{Payload, MavLinkVersion, MessageError};
            use mavspec::rust::spec::tbytes::{TBytesWriterFor, TBytesReader, TBytesReaderFor, TBytesWriter};

            use super::{#message_struct_ident, #message_raw_struct_ident, MESSAGE_ID};

            #[doc = #payload_size_v1_doc_comment]
            pub const PAYLOAD_SIZE: usize = #payload_v1_size;

            #[doc = #decode_raw_fn_leading_doc_comment]
            ///
            /// Fields are reordered according to [MAVLink specification](https://mavlink.io/en/guide/serialization.html#field_reordering).
            ///
            /// # Errors
            ///
            /// * Returns [`MessageError::InvalidPayloadSize`] if `payload` has incorrect size.
            ///   Payload size is defined in [`PAYLOAD_SIZE`].
            /// * Returns [`MessageError::BufferError`] in case of malformed `payload`.
            pub fn decode_raw(payload: &[u8]) -> Result<#message_raw_struct_ident, MessageError> {
                if payload.len() != PAYLOAD_SIZE {
                    return Err(MessageError::InvalidPayloadSize {
                        actual: payload.len(),
                        expected: PAYLOAD_SIZE,
                    });
                }
                let reader = TBytesReader::from(payload);

                Ok(#message_raw_struct_ident {
                    #(#v1_decode_raw_field_readers)*
                    #v1_decode_raw_default_fields
                })
            }

            #[doc = #decode_fn_leading_doc_comment]
            ///
            /// Fields are reordered according to [MAVLink specification](https://mavlink.io/en/guide/serialization.html#field_reordering).
            ///
            /// # Errors
            ///
            /// * Returns [`MessageError::InvalidPayloadSize`] if `payload` has incorrect size.
            ///   Payload size is defined in [`PAYLOAD_SIZE`].
            /// * Returns [`MessageError::BufferError`] in case of malformed `payload`.
            /// * Returns [`MessageError::InvalidEnumValue`] if invalid value was provided for MAVLink enum.
            pub fn decode(payload: &[u8]) -> Result<#message_struct_ident, MessageError> {
                decode_raw(payload)?.try_into()
            }

            #[doc = #encode_raw_fn_leading_doc_comment]
            ///
            /// Fields are reordered according to [MAVLink specification](https://mavlink.io/en/guide/serialization.html#field_reordering).
            ///
            /// # Errors
            ///
            /// This function does not returns errors at the moment. The [`Result`] returning type is
            /// reserved for future implementations where such errors may happen.
            pub fn encode_raw(
                message: &#message_raw_struct_ident
            ) -> Result<Payload, MessageError> {
                let mut buf = [0u8; PAYLOAD_SIZE];
                let mut writer = TBytesWriter::from(buf.as_mut_slice());

                #(#v1_encode_raw_field_writers)*

                let payload = Payload::new(MESSAGE_ID, buf.as_slice(), MavLinkVersion::V1);
                Ok(payload)
            }

            #[doc = #encode_fn_leading_doc_comment]
            ///
            /// Fields are reordered according to [MAVLink specification](https://mavlink.io/en/guide/serialization.html#field_reordering).
            ///
            /// # Errors
            ///
            /// This function does not returns errors at the moment. The [`Result`] returning type is
            /// reserved for future implementations where such errors may happen.
            pub fn encode(
                message: &#message_struct_ident
            ) -> Result<Payload, MessageError> {
                encode_raw(&message.to_raw_message())
            }
        }
    })
        .unwrap()
}

fn message_to_message_raw_field_setter(field: &FieldSpec) -> proc_macro2::TokenStream {
    let field_ident = format_ident!("{}", rust_var_name(field.name().into()));
    let field_rust_base_type: syn::Type =
        syn::parse_str(field.r#type().base_type().rust_type().as_str()).unwrap();

    if field.is_enum() {
        if field.is_bitmask() {
            if field.is_array() {
                let cast_to_base_type = if field.cast_enum() {
                    quote! { as #field_rust_base_type }
                } else {
                    quote!()
                };
                quote! {
                    #field_ident: self.#field_ident.map(|i| i.bits() #cast_to_base_type),
                }
            } else {
                let cast_to_rust_type = if field.cast_enum() {
                    quote! { as #field_rust_base_type }
                } else {
                    quote!()
                };
                quote! {
                    #field_ident: self.#field_ident.bits() #cast_to_rust_type,
                }
            }
        } else if field.is_array() {
            quote! {
                #field_ident: self.#field_ident.map(|i| i as #field_rust_base_type),
            }
        } else {
            quote! {
                #field_ident: self.#field_ident as #field_rust_base_type,
            }
        }
    } else {
        quote! {
            #field_ident: self.#field_ident,
        }
    }
}

fn message_raw_to_message_field_setter(field: &FieldSpec) -> proc_macro2::TokenStream {
    let field_ident = format_ident!("{}", rust_var_name(field.name().into()));

    if field.is_enum() {
        let enum_ident = format_ident!("{}", enum_rust_name(field.enum_name().into()));
        let enum_base_type: syn::Type =
            syn::parse_str(field.enum_type().base_type().rust_type().as_str()).unwrap();
        let cast_to_enum_base_type = if field.cast_enum() {
            quote! { as #enum_base_type }
        } else {
            quote!()
        };
        let array_length = field.array_length();

        if field.is_bitmask() {
            if field.is_array() {
                quote! {
                    #field_ident: {
                        use super::super::enums::#enum_ident as Enum;
                        let mut values: [Enum; #array_length] = [Enum::default(); #array_length];
                        #[allow(clippy::needless_range_loop)]
                        for i in 0 .. #array_length {
                            values[i] = Enum::from_bits_retain(
                                self.#field_ident[i] #cast_to_enum_base_type,
                            );
                        }
                        values
                    },
                }
            } else {
                quote! {
                    #field_ident: super::super::enums::#enum_ident::from_bits_retain(
                        self.#field_ident #cast_to_enum_base_type
                    ),
                }
            }
        } else if field.is_array() {
            quote! {
                #field_ident: {
                    use super::super::enums::#enum_ident as Enum;
                    let mut values: [Enum; #array_length] = [Enum::default(); #array_length];
                    #[allow(clippy::needless_range_loop)]
                    for i in 0 .. #array_length {
                        values[i] = Enum::try_from_discriminant(
                            self.#field_ident[i] #cast_to_enum_base_type,
                        )?;
                    }
                    values
                },
            }
        } else {
            quote! {
                #field_ident: super::super::enums::#enum_ident::try_from_discriminant(
                    self.#field_ident #cast_to_enum_base_type
                )?,
            }
        }
    } else {
        quote! {
            #field_ident: self.#field_ident,
        }
    }
}

fn generate_tests(spec: &MessageImplModuleSpec) -> proc_macro2::TokenStream {
    if !spec.params().generate_tests {
        return quote!();
    }

    let message_struct_ident = spec.ident();

    let v2_tests = quote! {
        #[test]
        fn basic_v2() {
            let message = #message_struct_ident::default();
            let encoded_payload = v2::encode(&message).unwrap();
            let decoded_message = v2::decode(encoded_payload.bytes()).unwrap();
            let encoded_versioned_payload = message.encode(MavLinkVersion::V2).unwrap();

            assert_eq!(decoded_message.id(), message.id());

            assert_eq!(encoded_payload.id(), message.id());
            assert!(matches!(encoded_payload.version(), MavLinkVersion::V2));

            assert_eq!(encoded_versioned_payload.id(), message.id());
            assert!(matches!(encoded_versioned_payload.version(), MavLinkVersion::V2));
        }
    };

    let v1_tests = if spec.is_v1_compatible() {
        quote! {
            #[test]
            fn basic_v1() {
                let message = #message_struct_ident::default();
                let encoded_payload = v1::encode(&message).unwrap();
                let decoded_message = v1::decode(encoded_payload.bytes()).unwrap();
                let encoded_versioned_payload = message.encode(MavLinkVersion::V1).unwrap();

                assert_eq!(decoded_message.id(), message.id());

                assert_eq!(encoded_payload.id(), message.id());
                assert!(matches!(encoded_payload.version(), MavLinkVersion::V1));

                assert_eq!(encoded_versioned_payload.id(), message.id());
                assert!(matches!(encoded_versioned_payload.version(), MavLinkVersion::V1));
            }
        }
    } else {
        quote!()
    };

    quote! {
        #[cfg(test)]
        mod tests {
            use super::*;

            #v2_tests
            #v1_tests
        }
    }
}

/// Inherited message module template.
pub(crate) fn inherited_message_module(spec: &MessageInheritedModuleSpec) -> syn::File {
    let dialect_mod_name = dialect_mod_name(spec.dialect_name().into());

    let module_doc_comment = format!(
        " MAVLink message `{}` inherited from [`super::super::super::{}`] dialect.",
        spec.message_name(),
        &dialect_mod_name
    );

    let dialect_mod_ident = format_ident!("{}", &dialect_mod_name);
    let message_mod_ident = format_ident!("{}", message_mod_name(spec.message_name().into()));

    let message_struct_ident = format_ident!("{}", message_struct_name(spec.message_name()));
    let message_raw_struct_ident =
        format_ident!("{}", message_raw_struct_name(spec.message_name()));

    let message_doc_comment = format!(" Originally defined in [`{dialect_mod_name}::messages::{message_mod_ident}`](dialect::messages::{message_struct_ident}).");
    let message_raw_doc_comment = format!(" Originally defined in [`{dialect_mod_name}::messages::{message_mod_ident}`](dialect::messages::{message_mod_ident}::{message_raw_struct_ident}).");
    let reexported_from_dialect_doc_comment =
        format!(" Re-exported from [`{dialect_mod_name}`](dialect) dialect.");

    let v1_reexports = if spec.is_v1_compatible() {
        quote! {
            #[doc = #reexported_from_dialect_doc_comment]
            pub use dialect::messages::#message_mod_ident::v1;
        }
    } else {
        quote!()
    };

    syn::parse2(quote! {
        #![doc = #module_doc_comment]

        use mavspec::rust::spec::MessageInfo;

        use super::super::super::#dialect_mod_ident as dialect;

        /// Message ID
        pub(crate) const MESSAGE_ID: u32 = dialect::messages::#message_mod_ident::MESSAGE_ID;
        /// Message info
        pub(crate) const MESSAGE_INFO: MessageInfo =
            dialect::messages::#message_mod_ident::MESSAGE_INFO;

        #[doc = #message_doc_comment]
        pub type #message_struct_ident = dialect::messages::#message_struct_ident;

        #[doc = #message_raw_doc_comment]
        pub type #message_raw_struct_ident = dialect::messages::#message_mod_ident::#message_raw_struct_ident;

        #[doc = #reexported_from_dialect_doc_comment]
        pub use dialect::messages::#message_mod_ident::spec;

        #[doc = #reexported_from_dialect_doc_comment]
        pub use dialect::messages::#message_mod_ident::v2;

        #v1_reexports
    })
    .unwrap()
}
