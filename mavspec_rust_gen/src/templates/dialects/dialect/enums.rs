use crate::conventions::{
    dialect_mod_name, enum_bitmask_entry_name, enum_entry_name, enum_mod_name, enum_rust_name,
};
use quote::{format_ident, quote};

use crate::specs::dialects::dialect::enums::{
    EnumImplModuleSpec, EnumInheritedModuleSpec, EnumsRootModuleSpec,
};
use crate::specs::Spec;
use crate::templates::helpers::make_serde_derive_annotation;

pub(crate) fn enums_root_module(spec: &EnumsRootModuleSpec) -> syn::File {
    let module_doc_comment = format!(" MAVLink enums of `{}` dialect.", spec.dialect_name());

    let enum_modules_and_imports = spec.enums().iter().map(|enm| {
        let enum_mod_name = format_ident!("{}", enum_mod_name(enm.name()));
        let enum_rust_name = format_ident!("{}", enum_rust_name(enm.name()));
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
    let module_doc_comment = format!(" MAVLink `{}` enum implementation.", spec.name());

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
    let leading_doc_comment = format!(" MAVLink bitmask enum `{}`.", spec.name());
    let description_doc_comments = spec.description().iter().map(|line| {
        quote! { #[doc = #line] }
    });
    let derive_serde = make_serde_derive_annotation(spec.params().serde);
    let enum_ident = format_ident!("{}", enum_rust_name(spec.name()));
    let enum_inferred_type = format_ident!("{}", spec.inferred_type().rust_type());

    let entry_consts = spec.entries().iter().map(|entry| {
        let name_doc_comment = format!("`{}` flag.", entry.name());
        let description_doc_comments = entry.description().iter().map(|line| {
            quote! { #[doc = #line] }
        });
        let flag_ident = format_ident!("{}", enum_bitmask_entry_name(entry.name_stripped()));
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

            #[allow(rustdoc::bare_urls)]
            #[allow(rustdoc::broken_intra_doc_links)]
            #[allow(rustdoc::invalid_rust_codeblocks)]
            #[doc = #leading_doc_comment]
            ///
            #(#description_doc_comments)*
            #[derive(core::marker::Copy, core::clone::Clone, core::fmt::Debug, core::default::Default)]
            #derive_serde
            pub struct #enum_ident(#enum_inferred_type);

            bitflags! {
                impl #enum_ident: #enum_inferred_type {
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
        let leading_doc_comment = format!(" MAVLink enum `{}`.", spec.name());
        let description_doc_comments = spec.description().iter().map(|line| {
            quote! { #[doc = #line] }
        });
        let derive_serde = make_serde_derive_annotation(spec.params().serde);
        let enum_ident = format_ident!("{}", enum_rust_name(spec.name()));
        let enum_inferred_type = format_ident!("{}", spec.inferred_type().rust_type());

        let enum_variants = spec.entries().iter().map(|entry| {
            let name_doc_comment = format!(" MAVLink enum entry `{}`.", entry.name());
            let description_doc_comments = entry.description().iter().map(|line| {
                quote! { #[doc = #line] }
            });
            let entry_ident = format_ident!("{}", enum_entry_name(entry.name_stripped()));
            let entry_value = entry.value_expr();

            quote! {
                #[doc = #name_doc_comment]
                ///
                #(#description_doc_comments)*
                #entry_ident = #entry_value,
            }
        });

        quote! {
            #[cfg(not(doctest))]
            #[allow(rustdoc::bare_urls)]
            #[allow(rustdoc::broken_intra_doc_links)]
            #[allow(rustdoc::invalid_rust_codeblocks)]
            #[doc = #leading_doc_comment]
            ///
            #(#description_doc_comments)*
            #[derive(mavspec::rust::derive::Enum)]
            #[derive(core::marker::Copy, core::clone::Clone, core::fmt::Debug, core::default::Default)]
            #[repr(#enum_inferred_type)]
            #derive_serde
            pub enum #enum_ident {
                #[default]
                #(#enum_variants)*
            }
        }
    }
}

pub(crate) fn enum_inherited_module(spec: &EnumInheritedModuleSpec) -> syn::File {
    let module_doc_comment = format!(
        " MAVLink enum `{}` inherited from `{}` dialect.",
        spec.name(),
        spec.original_dialect_name()
    );

    let enum_ident = format_ident!("{}", enum_rust_name(spec.name()));
    let dialect_mod_ident =
        format_ident!("{}", dialect_mod_name(spec.original_dialect_name().into()));
    let enum_mod_ident = format_ident!("{}", enum_mod_name(spec.name()));
    let enum_doc_comment = format!(" Originally defined in [`{dialect_mod_ident}::enums::{enum_mod_ident}`](dialect::enums::{enum_ident})");

    syn::parse2(quote! {
        #![doc = #module_doc_comment]

        use super::super::super::#dialect_mod_ident as dialect;

        #[doc = #enum_doc_comment]
        pub type #enum_ident = dialect::enums::#enum_mod_ident::#enum_ident;
    })
    .unwrap()
}
