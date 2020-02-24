#[derive(Debug, Serialize, Deserialize, Copy, Clone, PartialEq, Eq, Hash, Default)]
pub struct CachedBool {
    pub value: bool,
    dirty: bool,
}

impl CachedBool {
    pub fn new(value: bool) -> Self {
        Self { value, dirty: false }
    }

    pub fn is_dirty(&self) -> bool {
        self.dirty
    }

    pub fn clean(&mut self) {
        self.dirty = false;
    }

    pub fn inspect(&mut self, ui: &mut imgui::Ui<'_>, label: &str) {
        ui.checkbox(&imgui::ImString::new(label), &mut self.value);
    }
}
