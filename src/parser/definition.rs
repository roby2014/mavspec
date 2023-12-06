use std::path::Path;

use heck::AsSnakeCase;
use quick_xml::events::Event;
use quick_xml::reader::Reader;

/// MAVLink dialect XML definition.
#[derive(Debug, Clone)]
pub struct XmlDialectDefinition {
    name: String,
    path: String,
    includes: Vec<Self>,
    version: Option<u8>, // TODO: change to `u32`
    dialect: Option<u8>, // TODO: change to `u32`
}

impl PartialEq for XmlDialectDefinition {
    fn eq(&self, other: &Self) -> bool {
        self.name == other.name
    }
}

impl XmlDialectDefinition {
    /// Loads XML dialect definition from provided path.
    ///
    /// # Arguments
    ///
    /// * `path` - path to XML definition file.
    pub fn new(path: &str) -> Self {
        Self::load_from_path(path)
    }

    /// Dialect name according to its file name.
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Canonical name of the dialect, its unique identifier.
    ///
    /// See: [`XmlDialectDefinition::canonize_name`]
    pub fn canonical_name(&self) -> String {
        Self::canonize_name(&self.name)
    }

    /// Path to dialect XML definition.
    pub fn path(&self) -> &str {
        &self.path
    }

    /// Version if specified.
    pub fn version(&self) -> Option<u8> {
        self.version
    }

    /// MAVLink dialect if specified.
    pub fn dialect(&self) -> Option<u8> {
        self.dialect
    }

    /// List of XML definition included to this dialect.
    pub fn includes(&self) -> &[Self] {
        &self.includes
    }

    /// Converts dialect `name` to its canonical name (`ID`).
    ///
    /// Dialects with the same canonical name will be considered the same.
    ///
    /// As a dialect `ID` we use its `snake_case` version of its XML definition file basename
    /// (without extension). Which means that `MyDialect`, `my_dialect`, and `my-dialect` will be
    /// considered the same and have a canonical name `my_dialect`. At the same time, `mydialect`
    /// will be considered as a different dialect.
    pub fn canonize_name(name: &str) -> String {
        AsSnakeCase(name).to_string()
    }

    fn load_from_path(path: &str) -> Self {
        let name = Path::new(path)
            .file_stem()
            .unwrap()
            .to_str()
            .unwrap()
            .to_string();
        let base_path = Path::new(path).parent();
        let mut reader = Reader::from_file(path).unwrap();
        let mut buf = Vec::new();
        let mut tag_stack: Vec<String> = Vec::new();

        let mut includes_content: Vec<String> = Vec::new();
        let mut version: Option<u8> = None;
        let mut dialect: Option<u8> = None;

        // Parse includes, version and dialect
        loop {
            match reader.read_event_into(&mut buf) {
                Err(e) => panic!("Error at position {}: {:?}", reader.buffer_position(), e),
                // exits the loop when reaching end of file
                Ok(Event::Eof) => break,
                Ok(Event::Start(e)) => {
                    let tag_name = String::from_utf8_lossy(e.as_ref()).to_string();
                    tag_stack.push(tag_name.clone());

                    // Stop scanning dialect definition once content sections reached
                    match tag_name.as_ref() {
                        // Exit once enums reached
                        "enums" => break,
                        // Exit as messages reached
                        "messages" => break,
                        &_ => {}
                    }
                }
                Ok(Event::Text(t)) if !tag_stack.is_empty() => {
                    let data = String::from_utf8_lossy(t.as_ref()).to_string();
                    let last_open_tag = tag_stack.last().unwrap().as_ref();

                    match last_open_tag {
                        "include" => includes_content.push(data),
                        "version" => version = Some(data.parse::<u8>().unwrap()),
                        "dialect" => dialect = Some(data.parse::<u8>().unwrap()),
                        &_ => {}
                    }
                }
                Ok(Event::End(e)) => {
                    let tag_name = String::from_utf8_lossy(e.as_ref()).to_string();
                    let last_open_tag = tag_stack.last().unwrap().to_string();

                    if !tag_stack.is_empty() && last_open_tag == tag_name {
                        tag_stack.pop();
                    } else {
                        panic!("Invalid closing tag '{tag_name}' after '{last_open_tag}'!")
                    }
                }
                // There are several other `Event`s we do not consider here
                _ => (),
            }
        }

        // Load includes
        let includes: Vec<Self> = includes_content
            .iter()
            .map(|filepath| -> Self {
                let included_path = base_path
                    .unwrap()
                    .join(Path::new(filepath))
                    .to_str()
                    .unwrap()
                    .to_string();

                Self::load_from_path(&included_path)
            })
            .collect();

        Self {
            name,
            path: path.to_string(),
            includes,
            version,
            dialect,
        }
    }
}
