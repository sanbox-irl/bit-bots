#[derive(Debug, Default, Clone, Serialize, Deserialize, PartialEq, Hash)]
pub struct FragmentedData<T> {
    pub relative_path: String,
    marker: std::marker::PhantomData<T>,
}

impl<T> FragmentedData<T> {
    pub fn new(relative_path: String) -> FragmentedData<T> {
        FragmentedData {
            relative_path,
            marker: std::marker::PhantomData,
        }
    }
}
