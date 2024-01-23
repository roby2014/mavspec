use crate::conventions::{enum_bitmask_entry_name, enum_entry_name, enum_mod_name, enum_rust_name};
use quote::{format_ident, quote};

use crate::specs::dialects::dialect::enums::{EnumImplModuleSpec, EnumsRootModuleSpec};
use crate::specs::Spec;
use crate::templates::helpers::make_serde_derive_annotation;

pub(crate) fn enums_root_module(spec: &EnumsRootModuleSpec) -> syn::File {
    let module_doc_comment = format!("MAVLink enums of `{}` dialect.", spec.dialect_name());

    let enum_modules_and_imports = spec.enums().values().map(|enm| {
        let enum_mod_name = format_ident!("{}", enum_mod_name(enm.name().into()));
        let enum_rust_name = format_ident!("{}", enum_rust_name(enm.name().into()));
        quote! {
            pub mod #enum_mod_name;
            pub use #enum_mod_name::#enum_rust_name;
        }
    });

    syn::parse2(quote! {
        #![doc = #module_doc_comment]

        #(#enum_modules_and_imports)*
    })
    .unwrap()
}

pub(crate) fn enum_module(spec: &EnumImplModuleSpec) -> syn::File {
    let module_doc_comment = format!("MAVLink `{}` enum implementation.", spec.name());

    let bitmask_impl = make_bitmask_enum(spec);
    let enum_impl = make_enum(spec);

    syn::parse2(quote! {
        #![doc = #module_doc_comment]

        #bitmask_impl
        #enum_impl
    })
    .unwrap()
}

fn make_bitmask_enum(spec: &EnumImplModuleSpec) -> proc_macro2::TokenStream {
    let leading_doc_comment = format!("MAVLink bitmask enum `{}`.", spec.name());
    let description_doc_comments = spec.description().iter().map(|line| {
        quote! { #[doc = #line] }
    });
    let derive_serde = make_serde_derive_annotation(spec.params().serde);
    let enum_ident = format_ident!("{}", enum_rust_name(spec.name().into()));
    let enum_inferred_type = format_ident!("{}", spec.inferred_type().rust_type());

    let entry_consts = spec.entries().iter().map(|entry| {
        let name_doc_comment = format!("`{}` flag.", entry.name());
        let description_doc_comments = entry.description().iter().map(|line| {
            quote! { #[doc = #line] }
        });
        let flag_ident = format_ident!("{}", enum_bitmask_entry_name(entry.name_stripped().into()));
        let flag_value = entry.value_expr();

        quote! {
            #[doc = #name_doc_comment]
            ///
            #(#description_doc_comments)*
            const #flag_ident = #flag_value;
        }
    });

    if spec.is_bitmask() {
        quote! {
            use mavspec::rust::spec::bitflags::bitflags;

            bitflags! {
                #[allow(rustdoc::bare_urls)]
                #[allow(rustdoc::broken_intra_doc_links)]
                #[allow(rustdoc::invalid_rust_codeblocks)]
                #[doc = #leading_doc_comment]
                ///
                #(#description_doc_comments)*
                #[derive(core::marker::Copy, core::clone::Clone, core::fmt::Debug, core::default::Default)]
                #derive_serde
                pub struct #enum_ident: #enum_inferred_type {
                    #(#entry_consts)*
                }
            }
        }
    } else {
        quote!()
    }
}

fn make_enum(spec: &EnumImplModuleSpec) -> proc_macro2::TokenStream {
    if spec.is_bitmask() {
        quote!()
    } else {
        let leading_doc_comment = format!("MAVLink enum `{}`.", spec.name());
        let description_doc_comments = spec.description().iter().map(|line| {
            quote! { #[doc = #line] }
        });
        let derive_serde = make_serde_derive_annotation(spec.params().serde);
        let enum_ident = format_ident!("{}", enum_rust_name(spec.name().into()));
        let enum_inferred_type = format_ident!("{}", spec.inferred_type().rust_type());

        let enum_variants = spec.entries().iter().map(|entry| {
            let name_doc_comment = format!("MAVLink enum entry `{}`.", entry.name());
            let description_doc_comments = entry.description().iter().map(|line| {
                quote! { #[doc = #line] }
            });
            let entry_ident = format_ident!("{}", enum_entry_name(entry.name_stripped().into()));
            let entry_value = entry.value_expr();

            quote! {
                #[doc = #name_doc_comment]
                ///
                #(#description_doc_comments)*
                #entry_ident = #entry_value,
            }
        });

        let impl_leading_doc_comment = format!(
            "Attempts to create [`{enum_ident}`](enum@self::{enum_ident}) variant from discriminant (raw value)."
        );

        let enum_match_arms = spec.entries().iter().map(|entry| {
            let entry_ident = format_ident!("{}", enum_entry_name(entry.name_stripped().into()));
            let entry_value = entry.value_expr();

            quote! {
                #entry_value => Self::#entry_ident,
            }
        });

        let enum_name_str = spec.name();

        quote! {
            use mavspec::rust::spec::MessageError;

            #[cfg(not(doctest))]
            #[allow(rustdoc::bare_urls)]
            #[allow(rustdoc::broken_intra_doc_links)]
            #[allow(rustdoc::invalid_rust_codeblocks)]
            #[doc = #leading_doc_comment]
            ///
            #(#description_doc_comments)*
            #[derive(core::marker::Copy, core::clone::Clone, core::fmt::Debug, core::default::Default)]
            #[repr(#enum_inferred_type)]
            #derive_serde
            pub enum #enum_ident {
                #[default]
                #(#enum_variants)*
            }

            impl core::convert::TryFrom<#enum_inferred_type> for #enum_ident {
                type Error = MessageError;

                fn try_from(value: #enum_inferred_type) -> Result<Self, MessageError> {
                    Self::try_from_discriminant(value)
                }
            }

            impl #enum_ident {
                #[doc = #impl_leading_doc_comment]
                ///
                /// # Errors
                ///
                /// * Returns [`MessageError::InvalidEnumValue`] if there is no enum variant corresponding to discriminant `value`.
                pub fn try_from_discriminant(value: #enum_inferred_type) -> Result<Self, MessageError> {
                    Ok(match value {
                        #(#enum_match_arms)*
                        _ => {
                            return Err(MessageError::InvalidEnumValue {
                                enum_name: #enum_name_str,
                                value: value.into(),
                            })
                        }
                    })
                }
            }
        }
    }
}
