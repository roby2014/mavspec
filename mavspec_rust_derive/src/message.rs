use std::cmp::Ordering;

use quote::{format_ident, quote, TokenStreamExt};

use crate::errors::{Error, SpecError};
use crate::field_types::FieldType;
use crate::message_attributes::{CrcExtra, MessageId};
use crate::message_field::Field;

pub(crate) struct Message {
    ident: syn::Ident,
    ordered_fields: Vec<Field>,
    message_id: MessageId,
    crc_extra: CrcExtra,
}

enum PayloadType {
    Strict,
    Truncated,
}

impl TryFrom<syn::DeriveInput> for Message {
    type Error = Error;

    fn try_from(value: syn::DeriveInput) -> Result<Self, Self::Error> {
        Self::try_from_derive_input(value)
    }
}

impl Message {
    pub(crate) fn try_from_derive_input(value: syn::DeriveInput) -> Result<Self, Error> {
        let mut fields: Vec<Field> = Vec::new();

        match &value.data {
            syn::Data::Struct(data) => {
                for field in &data.fields {
                    fields.push(Field::try_from(field.clone())?);
                }
            }
            _ => return Err(SpecError::NotAStruct.into()),
        }

        let mut ordered_fields = fields;
        Self::reorder_fields(&mut ordered_fields);

        Ok(Self {
            ident: value.ident,
            ordered_fields,
            message_id: MessageId::try_from(&value.attrs)?,
            crc_extra: CrcExtra::try_from(&value.attrs)?,
        })
    }

    pub(crate) fn to_token_stream(&self) -> proc_macro2::TokenStream {
        let impl_message_spec = self.impl_message_spec();
        let impl_try_from_payload = self.impl_try_from_payload();
        let impl_into_payload = self.impl_into_payload();
        let impl_default = self.impl_default();
        let impl_message_impl = self.impl_message_impl();

        quote! {
            #impl_message_spec
            #impl_try_from_payload
            #impl_into_payload
            #impl_message_impl
            #impl_default
        }
    }

    fn ident(&self) -> &syn::Ident {
        &self.ident
    }

    fn fields_v1(&self) -> impl Iterator<Item = &Field> {
        self.ordered_fields
            .iter()
            .filter(|field| !field.is_extension())
    }

    fn fields_v2(&self) -> impl Iterator<Item = &Field> {
        self.ordered_fields.iter()
    }

    fn fields_ext(&self) -> impl Iterator<Item = &Field> {
        self.ordered_fields
            .iter()
            .filter(|field| field.is_extension())
    }

    fn impl_message_spec(&self) -> proc_macro2::TokenStream {
        let ident = self.ident();
        let min_supported_mavlink_version = self.message_id().min_supported_mavlink_version();
        let message_id = self.message_id().literal();
        let crc_extra = self.crc_extra().literal();

        quote! {
            impl mavspec::rust::spec::MessageSpec for #ident {
                #[inline]
                fn id(&self) -> mavspec::rust::spec::types::MessageId {
                    #message_id
                }

                #[inline]
                fn min_supported_mavlink_version(&self) -> mavspec::rust::spec::MavLinkVersion {
                    #min_supported_mavlink_version
                }

                #[inline]
                fn crc_extra(&self) -> mavspec::rust::spec::types::CrcExtra {
                    #crc_extra
                }
            }
        }
    }

    fn impl_try_from_payload(&self) -> proc_macro2::TokenStream {
        let ident = self.ident();
        let decode_fn_v1 = self.decode_fn_v1();
        let decode_fn_v2 = self.decode_fn_v2();

        quote! {
            impl TryFrom<&mavspec::rust::spec::Payload> for #ident {
                type Error = mavspec::rust::spec::SpecError;

                fn try_from(value: &mavspec::rust::spec::Payload) -> Result<Self, Self::Error> {
                    use mavspec::rust::spec::tbytes::{TBytesReader, TBytesReaderFor};

                    #decode_fn_v1
                    #decode_fn_v2

                    match value.version() {
                        mavspec::rust::spec::MavLinkVersion::V1 => decode_v1(value.bytes()),
                        mavspec::rust::spec::MavLinkVersion::V2 => decode_v2(value.bytes()),
                    }
                }
            }
        }
    }

    fn impl_into_payload(&self) -> proc_macro2::TokenStream {
        let ident = self.ident();
        let encode_fn_v1 = self.encode_fn_v1();
        let encode_fn_v2 = self.encode_fn_v2();

        quote! {
            impl mavspec::rust::spec::IntoPayload for #ident {
                fn encode(
                    &self,
                    version: mavspec::rust::spec::MavLinkVersion,
                ) -> Result<mavspec::rust::spec::Payload, mavspec::rust::spec::SpecError> {
                    use mavspec::rust::spec::tbytes::{TBytesWriter, TBytesWriterFor};

                    #encode_fn_v1
                    #encode_fn_v2

                    match version {
                        mavspec::rust::spec::MavLinkVersion::V1 => encode_v1(&self),
                        mavspec::rust::spec::MavLinkVersion::V2 => encode_v2(&self),
                    }
                }
            }
        }
    }

    fn impl_default(&self) -> proc_macro2::TokenStream {
        let ident = self.ident();
        let field_defaults = self.fields_v2().map(|field| {
            let ident = field.ident();
            let default_value = field.default_value();

            quote! {
                #ident: #default_value,
            }
        });

        quote! {
            impl core::default::Default for #ident {
                fn default() -> Self {
                    Self {
                        #(#field_defaults)*
                    }
                }
            }
        }
    }

    fn impl_message_impl(&self) -> proc_macro2::TokenStream {
        let ident = self.ident();
        quote! {
            impl mavspec::rust::spec::Message for #ident {}
        }
    }

    fn decode_fn_v1(&self) -> proc_macro2::TokenStream {
        if self.message_id.supports_mavlink_1() {
            self.decode_fn(
                quote!(decode_v1),
                self.ident(),
                self.payload_size_v1(),
                self.decode_fields_v1(),
                PayloadType::Strict,
            )
        } else {
            let signature = self.decode_fn_signature();
            quote! {
                #[inline]
                fn decode_v1 #signature {
                    Err(mavspec::rust::spec::SpecError::UnsupportedMavLinkVersion{
                        actual: mavspec::rust::spec::MavLinkVersion::V1,
                        minimal: mavspec::rust::spec::MavLinkVersion::V2,
                    })
                }
            }
        }
    }

    fn decode_fn_v2(&self) -> proc_macro2::TokenStream {
        self.decode_fn(
            quote!(decode_v2),
            self.ident(),
            self.payload_size_v2(),
            self.decode_fields_v2(),
            PayloadType::Truncated,
        )
    }

    fn decode_fn_signature(&self) -> proc_macro2::TokenStream {
        let ident = self.ident();
        quote! {
            (payload: &[u8]) -> Result<#ident, mavspec::rust::spec::SpecError>
        }
    }

    fn decode_fn(
        &self,
        name: proc_macro2::TokenStream,
        ident: &syn::Ident,
        payload_size: proc_macro2::TokenStream,
        decode_fields: impl Iterator<Item = proc_macro2::TokenStream>,
        payload_type: PayloadType,
    ) -> proc_macro2::TokenStream {
        let signature = self.decode_fn_signature();
        let reader_init = Self::reader_init(payload_type);

        quote! {
            #[inline]
            fn #name #signature {
                const PAYLOAD_SIZE: usize = #payload_size;

                #reader_init

                Ok(
                    #ident {
                        #(#decode_fields,)*
                    }
                )
            }
        }
    }

    fn reader_init(payload_type: PayloadType) -> proc_macro2::TokenStream {
        match payload_type {
            PayloadType::Strict => quote! {
                if payload.len() != PAYLOAD_SIZE {
                    return Err(mavspec::rust::spec::PayloadError::InvalidV1PayloadSize {
                        actual: payload.len(),
                        expected: PAYLOAD_SIZE,
                    }.into());
                }
                let reader = TBytesReader::from(payload);
            },
            PayloadType::Truncated => quote! {
                let mut full_payload = [0u8; PAYLOAD_SIZE];
                full_payload[0..payload.len()].copy_from_slice(payload);

                let reader = TBytesReader::from(full_payload.as_slice());
            },
        }
    }

    fn decode_fields_v1(&self) -> impl Iterator<Item = proc_macro2::TokenStream> + '_ {
        let mut fields: Vec<proc_macro2::TokenStream> =
            self.fields_v1().map(Self::decode_field).collect();

        for field in self.fields_ext() {
            let ident = field.ident();
            let default_value = field.default_value();
            fields.push(quote! {
               #ident: #default_value
            });
        }

        fields.into_iter()
    }

    fn decode_fields_v2(&self) -> impl Iterator<Item = proc_macro2::TokenStream> + '_ {
        self.fields_v2().map(Self::decode_field)
    }

    fn decode_field(field: &Field) -> proc_macro2::TokenStream {
        let field_ident = field.ident();

        match field.field_type() {
            FieldType::Scalar(scalar) => match field.custom_type() {
                None => quote! {
                    #field_ident: reader.read()?
                },
                Some(_) => {
                    let base_type = scalar.to_token_stream();
                    let raw_value_converter = field.decode_raw_value_converter();

                    quote! {
                        #field_ident: {
                            let raw_value: #base_type = reader.read()?;
                            #raw_value_converter
                        }
                    }
                }
            },
            FieldType::Array(scalar, len) => match field.custom_type() {
                None => quote! {
                    #field_ident: reader.read_array()?
                },
                Some(custom_type) => {
                    let base_type = scalar.to_token_stream();
                    let raw_value_converter = field.decode_raw_value_converter();
                    let default_value = field.default_value();
                    quote! {
                        #field_ident: {
                            let raw_values: [#base_type; #len] = reader.read_array()?;
                            let mut values: [#custom_type; #len] = #default_value;
                            for i in 0..#len {
                                let raw_value = raw_values[i];
                                values[i] = #raw_value_converter;
                            }
                            values
                        }
                    }
                }
            },
        }
    }

    fn encode_fn_signature(&self) -> proc_macro2::TokenStream {
        let ident = self.ident();
        quote! {
            (message: &#ident) -> Result<mavspec::rust::spec::Payload, mavspec::rust::spec::SpecError>
        }
    }

    fn encode_fn_v1(&self) -> proc_macro2::TokenStream {
        if self.message_id.supports_mavlink_1() {
            self.encode_fn(
                quote!(encode_v1),
                format_ident!("V1"),
                self.payload_size_v1(),
                self.encode_fields_v1(),
            )
        } else {
            let signature = self.encode_fn_signature();
            quote! {
                fn encode_v1 #signature {
                    Err(mavspec::rust::spec::SpecError::UnsupportedMavLinkVersion{
                        actual: mavspec::rust::spec::MavLinkVersion::V1,
                        minimal: mavspec::rust::spec::MavLinkVersion::V2,
                    })
                }
            }
        }
    }

    fn encode_fn_v2(&self) -> proc_macro2::TokenStream {
        self.encode_fn(
            quote!(encode_v2),
            format_ident!("V2"),
            self.payload_size_v2(),
            self.encode_fields_v2(),
        )
    }

    fn encode_fn(
        &self,
        name: proc_macro2::TokenStream,
        version: syn::Ident,
        payload_size: proc_macro2::TokenStream,
        encode_fields: impl Iterator<Item = proc_macro2::TokenStream>,
    ) -> proc_macro2::TokenStream {
        let signature = self.encode_fn_signature();
        let message_id = self.message_id.literal();
        quote! {
            #[inline]
            fn #name #signature {
                const PAYLOAD_SIZE: usize = #payload_size;

                let mut buf = [0u8; PAYLOAD_SIZE];
                let mut writer = TBytesWriter::from(buf.as_mut_slice());

                #(#encode_fields;)*

                let payload = mavspec::rust::spec::Payload::new(
                    #message_id,
                    buf.as_slice(),
                    mavspec::rust::spec::MavLinkVersion::#version,
                );
                Ok(payload)
            }
        }
    }

    fn encode_fields_v1(&self) -> impl Iterator<Item = proc_macro2::TokenStream> + '_ {
        self.fields_v1().map(Self::encode_field)
    }

    fn encode_fields_v2(&self) -> impl Iterator<Item = proc_macro2::TokenStream> + '_ {
        self.fields_v2().map(Self::encode_field)
    }

    fn encode_field(field: &Field) -> proc_macro2::TokenStream {
        let field_ident = field.ident();

        match field.field_type() {
            FieldType::Scalar(_) => match field.custom_type() {
                None => quote! {
                    writer.write(message.#field_ident)?
                },
                Some(_) => {
                    let value_converter = field.encode_value_converter();
                    quote! {
                        writer.write({
                            let value = message.#field_ident;
                            #value_converter
                        })?
                    }
                }
            },
            FieldType::Array(_, _) => match field.custom_type() {
                None => quote! {
                    writer.write_array(message.#field_ident)?
                },
                Some(_) => {
                    let value_converter = field.encode_value_converter();
                    quote! {
                        writer.write_array(message.#field_ident.map(|value| #value_converter))?
                    }
                }
            },
        }
    }

    fn message_id(&self) -> &MessageId {
        &self.message_id
    }

    fn crc_extra(&self) -> &CrcExtra {
        &self.crc_extra
    }

    fn payload_size_v1(&self) -> proc_macro2::TokenStream {
        let mut size: proc_macro2::TokenStream = quote!(0);
        for field in self.fields_v1() {
            let field_size = field.size_expr();
            size.append_all(quote! { +#field_size });
        }
        size
    }

    fn payload_size_v2(&self) -> proc_macro2::TokenStream {
        let mut size: proc_macro2::TokenStream = quote!(0);
        for field in self.fields_v2() {
            let field_size = field.size_expr();
            size.append_all(quote!(+#field_size));
        }
        size
    }

    fn reorder_fields(fields: &mut [Field]) {
        fields.sort_by(|field, other| {
            if other.is_extension() || field.is_extension() {
                Ordering::Equal
            } else {
                field
                    .field_type()
                    .base_type()
                    .size()
                    .cmp(&other.field_type().base_type().size())
                    .reverse()
            }
        });
    }
}

///////////////////////////////////////////////////////////////////////////////
/////                               TESTS                                 /////
///////////////////////////////////////////////////////////////////////////////
#[cfg(test)]
mod tests {
    use quote::quote;

    use crate::errors::{Error, SpecError};
    use crate::field_types::ScalarType;

    use super::*;

    fn create_message() -> Message {
        let input: syn::DeriveInput = syn::parse2(quote! {
            #[message_id(42)]
            #[crc_extra(42)]
            struct Message {
                scalar_u8: u8,
                array_u16_4: [u16; 4],
                scalar_u16: u16,
                scalar_f32: f32,
                scalar_i32: i32,

                #[base_type(u8)]
                custom_u8: Custom,

                #[base_type(u16)]
                custom_array_u16: [Custom; 4],

                #[extension]
                ext_array_u32: u32,

                #[extension]
                #[base_type(f64)]
                #[repr_type(u8)]
                ext_custom_arr_f64: [Custom; 4],
            }
        })
        .unwrap();

        Message::try_from(input).unwrap()
    }

    fn ordered_fields_v1() -> Vec<&'static str> {
        vec![
            "scalar_f32",
            "scalar_i32",
            "array_u16_4",
            "scalar_u16",
            "custom_array_u16",
            "scalar_u8",
            "custom_u8",
        ]
    }

    fn ordered_fields_v2() -> Vec<&'static str> {
        vec![
            "scalar_f32",
            "scalar_i32",
            "array_u16_4",
            "scalar_u16",
            "custom_array_u16",
            "scalar_u8",
            "custom_u8",
            "ext_array_u32",
            "ext_custom_arr_f64",
        ]
    }

    #[test]
    fn from_derive_input() {
        let message = create_message();
        assert_eq!(message.ident(), "Message");

        let input: syn::DeriveInput = syn::parse2(quote! {
            enum Struct {}
        })
        .unwrap();
        assert!(matches!(
            Message::try_from(input),
            Err(Error::Message(SpecError::NotAStruct))
        ));
    }

    #[test]
    fn message_attributes() {
        let message = create_message();

        assert_eq!(message.message_id().literal().to_string(), "42");
        assert_eq!(message.crc_extra().literal().to_string(), "42");
    }

    #[test]
    fn field_ordering_v1() {
        let message = create_message();

        let ordered: Vec<String> = message
            .fields_v1()
            .map(|field| field.ident().to_string())
            .collect();

        assert_eq!(ordered, ordered_fields_v1());
    }

    #[test]
    fn field_ordering_v2() {
        let message = create_message();

        let ordered: Vec<String> = message
            .fields_v2()
            .map(|field| field.ident().to_string())
            .collect();

        assert_eq!(ordered, ordered_fields_v2());
    }

    #[test]
    fn custom_fields() {
        let message = create_message();

        for field in message.fields_v2() {
            match field.ident().to_string().as_str() {
                "custom_u8" => {
                    assert!(matches!(
                        field.field_type(),
                        FieldType::Scalar(ScalarType::UInt8)
                    ));
                }
                "custom_array_u16" => {
                    assert!(
                        matches!(field.field_type(), FieldType::Array(ScalarType::UInt16, _)),
                        "field `{:?}` should be of array type",
                        field,
                    );
                }
                "ext_custom_arr_f64" => {
                    assert!(
                        matches!(field.field_type(), FieldType::Array(ScalarType::Float64, _)),
                        "field `{:?}` should be of array type",
                        field,
                    );
                }
                _ => {}
            }
        }
    }
}
