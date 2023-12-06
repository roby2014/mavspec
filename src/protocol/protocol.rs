use std::collections::HashMap;

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

use super::Dialect;

/// MAVLink protocol.
///
/// [`Protocol`] is a collection of MAVLink dialects.
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct Protocol {
    dialects: HashMap<String, Dialect>,
}

impl Protocol {
    /// Default constructor.
    pub fn new(dialects: HashMap<String, Dialect>) -> Self {
        Self { dialects }
    }

    /// Dialects within protocol.
    ///
    /// [`Dialect::name`] is used as a key.
    pub fn dialects(&self) -> &HashMap<String, Dialect> {
        &self.dialects
    }
}
