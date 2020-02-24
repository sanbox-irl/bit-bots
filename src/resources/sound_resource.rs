use strum_macros::{EnumCount, EnumIter, EnumString};

#[derive(
    Debug,
    PartialEq,
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
pub enum SoundResource {
    Back,
    Cursor,
}

impl SoundResource {
    pub(super) fn get_sound_file(&self) -> &'static [u8] {
        match self {
            Self::Back => include_bytes!("../../assets/audio/old_files/back.ogg"),
            Self::Cursor => include_bytes!("../../assets/audio/old_files/cursor.ogg"),
        }
    }
}
