//! Serializable description of a parsed BAT strategy.
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct Strategy {
    pub name: String,
    pub file_name: String,
    pub raw_args: String,
}
