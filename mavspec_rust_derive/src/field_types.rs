use std::fmt::{Debug, Display, Formatter};
use std::ops::Deref;

use crate::errors::{Error, TypeError};
use quote::{quote, ToTokens};

#[derive(Clone)]
pub(crate) enum FieldType {
    Scalar(ScalarType),
    Array(ScalarType, syn::Expr),
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) enum ScalarType {
    Int8,
    Int16,
    Int32,
    Int64,
    UInt8,
    UInt16,
    UInt32,
    UInt64,
    Float32,
    Float64,
}

impl TryFrom<syn::Type> for FieldType {
    type Error = Error;

    fn try_from(value: syn::Type) -> Result<Self, Self::Error> {
        Self::try_from_syn_type(value)
    }
}

impl TryFrom<proc_macro2::TokenStream> for FieldType {
    type Error = Error;

    fn try_from(value: proc_macro2::TokenStream) -> Result<Self, Self::Error> {
        Self::try_from_token_stream(value)
    }
}

impl Debug for FieldType {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            FieldType::Scalar(scalar) => write!(f, "FieldType({})", scalar.to_token_stream()),
            FieldType::Array(scalar, len) => write!(
                f,
                "FieldType([{}; {}])",
                scalar.to_token_stream(),
                len.to_token_stream()
            ),
        }
    }
}

impl Display for FieldType {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.to_token_stream())
    }
}

impl PartialEq for FieldType {
    fn eq(&self, other: &Self) -> bool {
        match self {
            FieldType::Scalar(scalar) => match other {
                FieldType::Scalar(other) => other == scalar,
                _ => false,
            },
            FieldType::Array(scalar, len) => match other {
                FieldType::Array(other_scalar, other_len) => {
                    scalar == other_scalar
                        && len.to_token_stream().to_string()
                            == other_len.to_token_stream().to_string()
                }
                _ => false,
            },
        }
    }
}

impl FieldType {
    pub(crate) fn try_from_syn_type(value: syn::Type) -> Result<Self, Error> {
        Ok(match value {
            syn::Type::Array(arr) => {
                let scalar_type = match arr.elem.deref() {
                    syn::Type::Path(path) => ScalarType::try_from(path.clone())?,
                    _ => {
                        return Err(TypeError::InvalidArrayElement(
                            arr.to_token_stream().to_string(),
                        )
                        .into())
                    }
                };

                Self::Array(scalar_type, arr.len)
            }
            syn::Type::Path(path) => Self::Scalar(path.try_into()?),
            _ => return Err(TypeError::Invalid(value.to_token_stream().to_string()).into()),
        })
    }

    pub(crate) fn try_from_token_stream(value: proc_macro2::TokenStream) -> Result<Self, Error> {
        match syn::parse2(value) {
            Ok(value) => Self::try_from_syn_type(value),
            Err(err) => Err(TypeError::ParseError(err).into()),
        }
    }

    pub(crate) fn to_token_stream(&self) -> proc_macro2::TokenStream {
        match self {
            FieldType::Scalar(scalar) => scalar.to_token_stream(),
            FieldType::Array(scalar, len) => {
                let scalar = scalar.to_token_stream();
                quote!([#scalar; #len])
            }
        }
    }

    pub(crate) fn size_expr(&self) -> proc_macro2::TokenStream {
        match self {
            FieldType::Scalar(scalar) => scalar.size_expr(),
            FieldType::Array(scalar, len) => {
                let scalar_size_expr = scalar.size_expr();
                quote!(#scalar_size_expr * #len)
            }
        }
    }

    pub(crate) fn base_type(&self) -> &ScalarType {
        match self {
            FieldType::Scalar(scalar) => scalar,
            FieldType::Array(scalar, _) => scalar,
        }
    }
}

impl TryFrom<syn::TypePath> for ScalarType {
    type Error = Error;

    fn try_from(value: syn::TypePath) -> Result<Self, Self::Error> {
        Self::try_from_type_path(value)
    }
}

impl TryFrom<proc_macro2::TokenStream> for ScalarType {
    type Error = Error;

    fn try_from(value: proc_macro2::TokenStream) -> Result<Self, Self::Error> {
        Self::try_from_token_stream(value)
    }
}

impl ScalarType {
    pub(crate) fn try_from_type_path(value: syn::TypePath) -> Result<Self, Error> {
        Self::try_from_str(value.to_token_stream().to_string().as_str())
    }

    pub(crate) fn try_from_token_stream(value: proc_macro2::TokenStream) -> Result<Self, Error> {
        match syn::parse2(value) {
            Ok(value) => Self::try_from_type_path(value),
            Err(err) => Err(TypeError::ParseError(err).into()),
        }
    }

    pub(crate) fn try_from_str(value: &str) -> Result<Self, Error> {
        Ok(match value {
            "i8" => Self::Int8,
            "i16" => Self::Int16,
            "i32" => Self::Int32,
            "i64" => Self::Int64,
            "u8" => Self::UInt8,
            "u16" => Self::UInt16,
            "u32" => Self::UInt32,
            "u64" => Self::UInt64,
            "f32" => Self::Float32,
            "f64" => Self::Float64,
            _ => return Err(TypeError::InvalidScalar(value.into()).into()),
        })
    }

    pub(crate) fn to_token_stream(&self) -> proc_macro2::TokenStream {
        match self {
            ScalarType::Int8 => quote!(i8),
            ScalarType::Int16 => quote!(i16),
            ScalarType::Int32 => quote!(i32),
            ScalarType::Int64 => quote!(i64),
            ScalarType::UInt8 => quote!(u8),
            ScalarType::UInt16 => quote!(u16),
            ScalarType::UInt32 => quote!(u32),
            ScalarType::UInt64 => quote!(u64),
            ScalarType::Float32 => quote!(f32),
            ScalarType::Float64 => quote!(f64),
        }
    }

    pub(crate) fn size_expr(&self) -> proc_macro2::TokenStream {
        match self {
            ScalarType::Int8 => quote!(1),
            ScalarType::Int16 => quote!(2),
            ScalarType::Int32 => quote!(4),
            ScalarType::Int64 => quote!(8),
            ScalarType::UInt8 => quote!(1),
            ScalarType::UInt16 => quote!(2),
            ScalarType::UInt32 => quote!(4),
            ScalarType::UInt64 => quote!(8),
            ScalarType::Float32 => quote!(4),
            ScalarType::Float64 => quote!(8),
        }
    }

    pub(crate) fn size(&self) -> u8 {
        match self {
            ScalarType::Int8 => 1,
            ScalarType::Int16 => 2,
            ScalarType::Int32 => 4,
            ScalarType::Int64 => 8,
            ScalarType::UInt8 => 1,
            ScalarType::UInt16 => 2,
            ScalarType::UInt32 => 4,
            ScalarType::UInt64 => 8,
            ScalarType::Float32 => 4,
            ScalarType::Float64 => 8,
        }
    }

    pub(crate) fn is_integer(&self) -> bool {
        !matches!(self, ScalarType::Float32 | ScalarType::Float64)
    }
}

///////////////////////////////////////////////////////////////////////////////
/////                               TESTS                                 /////
///////////////////////////////////////////////////////////////////////////////
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn type_parsing() -> Result<(), Error> {
        let type_ = FieldType::try_from(quote!(u8))?;
        assert!(matches!(type_, FieldType::Scalar(ScalarType::UInt8)));
        assert_eq!(type_.to_token_stream().to_string().as_str(), "u8");

        let type_ = FieldType::try_from(quote!(f32))?;
        assert!(matches!(type_, FieldType::Scalar(ScalarType::Float32)));
        assert_eq!(type_.to_token_stream().to_string().as_str(), "f32");

        let type_ = FieldType::try_from(quote!([f64; 4]))?;
        assert!(matches!(type_, FieldType::Array(ScalarType::Float64, _)));
        assert_eq!(type_.to_token_stream().to_string().as_str(), "[f64 ; 4]");

        Ok(())
    }

    #[test]
    fn type_sizes() -> Result<(), Error> {
        let type_ = FieldType::try_from(quote!(u8))?;
        assert_eq!(type_.base_type().size(), 1);

        let type_ = FieldType::try_from(quote!(f32))?;
        assert_eq!(type_.base_type().size(), 4);

        let type_ = FieldType::try_from(quote!([f64; 4]))?;
        assert_eq!(type_.base_type().size(), 8);

        Ok(())
    }

    #[test]
    fn partial_equality() -> Result<(), Error> {
        assert_eq!(
            FieldType::try_from(quote!(u8))?,
            FieldType::try_from(quote!(u8))?
        );

        assert_eq!(
            FieldType::try_from(quote!([u8; 3]))?,
            FieldType::try_from(quote!([u8; 3]))?
        );

        assert_ne!(
            FieldType::try_from(quote!(u8))?,
            FieldType::try_from(quote!(u16))?
        );

        assert_ne!(
            FieldType::try_from(quote!([u8; 4]))?,
            FieldType::try_from(quote!([u16; 4]))?
        );

        assert_ne!(
            FieldType::try_from(quote!([u8; 3]))?,
            FieldType::try_from(quote!([u8; 4]))?
        );

        Ok(())
    }
}
