use strum_macros::Display;

#[derive(Debug, Serialize, Display, Deserialize, Copy, Clone, PartialEq, Eq, Hash)]
pub enum Marker {
    Camera,
    ExemptFromGrid,
}
