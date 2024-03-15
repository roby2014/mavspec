use crate::conventions::{
    dialect_enum_name, message_mod_name, message_struct_name, messages_enum_entry_name,
};
use quote::{format_ident, quote};

use crate::specs::dialects::dialect::DialectModuleSpec;
use crate::specs::Spec;

pub fn dialect_module(specs: &DialectModuleSpec) -> syn::File {
    let leading_module_comment = format!("# MAVLink dialect `{}`", specs.name());
    let dialect_name_quoted = specs.name();

    let dialect_id = match specs.dialect_id() {
        None => quote! { None },
        Some(id) => quote! { Some(#id) },
    };

    let dialect_version = match specs.version() {
        None => quote! { None },
        Some(version) => quote! { Some(#version) },
    };

    let messages_enum_comment = format!(
        " Enum containing all messages within `{}` dialect.",
        specs.name()
    );

    let message_spec_const_ident = format_ident!("__MAVSPEC__MESSAGES");
    let messages_count = specs.messages().len();

    let messages_specs = specs.messages().iter().map(|msg| {
        let msg_id = msg.id();
        let crc_extra = msg.crc_extra();
        quote! {
            MessageInfo::new(#msg_id, #crc_extra)
        }
    });

    let messages_variants = specs.messages().iter().map(|msg| {
        let comment = format!(" MAVLink message `{}`.", msg.name());
        let messages_enum_entry_name = format_ident!("{}", messages_enum_entry_name(msg.name()));
        let message_struct_name = format_ident!("{}", message_struct_name(msg.name()));

        quote! {
            #[doc = #comment]
            #messages_enum_entry_name(messages::#message_struct_name),
        }
    });

    let message_info_arms = specs.messages().iter().map(|msg| {
        let message_mod_name = format_ident!("{}", message_mod_name(msg.name()));

        quote! {
            messages::#message_mod_name::MESSAGE_ID => &messages::#message_mod_name::MESSAGE_INFO,
        }
    });

    let dialect_enum_ident = format_ident!("{}", dialect_enum_name(specs.name()));

    let message_spec_id_arms = specs.messages().iter().map(|msg| {
        let message_mod_name = format_ident!("{}", message_mod_name(msg.name()));
        let messages_enum_entry_name = format_ident!("{}", messages_enum_entry_name(msg.name()));

        quote! {
            #dialect_enum_ident::#messages_enum_entry_name(_) => messages::#message_mod_name::MESSAGE_ID,
        }
    });

    let message_spec_msmv_arms = specs.messages().iter().map(|msg| {
        let message_mod_name = format_ident!("{}", message_mod_name(msg.name()));
        let messages_enum_entry_name = format_ident!("{}", messages_enum_entry_name(msg.name()));

        quote! {
            #dialect_enum_ident::#messages_enum_entry_name(_) =>
                messages::#message_mod_name::MESSAGE_INFO.min_supported_mavlink_version(),
        }
    });

    let message_spec_crc_extra_arms = specs.messages().iter().map(|msg| {
        let message_mod_name = format_ident!("{}", message_mod_name(msg.name()));
        let messages_enum_entry_name = format_ident!("{}", messages_enum_entry_name(msg.name()));

        quote! {
            #dialect_enum_ident::#messages_enum_entry_name(_) => messages::#message_mod_name::CRC_EXTRA,
        }
    });

    let decode_arms = specs.messages().iter().map(|msg| {
        let message_mod_name = format_ident!("{}", message_mod_name(msg.name()));
        let messages_enum_entry_name = format_ident!("{}", messages_enum_entry_name(msg.name()));
        let message_struct_name = format_ident!("{}", message_struct_name(msg.name()));

        quote! {
            messages::#message_mod_name::MESSAGE_ID => {
                #dialect_enum_ident::#messages_enum_entry_name(
                    messages::#message_struct_name::try_from(payload)?
                )
            }
        }
    });

    let encode_arms = specs.messages().iter().map(|msg| {
        let messages_enum_entry_name = format_ident!("{}", messages_enum_entry_name(msg.name()));

        quote! {
            #dialect_enum_ident::#messages_enum_entry_name(message) => message.encode(version)?,
        }
    });

    let tests = if specs.params().generate_tests {
        let ids = specs.messages().iter().map(|msg| {
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
                        let msg_info = #dialect_enum_ident::message_info(id);
                        assert!(msg_info.is_ok());
                        assert_eq!(msg_info.unwrap().id(), id);
                    }
                }
            }
        }
    } else {
        quote!()
    };

    let allow_unreachable = quote! {
        #[allow(unreachable_patterns)]
        #[allow(unreachable_code)]
    };

    syn::parse2(quote! {
        #![doc = #leading_module_comment]

        use mavspec::rust::spec::{
            Dialect, DialectSpec, MessageInfo, IntoPayload, Payload, MessageSpec,
            MavLinkVersion, SpecError,
        };
        use mavspec::rust::spec::types::{CrcExtra, MessageId, DialectId, DialectVersion};

        // MAVLink messages.
        pub mod messages;
        // MAVLink enums.
        pub mod enums;

        const #message_spec_const_ident: [MessageInfo; #messages_count] = [#(#messages_specs,)*];

        #[doc = #messages_enum_comment]
        #[derive(core::clone::Clone, core::fmt::Debug)]
        // {{#if params.serde}}#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]{{/if}}
        #[allow(clippy::large_enum_variant)]
        pub enum #dialect_enum_ident {
            #(#messages_variants)*
        }

        impl Dialect for #dialect_enum_ident {
            /// Dialect name as it appears in XML definition.
            #[inline]
            fn name() -> &'static str {
                #dialect_name_quoted
            }

            /// Returns `dialect` identifier as specified in MAVLink [XML definitions](https://mavlink.io/en/guide/xml_schema.html).
            #[inline]
            fn dialect() -> Option<DialectId> {
                #dialect_id
            }

            /// Minor dialect `version` as specified in MAVLink [XML definitions](https://mavlink.io/en/guide/xml_schema.html).
            #[inline]
            fn version() -> Option<DialectVersion> {
                #dialect_version
            }

            /// Message specification by `id`.
            fn message_info(id: MessageId) -> Result<&'static dyn MessageSpec, SpecError> {
                #allow_unreachable
                Ok(match id {
                    #(#message_info_arms)*
                    _ => return Err(SpecError::NotInDialect(id)),
                })
            }

            /// Decodes message from payload.
            fn decode(payload: &Payload) -> Result<Self, SpecError> {
                #allow_unreachable
                Ok(match payload.id() {
                    #(#decode_arms)*
                    id => return Err(SpecError::NotInDialect(id)),
                })
            }

            /// Dialect specification.
            fn spec() -> DialectSpec {
                DialectSpec::new(
                    #dialect_name_quoted,
                    #dialect_id,
                    #dialect_version,
                    &#message_spec_const_ident
                )
            }
        }

        impl core::convert::TryFrom<&Payload> for #dialect_enum_ident {
            type Error = SpecError;

            /// Decodes message from MAVLink payload.
            fn try_from(value: &Payload) -> Result<Self, Self::Error> {
                Self::decode(value)
            }
        }

        impl IntoPayload for #dialect_enum_ident {
            /// Encodes message into MAVLink payload.
            fn encode(
                &self,
                version: MavLinkVersion,
            ) -> Result<Payload, SpecError> {
                Ok(match self {
                    #(#encode_arms)*
                })
            }
        }

        impl MessageSpec for #dialect_enum_ident {
            /// MAVLink message ID.
            ///
            /// See [`MessageSpec::id`] for details.
            fn id(&self) -> MessageId {
                match self {
                    #(#message_spec_id_arms)*
                }
            }

            /// Minimum supported MAVLink protocol version.
            ///
            /// See [`MessageSpec::min_supported_mavlink_version`] for details.
            fn min_supported_mavlink_version(&self) -> MavLinkVersion {
                match self {
                    #(#message_spec_msmv_arms)*
                }
            }

            /// Message `EXTRA_CRC` calculated from message XML definition.
            ///
            /// See [`MessageSpec::crc_extra`] for details.
            fn crc_extra(&self) -> CrcExtra {
                match self {
                    #(#message_spec_crc_extra_arms)*
                }
            }
        }

        #tests
    })
    .unwrap()
}
