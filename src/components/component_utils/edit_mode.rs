#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum EditingMode<MainT, MemoT> {
    Editing(Option<MainT>, Vec<MemoT>),
    NoEdit,
}

impl<MainT, MemoT> Default for EditingMode<MainT, MemoT> {
    fn default() -> Self {
        Self::NoEdit
    }
}
