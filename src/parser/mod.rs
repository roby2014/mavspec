//! MAVLink protocol parser.
//!
//!

mod inspector;
pub use inspector::XMLInspector;

mod definition;
pub use definition::XmlDialectDefinition;

mod xml;
pub use xml::XmlParser;

/// Errors related to XML entities parsing
pub mod errors;
