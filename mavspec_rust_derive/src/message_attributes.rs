use quote::quote;

use crate::consts::{ATTR_CRC_EXTRA, ATTR_MESSAGE_ID, MESSAGE_ID_MAX};
use crate::errors::{CrcExtraError, Error, MessageIdError};

pub(crate) struct MessageId {
    literal: syn::LitInt,
    value: u32,
}

pub(crate) struct CrcExtra(syn::LitInt);

impl TryFrom<&Vec<syn::Attribute>> for MessageId {
    type Error = Error;

    fn try_from(value: &Vec<syn::Attribute>) -> Result<Self, Self::Error> {
        Self::try_from_attrs(value)
    }
}

impl TryFrom<&Vec<syn::Attribute>> for CrcExtra {
    type Error = Error;

    fn try_from(value: &Vec<syn::Attribute>) -> Result<Self, Self::Error> {
        Self::try_from_attrs(value)
    }
}

impl MessageId {
    pub(crate) fn try_from_attrs(attrs: &Vec<syn::Attribute>) -> Result<Self, Error> {
        let mut message_id: Option<MessageId> = None;

        for attr in attrs {
            if let Some(attr_ident) = attr.path().get_ident() {
                if attr_ident == ATTR_MESSAGE_ID {
                    match attr.parse_args().unwrap() {
                        syn::Expr::Lit(literal) => match literal.lit {
                            syn::Lit::Int(int_literal) => {
                                let value = match int_literal.base10_parse::<u32>() {
                                    Ok(value) => {
                                        if value > MESSAGE_ID_MAX {
                                            return Err(MessageIdError::OutOfBounds(value).into());
                                        }
                                        value
                                    }
                                    Err(err) => {
                                        return Err(MessageIdError::InvalidU32Literal(err).into())
                                    }
                                };

                                message_id = Some(MessageId {
                                    literal: int_literal,
                                    value,
                                });
                            }
                            _ => return Err(MessageIdError::NotAnInteger.into()),
                        },
                        _ => return Err(MessageIdError::NotALiteral.into()),
                    }
                }
            }
        }

        if message_id.is_none() {
            return Err(MessageIdError::MissingAttribute.into());
        }

        Ok(message_id.unwrap())
    }

    pub(crate) fn literal(&self) -> &syn::LitInt {
        &self.literal
    }

    pub(crate) fn supports_mavlink_1(&self) -> bool {
        self.value <= u8::MAX as u32
    }

    pub(crate) fn min_supported_mavlink_version(&self) -> proc_macro2::TokenStream {
        if self.supports_mavlink_1() {
            quote! {mavspec::rust::spec::MavLinkVersion::V1}
        } else {
            quote! {mavspec::rust::spec::MavLinkVersion::V2}
        }
    }
}

impl CrcExtra {
    pub(crate) fn try_from_attrs(attrs: &Vec<syn::Attribute>) -> Result<Self, Error> {
        let mut crc_extra: Option<CrcExtra> = None;

        for attr in attrs {
            if let Some(attr_ident) = attr.path().get_ident() {
                if attr_ident == ATTR_CRC_EXTRA {
                    match attr.parse_args().unwrap() {
                        syn::Expr::Lit(literal) => match literal.lit {
                            syn::Lit::Int(int_literal) => {
                                Self::validate_int_literal(&int_literal)?;

                                crc_extra = Some(CrcExtra(int_literal));
                            }
                            _ => return Err(CrcExtraError::NotAnInteger.into()),
                        },
                        _ => return Err(CrcExtraError::NotALiteral.into()),
                    }
                }
            }
        }

        if crc_extra.is_none() {
            return Err(CrcExtraError::MissingAttribute.into());
        }

        Ok(crc_extra.unwrap())
    }

    pub(crate) fn literal(&self) -> &syn::LitInt {
        &self.0
    }

    fn validate_int_literal(int_literal: &syn::LitInt) -> Result<(), Error> {
        match int_literal.base10_parse::<u8>() {
            Ok(value) => value,
            Err(err) => return Err(CrcExtraError::InvalidU8Literal(err).into()),
        };

        Ok(())
    }
}
