use std::collections::HashMap;

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

use crate::protocol::traits::{Buildable, Builder};
use crate::protocol::{enums::EnumEntry, Deprecated};

/// Enum
///
/// MAVLink enum is a special field type. There are two types of enums:
/// - **regular**: value specifies a particular enum option (entry)
/// - **bitmask**: each bit signifies a particular flag
///
/// Enum options (entries) and flags are specified by [`EnumEntry`].
#[derive(Debug, Clone, Default)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct Enum {
    name: String,
    description: String,
    entries: HashMap<String, EnumEntry>,
    bitmask: bool,
    deprecated: Option<Deprecated>,
    defined_in: Vec<String>,
}

impl Buildable for Enum {
    type Builder = EnumBuilder;

    /// Creates [`EnumBuilder`] initialised with current enum entry.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use mavspec::protocol::Enum;
    /// use mavspec::protocol::traits::{Buildable, Builder};
    ///
    /// let original = Enum::builder()
    ///     .set_name("original".to_string())
    ///     .set_description("original".to_string())
    ///     .build();
    ///
    /// let updated = original.to_builder()
    ///     .set_description("updated".to_string())
    ///     .build();
    ///
    /// assert_eq!(updated.name(), "original");
    /// assert_eq!(updated.description(), "updated");
    /// ```
    fn to_builder(&self) -> EnumBuilder {
        EnumBuilder {
            r#enum: self.clone(),
        }
    }
}

impl Enum {
    /// Initiates builder.
    ///
    /// Instead of constructor we use
    /// [builder](https://rust-unofficial.github.io/patterns/patterns/creational/builder.html)
    /// pattern. An instance of [`EnumBuilder`] returned by this function is initialized
    /// with default values. Once desired values are set, you can call [`EnumBuilder::build`] to
    /// obtain [`Enum`].
    ///
    /// # Examples
    ///
    /// ```rust
    /// use mavspec::protocol::Enum;
    /// use mavspec::protocol::traits::Builder;
    ///
    /// let mav_enum = Enum::builder()
    ///     .set_name("name".to_string())
    ///     .set_description("description".to_string())
    ///     .build();
    ///
    /// assert!(matches!(mav_enum, Enum { .. }));
    /// assert_eq!(mav_enum.name(), "name");
    /// assert_eq!(mav_enum.description(), "description");
    /// ```
    pub fn builder() -> EnumBuilder {
        EnumBuilder::new()
    }

    /// Enum name.
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Enum description.
    pub fn description(&self) -> &str {
        &self.description
    }

    /// Collection of enum entries.
    pub fn entries(&self) -> &HashMap<String, EnumEntry> {
        &self.entries
    }

    /// Whether this enum is a bitmask.
    pub fn bitmask(&self) -> bool {
        self.bitmask
    }

    /// Deprecation status.
    pub fn deprecated(&self) -> Option<&Deprecated> {
        self.deprecated.as_ref()
    }

    /// Dialects in which this enum was defined.
    /// 
    /// The dialect list in order of inheritance. The former is a "parent" dialect to the latter.
    pub fn defined_in(&self) -> &[String] {
        &self.defined_in
    }
}

/// Builder for [`Enum`].
#[derive(Debug, Clone, Default)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct EnumBuilder {
    r#enum: Enum,
}

impl Builder for EnumBuilder {
    type Buildable = Enum;

    /// Creates [`Enum`] from builder.
    fn build(&self) -> Enum {
        // We need this to get error when `Enum` changes
        #[allow(clippy::match_single_binding)]
        match self.r#enum.clone() {
            Enum {
                name,
                description,
                entries,
                bitmask,
                deprecated,
                defined_in,
            } => Enum {
                name,
                description,
                entries,
                bitmask,
                deprecated,
                defined_in,
            },
        }
    }
}

impl EnumBuilder {
    /// Creates builder instance.
    ///
    /// Instantiates builder with default values for [`Enum`].
    pub fn new() -> Self {
        Self::default()
    }

    /// Sets enum name.
    ///
    /// See: [`Enum::name`].
    pub fn set_name(&mut self, name: String) -> &mut Self {
        self.r#enum.name = name;
        self
    }

    /// Sets enum description.
    ///
    /// See: [`Enum::description`].
    pub fn set_description(&mut self, description: String) -> &mut Self {
        self.r#enum.description = description;
        self
    }

    /// Sets collection of enum entries.
    ///
    /// See: [`Enum::entries`].
    pub fn set_entries(&mut self, entries: HashMap<String, EnumEntry>) -> &mut Self {
        self.r#enum.entries = entries;
        self
    }

    /// Sets whether this enum is a bitmask.
    ///
    /// See: [`Enum::bitmask`].
    pub fn set_bitmask(&mut self, bitmask: bool) -> &mut Self {
        self.r#enum.bitmask = bitmask;
        self
    }

    /// Sets deprecation status.
    ///
    /// See: [`Enum::deprecated`].
    pub fn set_deprecated(&mut self, deprecated: Option<Deprecated>) -> &mut Self {
        self.r#enum.deprecated = deprecated;
        self
    }

    /// Sets dialects in which this enum was defined.
    ///
    /// See: [`Enum::defined_in`].
    pub fn set_defined_in(&mut self, defined_in: Vec<String>) -> &mut Self {
        self.r#enum.defined_in = defined_in;
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::protocol::DeprecatedSince;

    #[test]
    fn enum_entry_builder() {
        let entries = {
            let mut entries = HashMap::new();
            entries.insert(
                "entry".to_string(),
                EnumEntry::builder().set_name("entry".to_string()).build(),
            );
            entries
        };

        let mav_enum = EnumBuilder::new()
            .set_name("name".to_string())
            .set_description("description".to_string())
            .set_entries(entries)
            .set_bitmask(true)
            .set_deprecated(Some(Deprecated::new(
                DeprecatedSince::default(),
                "better".to_string(),
            )))
            .set_defined_in(vec!["unknown".to_string()])
            .build();

        assert!(matches!(mav_enum, Enum { .. }));
        assert_eq!(mav_enum.name(), "name");
        assert_eq!(mav_enum.description(), "description");
        assert_eq!(mav_enum.entries().len(), 1);
        assert_eq!(mav_enum.entries().get("entry").unwrap().name(), "entry");
        assert!(mav_enum.bitmask);
        assert_eq!(mav_enum.deprecated().unwrap().replaced_by(), "better");
        assert_eq!(mav_enum.defined_in().len(), 1);
        assert_eq!(mav_enum.defined_in().get(0).unwrap(), "unknown");
    }
}
