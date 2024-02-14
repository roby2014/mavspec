use crate::consts::ATTR_REPR;
use quote::{quote, ToTokens};

use crate::errors::{EnumError, Error};
use crate::field_types::ScalarType;

pub(crate) struct Enum {
    ident: syn::Ident,
    name: syn::LitStr,
    repr: ScalarType,
    variants: Vec<Variant>,
}

pub(crate) struct Variant {
    ident: syn::Ident,
    discriminant: syn::Expr,
}

impl Variant {
    pub(crate) fn ident(&self) -> &syn::Ident {
        &self.ident
    }

    pub(crate) fn discriminant(&self) -> &syn::Expr {
        &self.discriminant
    }
}

impl TryFrom<syn::DeriveInput> for Enum {
    type Error = Error;

    fn try_from(value: syn::DeriveInput) -> Result<Self, Self::Error> {
        Self::try_from_derive_input(value)
    }
}

impl Enum {
    pub(crate) fn try_from_derive_input(value: syn::DeriveInput) -> Result<Self, Error> {
        let data = match value.data {
            syn::Data::Enum(data) => data,
            _ => return Err(EnumError::NotAnEnum.into()),
        };

        let mut variants: Vec<Variant> = vec![];
        for variant in data.variants {
            let ident = variant.ident;

            let discriminant = match variant.discriminant {
                None => return Err(EnumError::MissingDiscriminant(ident.to_string()).into()),
                Some((_, discriminant)) => discriminant,
            };

            variants.push(Variant {
                ident,
                discriminant,
            });
        }

        let name = syn::parse_str(
            format!("\"{}\"", heck::AsShoutySnakeCase(value.ident.to_string())).as_str(),
        )
        .unwrap();

        let repr = Self::get_repr(value.attrs.as_slice())?;

        Ok(Self {
            ident: value.ident,
            name,
            repr,
            variants,
        })
    }

    pub(crate) fn ident(&self) -> &syn::Ident {
        &self.ident
    }

    pub(crate) fn name(&self) -> &syn::LitStr {
        &self.name
    }

    pub(crate) fn repr(&self) -> &ScalarType {
        &self.repr
    }

    pub(crate) fn variants(&self) -> &[Variant] {
        self.variants.as_slice()
    }

    pub(crate) fn to_token_stream(&self) -> proc_macro2::TokenStream {
        let ident = self.ident();
        let name = self.name();
        let repr = self.repr().to_token_stream();

        let mut variants: Vec<proc_macro2::TokenStream> = vec![];
        for variant in self.variants() {
            let ident = variant.ident();
            let discriminant = variant.discriminant();
            variants.push(quote! {
                #discriminant => Self::#ident
            });
        }

        quote! {
            impl core::convert::TryFrom<#repr> for #ident {
                type Error = mavspec::rust::spec::SpecError;

                fn try_from(value: #repr) -> Result<Self, mavspec::rust::spec::SpecError> {
                    Ok(match value {
                        #(#variants,)*
                        _ => {
                            return Err(mavspec::rust::spec::SpecError::InvalidEnumValue {
                                enum_name: #name,
                            })
                        }
                    })
                }
            }
        }
    }

    fn get_repr(attrs: &[syn::Attribute]) -> Result<ScalarType, Error> {
        for attr in attrs {
            if let Some(attr_ident) = attr.path().get_ident() {
                if attr_ident == ATTR_REPR {
                    if let syn::Expr::Path(path) = attr.parse_args().unwrap() {
                        return ScalarType::try_from(path.to_token_stream());
                    }
                }
            }
        }

        Err(EnumError::ReprIsMissing.into())
    }
}
