use crate::consts::{ATTR_BASE_TYPE, ATTR_BITMASK, ATTR_EXTENSION, ATTR_REPR_TYPE};
use quote::{quote, ToTokens};
use std::fmt::{Debug, Display, Formatter};

use crate::errors::{Error, FieldError, TypeError};
use crate::field_types::{FieldType, ScalarType};

#[derive(Clone)]
pub(crate) struct Field {
    ident: syn::Ident,
    field_type: FieldType,
    custom_type: Option<syn::TypePath>,
    repr_type: Option<ScalarType>,
    is_bitmask: bool,
    is_extension: bool,
    default_value: proc_macro2::TokenStream,
}

impl Display for Field {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}: {}", self.ident, self.field_type)
    }
}

impl Debug for Field {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Field(ident: {}, field_type: {:?}, custom_type: {:?}, is_extension: {:?})",
            self.ident.to_token_stream(),
            self.field_type,
            self.custom_type
                .as_ref()
                .map(|t| t.to_token_stream().to_string()),
            self.is_extension,
        )
    }
}

impl TryFrom<syn::Field> for Field {
    type Error = Error;

    fn try_from(value: syn::Field) -> Result<Self, Self::Error> {
        Self::try_from_syn_field(value)
    }
}

impl Field {
    pub(crate) fn try_from_syn_field(value: syn::Field) -> Result<Self, Error> {
        let ident = value.ident.unwrap();
        let is_extension = Self::has_attr(value.attrs.as_slice(), ATTR_EXTENSION);
        let is_bitmask = Self::has_attr(value.attrs.as_slice(), ATTR_BITMASK);
        let base_type = Self::get_type_from_attr(value.attrs.as_slice(), ATTR_BASE_TYPE)?;

        let repr_type = {
            let mut repr_type = Self::get_type_from_attr(value.attrs.as_slice(), ATTR_REPR_TYPE)?;
            if base_type.is_some() && repr_type.is_none() {
                repr_type = base_type.clone()
            }
            repr_type
        };
        Self::validate_enum_repr(repr_type.as_ref(), &ident)?;

        let (field_type, custom_type) = Self::derive_field_type(value.ty, base_type.as_ref())?;

        let default_value = match &field_type {
            FieldType::Scalar(_) => quote! { core::default::Default::default() },
            FieldType::Array(_, len) => {
                quote! { [core::default::Default::default(); #len] }
            }
        };

        Ok(Self {
            ident,
            field_type,
            custom_type,
            repr_type,
            is_bitmask,
            is_extension,
            default_value,
        })
    }

    pub(crate) fn ident(&self) -> &syn::Ident {
        &self.ident
    }

    pub(crate) fn field_type(&self) -> &FieldType {
        &self.field_type
    }

    pub(crate) fn custom_type(&self) -> Option<&syn::TypePath> {
        self.custom_type.as_ref()
    }

    pub(crate) fn repr_type(&self) -> Option<&ScalarType> {
        self.repr_type.as_ref()
    }

    pub(crate) fn is_extension(&self) -> bool {
        self.is_extension
    }

    pub(crate) fn is_bitmask(&self) -> bool {
        self.is_bitmask
    }

    pub(crate) fn size_expr(&self) -> proc_macro2::TokenStream {
        self.field_type.size_expr()
    }

    pub(crate) fn decode_raw_value_converter(&self) -> proc_macro2::TokenStream {
        let custom_type = self.custom_type().unwrap();
        let repr_type = self.repr_type().unwrap().to_token_stream();

        if self.is_bitmask() {
            quote! {
                #custom_type::from_bits_truncate(raw_value as #repr_type)
            }
        } else {
            quote! {
                #custom_type::try_from(raw_value as #repr_type)?
            }
        }
    }

    pub(crate) fn encode_value_converter(&self) -> proc_macro2::TokenStream {
        let base_type = self.field_type().base_type().to_token_stream();
        let repr_type = self.repr_type().unwrap().to_token_stream();

        if self.is_bitmask() {
            quote! {
                value.bits() as #base_type
            }
        } else {
            quote! {
                (value as #repr_type) as #base_type
            }
        }
    }

    pub(crate) fn default_value(&self) -> &proc_macro2::TokenStream {
        &self.default_value
    }

    fn has_attr(attrs: &[syn::Attribute], name: &str) -> bool {
        for attr in attrs {
            if let Some(ident) = attr.path().get_ident() {
                if ident == name {
                    return true;
                }
            }
        }
        false
    }

    fn validate_enum_repr(
        repr: Option<&ScalarType>,
        field_ident: &syn::Ident,
    ) -> Result<(), Error> {
        if let Some(repr) = repr {
            if !repr.is_integer() {
                return Err(FieldError::NonIntegerRepr(field_ident.to_string()).into());
            }
        }

        Ok(())
    }

    fn get_type_from_attr(
        attrs: &[syn::Attribute],
        attr_name: &str,
    ) -> Result<Option<ScalarType>, Error> {
        let mut base_type: Option<ScalarType> = None;

        for attr in attrs {
            if let Some(ident) = attr.path().get_ident() {
                if ident == attr_name {
                    let argument: syn::ExprPath = match attr.parse_args() {
                        Ok(expr) => {
                            if let syn::Expr::Path(path) = expr {
                                path
                            } else {
                                return Err(FieldError::InvalidBaseTypeArgument(
                                    expr.to_token_stream().to_string(),
                                )
                                .into());
                            }
                        }
                        Err(err) => return Err(FieldError::BaseTypeArgumentParseError(err).into()),
                    };

                    base_type = Some(ScalarType::try_from(argument.to_token_stream())?);
                    break;
                }
            }
        }

        Ok(base_type)
    }

    fn derive_field_type(
        syn_type: syn::Type,
        base_type: Option<&ScalarType>,
    ) -> Result<(FieldType, Option<syn::TypePath>), Error> {
        if let Some(scalar) = base_type {
            match syn_type {
                syn::Type::Array(syn::TypeArray { elem, len, .. }) => match elem.as_ref() {
                    syn::Type::Path(path) => {
                        Ok((FieldType::Array(scalar.clone(), len), Some(path.clone())))
                    }
                    _ => Err(
                        TypeError::InvalidArrayElement(elem.to_token_stream().to_string()).into(),
                    ),
                },
                syn::Type::Path(path) => Ok((FieldType::Scalar(scalar.clone()), Some(path))),
                _ => Err(TypeError::Invalid(syn_type.to_token_stream().to_string()).into()),
            }
        } else {
            Ok((FieldType::try_from(syn_type)?, None))
        }
    }
}

///////////////////////////////////////////////////////////////////////////////
/////                               TESTS                                 /////
///////////////////////////////////////////////////////////////////////////////
#[cfg(test)]
mod tests {
    use quote::{quote, ToTokens};
    use syn::Data;

    use crate::field_types::ScalarType;

    use super::*;

    fn create_fields() -> Vec<Field> {
        let input: syn::DeriveInput = syn::parse2(quote! {
            struct Struct {
                scalar_u8: u8,
                array_u8_4: [u8; 4],

                #[base_type(u8)]
                custom_u8: Custom,

                #[base_type(f64)]
                #[repr_type(u8)]
                custom_arr_f64_large: [Custom; 10],

                #[base_type(u16)]
                #[repr_type(u8)]
                custom_u16_large: Custom,

                #[extension]
                ext_scalar_u16: u16,

                #[extension]
                #[base_type(u8)]
                ext_custom_u8: Custom,

                #[extension]
                #[base_type(u8)]
                ext_custom_array_u8_4: [Custom; 4],
            }
        })
        .unwrap();

        match input.data {
            Data::Struct(struct_) => {
                let mut fields: Vec<Field> = vec![];
                for field in struct_.fields {
                    fields.push(Field::try_from(field).unwrap());
                }
                fields
            }
            _ => panic!("not a struct: {}", input.to_token_stream()),
        }
    }

    #[test]
    fn field_types() {
        for field in create_fields() {
            match field.ident().to_string().as_str() {
                "scalar_u8" => {
                    assert!(matches!(
                        field.field_type(),
                        &FieldType::Scalar(ScalarType::UInt8)
                    ));
                }
                "array_u8_4" => {
                    assert!(matches!(
                        field.field_type(),
                        &FieldType::Array(ScalarType::UInt8, _)
                    ));
                }
                "custom_u8" => {
                    assert!(matches!(
                        field.field_type(),
                        &FieldType::Scalar(ScalarType::UInt8)
                    ));
                }
                "custom_arr_f64_large" => {
                    assert!(matches!(
                        field.field_type(),
                        &FieldType::Array(ScalarType::Float64, _)
                    ));
                }
                "custom_u16_large" => {
                    assert!(matches!(
                        field.field_type(),
                        &FieldType::Scalar(ScalarType::UInt16)
                    ));
                }
                "ext_scalar_u16" => {
                    assert!(matches!(
                        field.field_type(),
                        &FieldType::Scalar(ScalarType::UInt16)
                    ));
                }
                "ext_custom_u8" => {
                    assert!(matches!(
                        field.field_type(),
                        &FieldType::Scalar(ScalarType::UInt8)
                    ));
                }
                "ext_custom_array_u8_4" => {
                    assert!(matches!(
                        field.field_type(),
                        &FieldType::Array(ScalarType::UInt8, _)
                    ));
                }
                _ => panic!("unknown field: {}", field),
            }
        }
    }

    #[test]
    fn custom_types() {
        for field in create_fields() {
            match field.ident().to_string().as_str() {
                "scalar_u8" => assert!(field.custom_type().is_none()),
                "array_u8_4" => assert!(field.custom_type().is_none()),
                "custom_u8" => assert_eq!(
                    field.custom_type().unwrap().to_token_stream().to_string(),
                    "Custom"
                ),
                "custom_arr_f64_large" => assert_eq!(
                    field.custom_type().unwrap().to_token_stream().to_string(),
                    "Custom"
                ),
                "custom_u16_large" => assert_eq!(
                    field.custom_type().unwrap().to_token_stream().to_string(),
                    "Custom"
                ),
                "ext_scalar_u16" => assert!(field.custom_type().is_none()),
                "ext_custom_u8" => assert_eq!(
                    field.custom_type().unwrap().to_token_stream().to_string(),
                    "Custom"
                ),
                "ext_custom_array_u8_4" => assert_eq!(
                    field.custom_type().unwrap().to_token_stream().to_string(),
                    "Custom"
                ),
                _ => panic!("unknown field: {}", field),
            }
        }
    }

    #[test]
    fn extension_fields() {
        for field in create_fields() {
            match field.ident().to_string().as_str() {
                "scalar_u8" => assert!(!field.is_extension()),
                "array_u8_4" => assert!(!field.is_extension()),
                "custom_u8" => assert!(!field.is_extension()),
                "custom_arr_f64_large" => assert!(!field.is_extension()),
                "custom_u16_large" => assert!(!field.is_extension()),
                "ext_scalar_u16" => assert!(field.is_extension()),
                "ext_custom_u8" => assert!(field.is_extension()),
                "ext_custom_array_u8_4" => assert!(field.is_extension()),
                _ => panic!("unknown field: {}", field),
            }
        }
    }

    #[test]
    fn fields_larger_than_custom_types() {
        for field in create_fields() {
            match field.ident().to_string().as_str() {
                "scalar_u8" => {
                    assert!(field.custom_type().is_none() && field.repr_type().is_none())
                }
                "array_u8_4" => {
                    assert!(field.custom_type().is_none() && field.repr_type().is_none())
                }
                "custom_u8" => {
                    assert_eq!(field.field_type().base_type(), field.repr_type().unwrap())
                }
                "custom_arr_f64_large" => {
                    assert!(field.repr_type().is_some());
                    assert_ne!(field.field_type().base_type(), field.repr_type().unwrap());
                    assert!(
                        field.field_type().base_type().size() > field.repr_type().unwrap().size()
                    );
                }
                "custom_u16_large" => {
                    assert!(field.repr_type().is_some());
                    assert_ne!(field.field_type().base_type(), field.repr_type().unwrap());
                    assert!(
                        field.field_type().base_type().size() > field.repr_type().unwrap().size()
                    );
                }
                "ext_scalar_u16" => {
                    assert!(field.custom_type().is_none() && field.repr_type().is_none())
                }
                "ext_custom_u8" => {
                    assert_eq!(field.field_type().base_type(), field.repr_type().unwrap())
                }
                "ext_custom_array_u8_4" => {
                    assert_eq!(field.field_type().base_type(), field.repr_type().unwrap())
                }
                _ => panic!("unknown field: {}", field),
            }
        }
    }
}
