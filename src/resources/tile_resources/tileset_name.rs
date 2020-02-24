use strum_macros::{EnumCount, EnumIter, EnumString};

#[derive(
    Debug,
    PartialEq,
    Ord,
    PartialOrd,
    Eq,
    Copy,
    Clone,
    Hash,
    EnumIter,
    EnumString,
    EnumCount,
    Serialize,
    Deserialize,
    typename::TypeName,
)]
#[strum(serialize_all = "snake_case")]
pub enum TileSetName {
    Default,
    Walls,
    Floors,
}

impl Default for TileSetName {
    fn default() -> Self {
        Self::Default
    }
}

use std::fmt;
impl fmt::Display for TileSetName {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}
