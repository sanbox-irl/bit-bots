#[derive(
    Debug,
    PartialEq,
    Ord,
    PartialOrd,
    Eq,
    Copy,
    Clone,
    Hash,
    strum_macros::Display,
    strum_macros::EnumIter,
    strum_macros::EnumString,
    strum_macros::EnumCount,
    Serialize,
    Deserialize,
    typename::TypeName,
)]
pub enum SpriteName {
    WhitePixel,
    Background,
    PixelMainCharacterStanding,
    PlayerDead,
    Grass,
    Block,
    Fire,
    Flag,
    PushableBlock,
    Wall,
    Button,
    Target,
}
impl Default for SpriteName {
    fn default() -> Self {
        Self::WhitePixel
    }
}

impl SpriteName {
    pub fn better_display(&self) -> String {
        format!("{:?}", self)
    }
}
