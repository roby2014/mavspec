use quote::quote;

pub(crate) fn make_serde_derive_annotation(enabled: bool) -> proc_macro2::TokenStream {
    if enabled {
        quote! {
            #[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
        }
    } else {
        quote!()
    }
}
