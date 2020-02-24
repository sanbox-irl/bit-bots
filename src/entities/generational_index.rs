#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash, Serialize, Default, Deserialize)]
pub struct GenerationalIndex {
    pub(super) index: usize,
    pub(super) generation: u64,
}

impl GenerationalIndex {
    pub fn index(&self) -> usize {
        self.index
    }
}

impl std::fmt::Display for GenerationalIndex {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Entity: [Index: {}, Generation: {}]",
            self.index, self.generation
        )
    }
}

use std::cmp::Ordering;
impl Ord for GenerationalIndex {
    fn cmp(&self, other: &Self) -> Ordering {
        self.generation
            .cmp(&other.generation)
            .then(self.index.cmp(&other.index))
    }
}

impl PartialOrd for GenerationalIndex {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

pub struct AllocatorEntry {
    is_live: bool,
    generation: u64,
}

impl AllocatorEntry {
    fn update(&mut self) -> u64 {
        self.is_live = true;
        self.generation += 1;
        self.generation
    }
}

pub struct GenerationalIndexAllocator {
    entries: Vec<AllocatorEntry>,
    free: Vec<usize>,
}

impl GenerationalIndexAllocator {
    pub fn new() -> Self {
        Self {
            entries: Vec::new(),
            free: Vec::new(),
        }
    }

    pub fn allocate(&mut self) -> GenerationalIndex {
        match self.free.pop() {
            Some(i) => {
                let new_generation = self.entries[i].update();

                GenerationalIndex {
                    index: i,
                    generation: new_generation,
                }
            }

            None => {
                let index = self.entries.len();

                self.entries.push(AllocatorEntry {
                    is_live: true,
                    generation: 0,
                });

                GenerationalIndex { index, generation: 0 }
            }
        }
    }

    /// Returns true if the index was allocated, and is now
    /// deallocated.
    #[allow(dead_code)]
    pub fn deallocate(&mut self, index: &GenerationalIndex) -> bool {
        let entry = &mut self.entries[index.index];
        if entry.is_live == false {
            false
        } else {
            entry.is_live = false;
            self.free.push(index.index);

            true
        }
    }

    #[allow(dead_code)]
    pub fn is_live(&self, index: GenerationalIndex) -> bool {
        if index.index >= self.entries.len() {
            false
        } else {
            self.entries[index.index].is_live
        }
    }
}
