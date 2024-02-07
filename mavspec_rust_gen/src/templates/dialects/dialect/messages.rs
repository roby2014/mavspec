use quote::{format_ident, quote};

use crate::conventions::{
    dialect_mod_name, enum_rust_name, message_mod_name, message_struct_name, rust_var_name,
};
use crate::specs::dialects::dialect::messages::{
    MessageImplModuleSpec, MessageInheritedModuleSpec, MessagesRootModuleSpec,
};
use crate::specs::Spec;
use crate::templates::helpers::make_serde_derive_annotation;

/// Messages root module template.
pub(crate) fn messages_root_module(spec: &MessagesRootModuleSpec) -> syn::File {
    let module_doc_comment = format!(" MAVLink messages of `{}` dialect.", spec.dialect_name());

    let message_modules_and_imports = spec.messages().iter().map(|msg| {
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
        let serde_arrays_attr = if field.requires_serde_arrays() {
            quote! {
                #[cfg_attr(feature = "serde", serde(with = "serde_arrays"))]
            }
        } else {
            quote!()
        };
        let field_ident = format_ident!("{}", rust_var_name(field.name().into()));
        let field_rust_type: syn::Type =
            syn::parse_str(field.r#type().rust_type().as_str()).unwrap();
        let enum_ident = format_ident!("{}", enum_rust_name(field.enum_name().into()));
        let array_length = field.array_length();

        let field_base_type: syn::Type =
            syn::parse_str(field.r#type().base_type().rust_type().as_str()).unwrap();
        let base_type_attr = if field.is_enum() {
            quote! { #[base_type(#field_base_type)] }
        } else {
            quote!()
        };
        let enum_type: syn::Type = syn::parse_str(field.enum_type().rust_type().as_str()).unwrap();
        let repr_type_attr = if field.requires_enum_casting() {
            quote! { #[repr_type(#enum_type)] }
        } else {
            quote!()
        };
        let bitmask_attr = if field.is_bitmask() {
            quote! { #[bitmask] }
        } else {
            quote!()
        };

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
            #bitmask_attr
            #base_type_attr
            #repr_type_attr
            #field_definition
        }
    });

    let tests = generate_tests(spec);

    syn::parse2(quote! {
        #![doc = #module_doc_comment]

        use mavspec::rust::spec::{MessageInfo, MessageSpec};
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
        /// Message encoding/decoding are provided by implementing [`core::convert::TryFrom<Payload>`] for
        #[doc = #message_encode_decode_doc_comment]
        /// These traits are implemented by [`Message`](mavspec::rust::derive::Message) proc macro.
        #[derive(mavspec::rust::derive::Message)]
        #[derive(core::clone::Clone, core::fmt::Debug)]
        #derive_serde
        #[message_id(#message_id)]
        #[crc_extra(#crc_extra)]
        pub struct #message_struct_ident {
            #(#message_fields)*
        }

        impl core::convert::From<#message_struct_ident> for super::super::Message {
            fn from(value: #message_struct_ident) -> Self {
                Self::#message_struct_ident(value)
            }
        }

        #tests
    })
    .unwrap()
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
            let encoded_payload = message.encode(MavLinkVersion::V2).unwrap();
            let decoded_message = #message_struct_ident::try_from(&encoded_payload).unwrap();

            assert_eq!(decoded_message.id(), message.id());

            assert_eq!(encoded_payload.id(), message.id());
            assert!(matches!(encoded_payload.version(), MavLinkVersion::V2));
        }
    };

    let v1_tests = if spec.is_v1_compatible() {
        quote! {
            #[test]
            fn basic_v1() {
                let message = #message_struct_ident::default();
                let encoded_payload = message.encode(MavLinkVersion::V1).unwrap();
                let decoded_message = #message_struct_ident::try_from(&encoded_payload).unwrap();

                assert_eq!(decoded_message.id(), message.id());

                assert_eq!(encoded_payload.id(), message.id());
                assert!(matches!(encoded_payload.version(), MavLinkVersion::V1));
            }
        }
    } else {
        quote!()
    };

    quote! {
        #[cfg(test)]
        mod tests {
            use mavspec::rust::spec::{IntoPayload, MavLinkVersion};

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

    let message_doc_comment = format!(" Originally defined in [`{dialect_mod_name}::messages::{message_mod_ident}`](dialect::messages::{message_struct_ident}).");
    let reexported_from_dialect_doc_comment =
        format!(" Re-exported from [`{dialect_mod_name}`](dialect) dialect.");

    syn::parse2(quote! {
        #![doc = #module_doc_comment]

        use mavspec::rust::spec::MessageInfo;
        use mavspec::rust::spec::types::{CrcExtra, MessageId};

        use super::super::super::#dialect_mod_ident as dialect;

        /// Message ID.
        pub(crate) const MESSAGE_ID: MessageId = dialect::messages::#message_mod_ident::MESSAGE_ID;
        /// `CRC_EXTRA` calculated from message XML definition.
        pub(crate) const CRC_EXTRA: CrcExtra = dialect::messages::#message_mod_ident::CRC_EXTRA;
        /// Generic message info that contains all message metadata.
        pub(crate) const MESSAGE_INFO: MessageInfo =
            dialect::messages::#message_mod_ident::MESSAGE_INFO;

        #[doc = #message_doc_comment]
        pub type #message_struct_ident = dialect::messages::#message_struct_ident;

        #[doc = #reexported_from_dialect_doc_comment]
        pub use dialect::messages::#message_mod_ident::spec;
    })
    .unwrap()
}
