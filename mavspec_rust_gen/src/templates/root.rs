use quote::quote;

/// Root module template.
pub fn root_module() -> syn::File {
    syn::parse2(quote! {
        // MAVLink protocol definition.
        //
        // Since this file is intended to be included with `include!`, we can not  provide module
        // documentation and leave this responsibility to the client.
        //
        // In most cases it makes sense to re-export `dialects` module:
        //
        // ```
        // mod mavlink {
        //     include!(concat!(env!("OUT_DIR"), "/mavlink/mod.rs"));
        // }
        // pub use mavlink::dialects;
        // ```
        //
        // Althought this is possible, we do not suggest to use `!include` without `mod` wrapper since
        // newer versions may introduce extra definitions that may interfere with your source code.

        // Import all dialects
        pub mod dialects;
    })
        .unwrap()
}
