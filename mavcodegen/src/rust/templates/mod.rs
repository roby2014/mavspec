pub mod dialects;

/// Root module template.
///
/// Input: [`mavspec::protocol::Protocol`].
pub const ROOT_MODULE: &str = "\
// MAVLink protocol definition.
// 
// Since this file is intended to be included with `include!`, we can not  provide module
// documentation and leave this responsibility to the client.

// Import all dialects
pub mod dialects;
";
