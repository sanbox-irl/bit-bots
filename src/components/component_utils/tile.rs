#[derive(Debug, Clone, Copy, Default, PartialEq, Serialize, Deserialize, typename::TypeName)]
#[serde(default)]
pub struct Tile {
    pub index: usize,
}

impl From<usize> for Tile {
    fn from(o: usize) -> Tile {
        Tile { index: o }
    }
}

impl From<Tile> for usize {
    fn from(o: Tile) -> usize {
        o.index
    }
}
