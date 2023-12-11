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
/// let inspector = XMLInspector::builder()
///     .set_sources(vec![
///         // Standard definitions from
///         // https://github.com/mavlink/mavlink/tree/master/message_definitions/v1.0
///         "./message_definitions/standard".to_string(),
///         // Extra definitions which depend on standard dialects
///         "./message_definitions/extra".to_string(),
///     ]).build()
///     .unwrap();
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
    definitions: Vec<XmlDialectDefinition>,
}

/// Configures [`XMLInspector`]
///
/// # Fields
///
/// ### [`sources`](XMLInspectorBuilder::set_sources)
///
/// List of paths to directories with `MAVLink` XML definitions.
///
/// ### [`include`](XMLInspectorBuilder::set_include) / [`exclude`](XMLInspectorBuilder::set_exclude)
///
/// These fields specify which dialects will be added. Exclusion/inclusion rules will be applied
/// only to the first level of dialects. Which means that if dialect `X` has dialect `Y` as a
/// dependency, then `Y` will be loaded and parsed even if explicitly excluded.  
///
/// If both [`include`](XMLInspectorBuilder::set_include) and [`XMLInspector`](XMLInspectorBuilder::set_exclude)
/// are set, then includes will have precedence over exclusion list. Which means that dialects will
/// be excluded within the list of explicitly specified dialects.
#[derive(Clone, Debug, Default)]
pub struct XMLInspectorBuilder {
    sources: Vec<String>,
    include: Option<Vec<String>>,
    exclude: Option<Vec<String>>,
}

impl XMLInspectorBuilder {
    /// Builds an instance of [`XMLInspector`].
    ///
    /// Loads a collection of [`XmlDialectDefinition`] in the process.
    ///
    /// # Errors
    ///
    /// Returns [`XmlInspectionError`] if XML definitions discovery fails.
    pub fn build(&self) -> Result<XMLInspector, XmlInspectionError> {
        let mut definitions = XMLInspector::discover_definitions(&self.sources)?;

        // If `include` is set, keep only explicitly specified dialects
        if let Some(names) = &self.include {
            log::info!("Only the following dialects will be included: {names:?}");
            definitions.retain(|def| names.contains(&def.name().to_string()));
        }

        // If `exclude` is set, remove excluded dialects
        if let Some(names) = &self.exclude {
            log::info!("The following dialects will be excluded: {names:?}");
            definitions.retain(|def| !names.contains(&def.name().to_string()));
        }

        Ok(XMLInspector { definitions })
    }

    /// Sets source paths.
    ///
    /// Each `sources` item is a path to a directory containing `MAVLink` XML definitions.
    pub fn set_sources(&mut self, sources: Vec<String>) -> &mut Self {
        self.sources = sources;
        self
    }

    /// Adds source path.
    ///
    /// Adds path to a directory containing `MAVLink` XML definitions.
    pub fn add_source(&mut self, source: String) -> &mut Self {
        self.sources.push(source);
        self
    }

    /// Sets the list of dialect names to include.
    pub fn set_include(&mut self, dialect_names: Vec<String>) -> &mut Self {
        self.include = Some(dialect_names);
        self
    }

    /// Sets the list of dialect names to exclude.
    pub fn set_exclude(&mut self, dialect_names: Vec<String>) -> &mut Self {
        self.exclude = Some(dialect_names);
        self
    }
}

/// MAVLink protocol parser.
impl XMLInspector {
    /// Instantiates builder for [`XMLInspector`].
    pub fn builder() -> XMLInspectorBuilder {
        XMLInspectorBuilder::default()
    }

    /// Returns a list of dialect definitions or [`None`] if nothing was parsed.
    pub fn definitions(&self) -> &[XmlDialectDefinition] {
        &self.definitions
    }

    /// Discovers MAVLink dialects XML definitions within provided paths.
    ///
    /// Prevents collisions between dialects with the same name (up to canonical form). See:
    /// [`canonize_name`](XmlDialectDefinition::canonize_name).
    ///
    /// # Errors
    ///
    /// Returns an variant of [`XmlInspectionError`].
    pub fn discover_definitions(
        paths: &[String],
    ) -> Result<Vec<XmlDialectDefinition>, XmlInspectionError> {
        // Found dialects
        let mut dialects: Vec<XmlDialectDefinition> = Vec::new();
        // A set of dialect IDs. New dialects with existing IDs will be rejected
        let mut dialect_ids: HashMap<String, String> = HashMap::new();

        // Iterate over paths
        for path in paths {
            let directory_path = Path::new(&path).canonicalize()?;
            log::debug!(
                "Entering XML definitions directory: {}",
                directory_path
                    .to_str()
                    .ok_or(XmlInspectionError::InvalidPath)?
            );

            // Iterate over files within directory
            for entry in fs::read_dir(directory_path)? {
                let entry_path = entry?.path();

                // Filter only XML files
                if entry_path.is_file()
                    && entry_path
                        .extension()
                        .unwrap_or_default()
                        .to_str()
                        .unwrap_or_default()
                        .to_lowercase()
                        .eq("xml")
                {
                    let path = entry_path
                        .to_str()
                        .ok_or(XmlInspectionError::InvalidPath)?
                        .to_string();
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

    /// Tests that MAVLink message definitions available ot default path.
    #[test]
    fn dialects_are_available() {
        let parser = XMLInspector::builder()
            .set_sources(default_dialect_paths())
            .build()
            .unwrap();

        assert!(!parser.definitions().is_empty());
    }

    /// Tests that inclusion rules works.
    #[test]
    fn inclusion_rules() {
        let parser = XMLInspector::builder()
            .set_sources(default_dialect_paths())
            .set_include(vec!["crazyflight".to_string()])
            .build()
            .unwrap();

        assert!(!parser.definitions().is_empty());
        assert_eq!(parser.definitions().len(), 1);
        assert_eq!(parser.definitions()[0].name(), "crazyflight");
    }

    /// Tests that exclusion rules works.
    #[test]
    fn exclusion_rules() {
        let parser = XMLInspector::builder()
            .set_sources(default_dialect_paths())
            .set_exclude(vec!["crazyflight".to_string()])
            .build()
            .unwrap();

        assert!(!parser.definitions().is_empty());

        for def in parser.definitions() {
            assert_ne!(def.name(), "crazyflight");
        }
    }
}
