use crate::conventions::{
    message_mod_name, message_raw_struct_name, message_struct_name, messages_enum_entry_name,
};
use quote::{format_ident, quote};

use crate::specs::dialects::dialect::DialectModuleSpec;
use crate::specs::Spec;

pub fn dialect_module(specs: &DialectModuleSpec) -> syn::File {
    let leading_module_comment = format!("# MAVLink dialect `{}`", specs.name());
    let dialect_name_quoted = format!("`\"{}\"", specs.name());
    let dialect_id = match specs.dialect_id() {
        None => quote! { None },
        Some(id) => quote! { Some(#id) },
    };
    let dialect_version = match specs.version() {
        None => quote! { None },
        Some(version) => quote! { Some(#version) },
    };
    let messages_enum_comment = format!(
        "Enum containing all messages within `{}` dialect.",
        specs.name()
    );
    let messages_variants = specs.messages().values().map(|msg| {
        let comment = format!("MAVLink message `{}`.", msg.name());
        let messages_enum_entry_name =
            format_ident!("{}", messages_enum_entry_name(msg.name().into()));
        let message_struct_name = format_ident!("{}", message_struct_name(msg.name().into()));

        quote! {
            #[doc = #comment]
            #messages_enum_entry_name(messages::#message_struct_name),
        }
    });
    let messages_raw_enum_comment = format!(
        "Enum containing all raw messages within `{}` dialect.",
        specs.name()
    );
    let messages_raw_variants = specs.messages().values().map(|msg| {
        let comment = format!("Raw MAVLink message `{}`.", msg.name());
        let messages_enum_entry_name =
            format_ident!("{}", messages_enum_entry_name(msg.name().into()));
        let message_mod_name = format_ident!("{}", message_mod_name(msg.name().into()));
        let message_raw_struct_name =
            format_ident!("{}", message_raw_struct_name(msg.name().into()));

        quote! {
            #[doc = #comment]
            #messages_enum_entry_name(messages::#message_mod_name::#message_raw_struct_name),
        }
    });
    let message_info_arms = specs.messages().values().map(|msg| {
        let message_mod_name = format_ident!("{}", message_mod_name(msg.name().into()));

        quote! {
            messages::#message_mod_name::MESSAGE_ID => &messages::#message_mod_name::MESSAGE_INFO,
        }
    });
    let decode_arms = specs.messages().values().map(|msg| {
        let message_mod_name = format_ident!("{}", message_mod_name(msg.name().into()));
        let messages_enum_entry_name =
            format_ident!("{}", messages_enum_entry_name(msg.name().into()));
        let message_struct_name = format_ident!("{}", message_struct_name(msg.name().into()));

        quote! {
            messages::#message_mod_name::MESSAGE_ID => {
                Message::#messages_enum_entry_name(
                    messages::#message_struct_name::try_from(payload)?
                )
            }
        }
    });
    let decode_raw_arms = specs.messages().values().map(|msg| {
        let message_mod_name = format_ident!("{}", message_mod_name(msg.name().into()));
        let messages_enum_entry_name =
            format_ident!("{}", messages_enum_entry_name(msg.name().into()));
        let message_raw_struct_name =
            format_ident!("{}", message_raw_struct_name(msg.name().into()));

        quote! {
            messages::#message_mod_name::MESSAGE_ID => {
                MessageRaw::#messages_enum_entry_name(
                    messages::#message_mod_name::#message_raw_struct_name::try_from(payload)?
                )
            }
        }
    });
    let encode_arms = specs.messages().values().map(|msg| {
        let messages_enum_entry_name =
            format_ident!("{}", messages_enum_entry_name(msg.name().into()));

        quote! {
            Message::#messages_enum_entry_name(message) => message.encode(version)?,
        }
    });
    let encode_raw_arms = specs.messages().values().map(|msg| {
        let messages_enum_entry_name =
            format_ident!("{}", messages_enum_entry_name(msg.name().into()));

        quote! {
            MessageRaw::#messages_enum_entry_name(raw_message) => raw_message.encode(version)?,
        }
    });
    let tests = if specs.params().generate_tests {
        let ids = specs.messages().values().map(|msg| {
            let id = msg.id();
            quote! {
                #id
            }
        });

        quote! {
            #[cfg(test)]
            mod tests {
                use super::*;

                #[test]
                fn retrieve_message_info() {
                    for id in [
                        #(#ids,)*
                    ] {
                        let msg_info = spec().message_info(id);
                        assert!(msg_info.is_ok());
                        assert_eq!(msg_info.unwrap().id(), id);
                    }
                }
            }
        }
    } else {
        quote!()
    };

    syn::parse2(quote! {
        #![doc = #leading_module_comment]

        use mavspec::rust::spec::{
            IntoPayload, DialectSpec, Payload, MessageSpec,
            MavLinkVersion, MessageError,
        };
        use mavspec::rust::spec::types::{MessageId, DialectId, DialectVersion};

        // MAVLink messages.
        pub mod messages;
        // MAVLink enums.
        pub mod enums;

        /// Dialect name as it appears in XML definition.
        ///
        /// See [`DialectModuleSpec::name`].
        const NAME: &str = #dialect_name_quoted;
        /// Dialect id as it appears in XML definition.
        ///
        /// See [`DialectModuleSpec::dialect`].
        const ID: Option<DialectId> = #dialect_id;
        /// Dialect version as it appears in XML definition.
        ///
        /// See [`DialectModuleSpec::dialect`].
        const VERSION: Option<DialectVersion> = #dialect_version;
        /// [`Dialect`] specification.
        ///
        /// See: [`DialectModuleSpec`].
        const SPEC: Dialect = Dialect {};

        /// Dialect specification.
        ///
        /// This struct can't be instantiated directly. The constant (and the only) instance is accessible
        /// through [`spec`] function.
        #[derive(core::clone::Clone, core::fmt::Debug, Default)]
        struct Dialect;

        impl Dialect {
            /// Dialect name as it appears in XML definition.
            ///
            /// See [`DialectModuleSpec::name`].
            #[inline]
            pub fn name() -> &'static str {
                NAME
            }

            /// Returns `dialect` identifier as specified in MAVLink [XML definitions](https://mavlink.io/en/guide/xml_schema.html).
            ///
            /// See [`DialectModuleSpec::dialect`].
            #[inline]
            fn dialect() -> Option<DialectId> {
                ID
            }

            /// Minor dialect `version` as specified in MAVLink [XML definitions](https://mavlink.io/en/guide/xml_schema.html).
            ///
            /// See [`DialectModuleSpec::version`].
            #[inline]
            fn version() -> Option<DialectVersion> {
                VERSION
            }

            /// Message specification by `id`.
            ///
            /// See [`DialectModuleSpec::message_info`].
            #[inline]
            pub fn message_info(id: MessageId) -> Result<&'static dyn MessageSpec, MessageError> {
                message_info(id)
            }
        }

        impl DialectSpec for Dialect {
            /// Message specification by `id`.
            ///
            /// See [`DialectModuleSpec::name`].
            #[inline]
            fn name(&self) -> &str {
                Self::name()
            }

            /// Returns `dialect` identifier as specified in MAVLink [XML definitions](https://mavlink.io/en/guide/xml_schema.html).
            ///
            /// See [`DialectModuleSpec::dialect`].
            #[inline]
            fn dialect(&self) -> Option<DialectId> {
                Self::dialect()
            }

            /// Minor dialect `version` as specified in MAVLink [XML definitions](https://mavlink.io/en/guide/xml_schema.html).
            ///
            /// See [`DialectModuleSpec::version`].
            #[inline]
            fn version(&self) -> Option<DialectVersion> {
                Self::version()
            }

            /// Message specification by `id`.
            ///
            /// See [`DialectModuleSpec::message_info`].
            #[inline]
            fn message_info(&self, id: MessageId) -> Result<&dyn MessageSpec, MessageError> {
                Self::message_info(id)
            }
        }

        #[doc = #messages_enum_comment]
        #[derive(core::clone::Clone, core::fmt::Debug)]
        // {{#if params.serde}}#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]{{/if}}
        #[allow(clippy::large_enum_variant)]
        pub enum Message {
            #(#messages_variants)*
        }

        #[doc = #messages_raw_enum_comment]
        #[derive(core::clone::Clone, core::fmt::Debug)]
        // {{#if params.serde}}#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]{{/if}}
        #[allow(clippy::large_enum_variant)]
        pub enum MessageRaw {
            #(#messages_raw_variants)*
        }

        impl core::convert::TryFrom<&Payload> for Message {
            type Error = MessageError;

            /// Decodes message from MAVLink payload.
            fn try_from(value: &Payload) -> Result<Self, Self::Error> {
                Self::decode(value)
            }
        }

        impl core::convert::TryFrom<&Payload> for MessageRaw {
            type Error = MessageError;

            /// Decodes message from MAVLink payload.
            fn try_from(value: &Payload) -> Result<Self, Self::Error> {
                Self::decode(value)
            }
        }

        impl IntoPayload for Message {
            /// Encodes message into MAVLink payload.
            fn encode(
                &self,
                version: MavLinkVersion,
            ) -> Result<Payload, MessageError> {
                self.encode(version)
            }
        }

        impl IntoPayload for MessageRaw {
            /// Encodes raw message into MAVLink payload.
            fn encode(
                &self,
                version: MavLinkVersion,
            ) -> Result<Payload, MessageError> {
                self.encode(version)
            }
        }

        impl Message {
            /// Decodes message from MAVLink payload.
            pub fn decode(
                payload: &Payload,
            ) -> Result<Self, MessageError> {
                decode(payload)
            }

            /// Encodes message to MAVLink payload.
            pub fn encode(&self, version: MavLinkVersion) -> Result<Payload, MessageError> {
                encode(self, version)
            }
        }

        impl MessageRaw {
            /// Decodes raw message from MAVLink payload.
            pub fn decode(
                payload: &Payload,
            ) -> Result<Self, MessageError> {
                decode_raw(payload)
            }

            /// Encodes raw message to MAVLink payload.
            pub fn encode(&self, version: MavLinkVersion) -> Result<Payload, MessageError> {
                encode_raw(self, version)
            }
        }

        /// Dialect specification.
        ///
        /// Returns the current dialect specification as [`DialectSpec`] trait object.
        #[inline]
        pub const fn spec() -> &'static dyn DialectSpec {
            &SPEC
        }

        /// Retrieve message specification by its `id`.
        ///
        /// See [`DialectSpec::message_info`].
        pub fn message_info(id: MessageId) -> Result<&'static dyn MessageSpec, MessageError> {
            Ok(match id {
                #(#message_info_arms)*
                _ => return Err(MessageError::UnsupportedMessageId(id)),
            })
        }

        /// Decodes [`Message`] from [`Payload`].
        pub fn decode(payload: &Payload) -> Result<Message, MessageError> {
            Ok(match payload.id() {
                #(#decode_arms)*
                id => return Err(MessageError::UnsupportedMessageId(id)),
            })
        }

        /// Decodes [`MessageRaw`] from [`Payload`].
        pub fn decode_raw(payload: &Payload) -> Result<MessageRaw, MessageError> {
            Ok(match payload.id() {
                #(#decode_raw_arms)*
                id => return Err(MessageError::UnsupportedMessageId(id)),
            })
        }

        /// Encodes [`Message`] into [`Payload`].
        pub fn encode(msg: &Message, version: MavLinkVersion) -> Result<Payload, MessageError> {
            Ok(match msg {
                #(#encode_arms)*
            })
        }

        /// Encodes [`MessageRaw`] into [`Payload`].
        pub fn encode_raw(msg: &MessageRaw, version: MavLinkVersion) -> Result<Payload, MessageError> {
            Ok(match msg {
                #(#encode_raw_arms)*
            })
        }

        #tests
    })
    .unwrap()
}
