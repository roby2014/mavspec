/// Enums module root template.
///
/// Input: [`mavspec::protocol::Dialect`].
pub const ENUMS_MODULE_ROOT: &str = "\
//! MAVLink enums of `{{name}}` dialect. 

/// MAVLink enums of `{{name}}` dialect. 
pub enum Enums {
{{#each enums}}
    /// Mavlink enum `{{name}}`.
    {{to-enum-rust-name name}},
{{/each}}
}
";
