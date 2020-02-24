use strum_macros::EnumIter;

#[derive(Debug, Clone, PartialEq, Serialize, EnumIter, Deserialize, typename::TypeName)]
pub enum Approach {
    Instant,
    Linear(f32),
    Asymptotic(f32),
}

impl Default for Approach {
    fn default() -> Self {
        Self::Instant
    }
}
