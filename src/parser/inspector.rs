use std::collections::HashMap;
use std::fs;
use std::path::Path;
use std::time::Instant;

use quick_xml::reader::Reader;

use crate::protocol::{Dialect, Protocol};

use super::definition::XmlDialectDefinition;
use super::errors::{XmlInspectionError, XmlParseError};
use super::xml::XmlParser;

/// Discovers and parses MAVLink XML definitions.
///
/// # Examples
///
/// Load dialects from `./message_definitions/standard` and get `HEARTBEAT_MESSAGE` from `minimal`
/// dialect:
///
/// ```rust
/// use mavspec::parser::XMLInspector;
///
/// // Instantiate inspector and load list of XML definitions
/// let inspector = XMLInspector::new(vec![
///     // Standard definitions from
///     // https://github.com/mavlink/mavlink/tree/master/message_definitions/v1.0
///     "./message_definitions/standard".to_string(),
///     // Extra definitions which depend on standard dialects
///     "./message_definitions/extra".to_string(),
/// ]).unwrap();
///
/// // Parse all XML definitions
/// let protocol = inspector.parse().unwrap();
///   
/// // Get `crazyflight` custom-defined dialect
/// let crazyflight = protocol.dialects().get("crazyflight").unwrap();
///
/// // Get `DUMMYFLIGHT_OUTCRY` message
/// let outcry_message = crazyflight.messages().get(&54000u32).unwrap();
/// assert_eq!(outcry_message.name(), "CRAZYFLIGHT_OUTCRY");
/// println!("\n`CRAZYFLIGHT_OUTCRY` message: {:#?}", outcry_message);
///
/// // Get `HEARTBEAT` message which custom dialect inherits from `standard` dialect
/// let heartbeat_message = crazyflight.messages().get(&0u32).unwrap();
/// assert_eq!(heartbeat_message.name(), "HEARTBEAT");
/// // Verify that `HEARTBEAT` message is defined in `minimal` dialect
/// assert_eq!(heartbeat_message.defined_in().as_deref().unwrap(), "minimal");
/// ```
#[derive(Debug)]
pub struct XMLInspector {
    sources: Vec<String>,
    definitions: Vec<XmlDialectDefinition>,
}

/// MAVLink protocol parser.
impl XMLInspector {
    /// Default constructor.
    ///
    /// # Arguments
    ///
    /// * `sources` - list of paths to message definition directories.
    ///
    /// # Examples
    ///
    /// See examples in [`XMLInspector`].
    pub fn new(sources: Vec<String>) -> Result<Self, XmlInspectionError> {
        let definitions = Self::discover_definitions(sources.as_slice())?;

        Ok(XMLInspector {
            sources,
            definitions,
        })
    }

    /// Returns paths to MAVLink message definition directories.
    pub fn src(&self) -> &[String] {
        &self.sources
    }

    /// Returns a list of dialect definitions or [`None`] if nothing was parsed.
    pub fn definitions(&self) -> &[XmlDialectDefinition] {
        &self.definitions
    }

    /// Discovers MAVLink dialects XML definitions within provided path.
    pub fn discover_definitions(
        paths: &[String],
    ) -> Result<Vec<XmlDialectDefinition>, XmlInspectionError> {
        let mut dialects: Vec<XmlDialectDefinition> = Vec::new();
        let mut dialect_ids: HashMap<String, String> = HashMap::new();

        for path in paths {
            let canonic = Path::new(&path).canonicalize()?;
            let path = canonic.to_str().unwrap();

            for entry in fs::read_dir(path)? {
                let entry_path = entry?.path();

                if entry_path.is_file()
                    && entry_path
                        .extension()
                        .unwrap()
                        .to_str()
                        .unwrap()
                        .to_lowercase()
                        .eq("xml")
                {
                    let path = entry_path.to_str().unwrap().to_string();
                    let definition = XmlDialectDefinition::new(&path);

                    // Check for naming collisions
                    #[allow(clippy::map_entry)]
                    if dialect_ids.contains_key(&definition.canonical_name()) {
                        return Err(XmlInspectionError::NamingCollision {
                            first: definition.name().to_string(),
                            second: dialect_ids
                                .get(&definition.canonical_name())
                                .unwrap()
                                .clone(),
                            canonical: definition.canonical_name(),
                        });
                    } else {
                        dialect_ids
                            .insert(definition.canonical_name(), definition.name().to_string());
                    }

                    dialects.push(definition);
                }
            }
        }

        Ok(dialects)
    }

    /// Parses XML definitions.
    pub fn parse(&self) -> Result<Protocol, XmlParseError> {
        let mut dialects: HashMap<String, Dialect> = HashMap::new();

        log::info!("Parsing dialects.");

        // Parsing started
        let started_at = Instant::now();

        // Iterate through definitions and parse dialects
        for def in &self.definitions {
            // Do nothing if this dialect has already been parsed
            if dialects.contains_key(&def.canonical_name()) {
                continue;
            }

            Self::parse_definition(def, &mut dialects)?;
        }

        // Calculate parsing duration
        let ended_at = Instant::now();
        let duration = ended_at - started_at;

        log::info!("All dialects parsed.");
        log::info!("Parsed dialects: {:?}", dialects.keys());
        log::info!(
            "Parse duration: {}s",
            (duration.as_micros() as f64) / 1000000.0
        );

        Ok(Protocol::new(dialects))
    }

    /// Parses MAVLink XML message definition.
    ///
    /// This function will update provided dialects map potentially loading and parsing dialect
    /// dependencies.
    ///
    /// If dialect has been already parsed, this function just returns parsed dialect.
    ///
    /// > Note that as dialect key we use its canonical name. Which means that dialects with the
    /// > same canonical names will cause collisions. See: [`XmlDialectDefinition::canonize_name`].
    ///
    /// # Arguments
    ///
    /// * `definition` - MAVLink XML definition.
    /// * `dialects` - mutable map for dialects.
    pub fn parse_definition<'a>(
        definition: &XmlDialectDefinition,
        dialects: &'a mut HashMap<String, Dialect>,
    ) -> Result<&'a Dialect, XmlParseError> {
        // Return already parsed dialect
        if dialects.contains_key(&definition.canonical_name()) {
            // The following `unwrap` is OK as we know that this dialect exists
            return Ok(dialects.get(&definition.canonical_name()).unwrap());
        }

        // Load all dependencies
        for dependency in definition.includes() {
            Self::parse_definition(dependency, dialects)?;
        }

        let mut enums = HashMap::new();
        let mut messages = HashMap::new();

        // Collect all enums from dependencies
        for dependency in definition.includes() {
            for (name, enm) in dialects.get(&dependency.canonical_name()).unwrap().enums() {
                enums.insert(name.clone(), enm.clone());
            }
        }

        // Collect all messages from dependencies
        for dependency in definition.includes() {
            for (&id, message) in dialects
                .get(&dependency.canonical_name())
                .unwrap()
                .messages()
            {
                messages.insert(id, message.clone());
            }
        }

        // Parsing started
        let started_at = Instant::now();

        // Parse dialect entries from definition
        let mut parser: XmlParser = XmlParser::new(&mut enums, &mut messages);
        let mut file_reader = Reader::from_file(definition.path()).unwrap();
        parser.parse(definition.name(), &mut file_reader)?;

        // Construct and save dialect
        dialects.insert(
            definition.canonical_name(),
            Dialect::new(
                definition.name().to_string(),
                definition.version(),
                definition.dialect(),
                messages,
                enums,
            ),
        );

        // Calculate parsing duration
        let ended_at = Instant::now();
        let duration = ended_at - started_at;

        // Report debug information:
        if log::log_enabled!(log::Level::Debug) {
            log::debug!("Parsed definition '{}'.", definition.name());
            log::debug!("Definition path: {}", definition.path());
            log::debug!("Definition version: {:?}", definition.version());
            log::debug!("Definition dialect #: {:?}", definition.dialect());
            log::debug!(
                "Parse duration: {}s",
                (duration.as_micros() as f64) / 1000000.0
            );
        }

        // The following `unwrap` is OK as we've just inserted this dialect
        Ok(dialects.get(&definition.canonical_name()).unwrap())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn default_dialect_paths() -> Vec<String> {
        vec![
            "./message_definitions/standard".to_string(),
            "./message_definitions/extra".to_string(),
        ]
    }

    /// Tests that MAVLink message definitions available ot default path
    #[test]
    fn dialects_are_available() {
        let parser = XMLInspector::new(default_dialect_paths()).unwrap();
        assert!(!parser.definitions().is_empty())
    }
}
