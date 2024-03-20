use quote::{format_ident, quote, ToTokens};

use crate::errors::{DialectError, Error};

pub(crate) struct Dialect {
    ident: syn::Ident,
    name: String,
    dialect_id: Option<u32>,
    version: Option<u8>,
    messages_count: usize,
    variants: Vec<Variant>,
}

struct Variant {
    ident: syn::Ident,
    message_type: proc_macro2::TokenStream,
}

struct DialectAttrs {
    name: String,
    dialect_id: Option<u32>,
    version: Option<u8>,
}

const ATTR_DIALECT_NAME: &str = "name";
const ATTR_DIALECT_ID: &str = "dialect";
const ATTR_DIALECT_VERSION: &str = "version";

impl TryFrom<syn::DeriveInput> for Dialect {
    type Error = Error;

    fn try_from(value: syn::DeriveInput) -> Result<Self, Self::Error> {
        Self::try_from_derive_input(value)
    }
}

impl Dialect {
    pub(crate) fn try_from_derive_input(value: syn::DeriveInput) -> Result<Self, Error> {
        let data = match value.data {
            syn::Data::Enum(data) => data,
            _ => return Err(DialectError::NotAnEnum.into()),
        };

        let messages_count = data.variants.len();
        let variants = Self::load_variants(data.clone())?;
        let attrs = DialectAttrs::parse(&value.ident, &value.attrs)?;

        Ok(Self {
            ident: value.ident.clone(),
            name: attrs.name,
            dialect_id: attrs.dialect_id,
            version: attrs.version,
            messages_count,
            variants,
        })
    }

    pub(crate) fn to_token_stream(&self) -> proc_macro2::TokenStream {
        let dialect_enum_ident = self.ident.clone();
        let dialect_name = self.name_literal();
        let dialect_id = self.dialect_id_expr();
        let dialect_version = self.dialect_version_expr();

        let message_spec_const_ident = self.message_spec_const_ident();
        let dialect_spec_const_ident = self.dialect_spec_const_ident();

        let messages_count = self.messages_count();
        let messages_specs = self.messages_specs();

        let message_spec_id_arms = self.variants.iter().map(|variant| {
            let enum_variant_ident = variant.ident.clone();
            let message_type = variant.message_type.clone();
            quote! {
                #dialect_enum_ident::#enum_variant_ident(_) => #message_type::message_id(),
            }
        });

        let message_spec_msmv_arms = self.variants.iter().map(|variant| {
            let enum_variant_ident = variant.ident.clone();
            let message_type = variant.message_type.clone();
            quote! {
                #dialect_enum_ident::#enum_variant_ident(_) => #message_type::min_supported_mavlink_version(),
            }
        });

        let message_spec_crc_extra_arms = self.variants.iter().map(|variant| {
            let enum_variant_ident = variant.ident.clone();
            let message_type = variant.message_type.clone();
            quote! {
                #dialect_enum_ident::#enum_variant_ident(_) => #message_type::crc_extra(),
            }
        });

        let decode_arms = self.variants.iter().map(|variant| {
            let enum_variant_ident = variant.ident.clone();
            let message_type = variant.message_type.clone();

            quote! {
                id if #message_type::message_id() == id => {
                    #dialect_enum_ident::#enum_variant_ident(
                        #message_type::try_from(payload)?
                    )
                },
            }
        });

        let encode_arms = self.variants.iter().map(|variant| {
            let enum_variant_ident = variant.ident.clone();
            quote! {
                #dialect_enum_ident::#enum_variant_ident(message) => message.encode(version)?,
            }
        });

        let allow_unreachable = quote! {
            #[allow(unreachable_patterns)]
            #[allow(unreachable_code)]
        };

        quote! {
            const #message_spec_const_ident: [mavspec::rust::spec::MessageInfo; #messages_count] = [#(#messages_specs,)*];
            const #dialect_spec_const_ident: mavspec::rust::spec::DialectSpec = mavspec::rust::spec::DialectSpec::new(
                #dialect_name,
                #dialect_id,
                #dialect_version,
                &#message_spec_const_ident
            );

            impl mavspec::rust::spec::Dialect for #dialect_enum_ident {
                #[inline]
                fn name() -> &'static str {
                    #dialect_name
                }

                #[inline]
                fn dialect() -> core::option::Option<mavspec::rust::spec::types::DialectId> {
                    #dialect_id
                }

                #[inline]
                fn version() -> core::option::Option<mavspec::rust::spec::types::DialectVersion> {
                    #dialect_version
                }

                fn message_info(id: mavspec::rust::spec::types::MessageId) -> core::result::Result<&'static dyn mavspec::rust::spec::MessageSpec, mavspec::rust::spec::SpecError> {
                    for info in &#message_spec_const_ident {
                        if info.id() == id {
                            return Ok(info);
                        }
                    }
                    Err(mavspec::rust::spec::SpecError::NotInDialect(id))
                }

                fn decode(payload: &mavspec::rust::spec::Payload) -> core::result::Result<Self, mavspec::rust::spec::SpecError> {
                    #allow_unreachable
                    Ok(match payload.id() {
                        #(#decode_arms)*
                        id => return Err(mavspec::rust::spec::SpecError::NotInDialect(id)),
                    })
                }

                #[inline(always)]
                fn spec() -> &'static mavspec::rust::spec::DialectSpec {
                    &#dialect_spec_const_ident
                }
            }

            impl core::convert::TryFrom<&mavspec::rust::spec::Payload> for #dialect_enum_ident {
                type Error = mavspec::rust::spec::SpecError;

                fn try_from(value: &mavspec::rust::spec::Payload) -> Result<Self, Self::Error> {
                    use mavspec::rust::spec::Dialect;
                    Self::decode(value)
                }
            }

            impl mavspec::rust::spec::IntoPayload for #dialect_enum_ident {
                fn encode(
                    &self,
                    version: mavspec::rust::spec::MavLinkVersion,
                ) -> Result<mavspec::rust::spec::Payload, mavspec::rust::spec::SpecError> {
                    Ok(match self {
                        #(#encode_arms)*
                    })
                }
            }

            impl mavspec::rust::spec::MessageSpec for #dialect_enum_ident {
                fn id(&self) -> mavspec::rust::spec::types::MessageId {
                    match self {
                        #(#message_spec_id_arms)*
                    }
                }

                fn min_supported_mavlink_version(&self) -> mavspec::rust::spec::MavLinkVersion {
                    match self {
                        #(#message_spec_msmv_arms)*
                    }
                }

                fn crc_extra(&self) -> mavspec::rust::spec::types::CrcExtra {
                    match self {
                        #(#message_spec_crc_extra_arms)*
                    }
                }
            }
        }
    }

    fn name_literal(&self) -> &str {
        self.name.as_str()
    }

    fn name_canonical_uppercase(&self) -> String {
        self.name.to_uppercase()
    }

    fn dialect_id_expr(&self) -> proc_macro2::TokenStream {
        match self.dialect_id {
            None => quote! { core::option::Option::None },
            Some(value) => quote! { core::option::Option::Some(#value) },
        }
    }

    fn dialect_version_expr(&self) -> proc_macro2::TokenStream {
        match self.version {
            None => quote! { core::option::Option::None },
            Some(value) => quote! { core::option::Option::Some(#value) },
        }
    }

    fn messages_specs(&self) -> impl Iterator<Item = proc_macro2::TokenStream> {
        let mut items = Vec::default();

        for variant in &self.variants {
            let message_type = variant.message_type.clone();

            items.push(quote! {
                mavspec::rust::spec::MessageInfo::new(#message_type::message_id(), #message_type::crc_extra())
            });
        }

        items.into_iter()
    }

    fn messages_count(&self) -> usize {
        self.messages_count
    }

    fn message_spec_const_ident(&self) -> proc_macro2::Ident {
        format_ident!("__MAVSPEC__MESSAGES__{}", self.name_canonical_uppercase())
    }

    fn dialect_spec_const_ident(&self) -> proc_macro2::Ident {
        format_ident!(
            "__MAVSPEC__DIALECT_SPEC__{}",
            self.name_canonical_uppercase()
        )
    }

    fn load_variants(data: syn::DataEnum) -> Result<Vec<Variant>, Error> {
        let mut variants: Vec<Variant> = vec![];

        for variant in data.variants {
            let ident = variant.ident;

            if variant.fields.is_empty() {
                return Err(DialectError::MissingEnumFields.into());
            }
            if variant.fields.len() > 1 {
                return Err(DialectError::MultipleEnumFields.into());
            }

            let field = variant.fields.into_iter().next().unwrap();
            let message_type = field.to_token_stream();

            variants.push(Variant {
                ident,
                message_type,
            });
        }

        Ok(variants)
    }
}

impl DialectAttrs {
    fn parse(ident: &syn::Ident, attrs: &Vec<syn::Attribute>) -> Result<Self, Error> {
        let mut name: String = heck::AsSnakeCase(ident.to_string()).to_string();
        let mut dialect_id: Option<u32> = None;
        let mut version: Option<u8> = None;

        for attr in attrs {
            if let Some(attr_ident) = attr.path().get_ident() {
                if attr_ident == ATTR_DIALECT_NAME {
                    let lit: syn::LitStr = attr
                        .parse_args()
                        .map_err(|_| Error::from(DialectError::InvalidDialectName))?;
                    name = lit.value();
                }
                if attr_ident == ATTR_DIALECT_ID {
                    let lit: syn::LitInt = attr
                        .parse_args()
                        .map_err(|_| Error::from(DialectError::InvalidDialectId))?;
                    dialect_id = match lit.base10_parse::<u32>() {
                        Ok(value) => Some(value),
                        Err(_) => return Err(DialectError::InvalidDialectId.into()),
                    }
                }
                if attr_ident == ATTR_DIALECT_VERSION {
                    let lit: syn::LitInt = attr
                        .parse_args()
                        .map_err(|_| Error::from(DialectError::InvalidDialectId))?;
                    version = match lit.base10_parse::<u8>() {
                        Ok(value) => Some(value),
                        Err(_) => return Err(DialectError::InvalidDialectVersion.into()),
                    }
                }
            }
        }

        Ok(Self {
            name,
            dialect_id,
            version,
        })
    }
}
