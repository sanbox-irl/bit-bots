use std::marker::PhantomData;

pub struct GuardedRwLock<T, K>(T, PhantomData<K>);

impl<T, K> GuardedRwLock<T, K> {
    pub fn new(t: T) -> Self {
        Self(t, PhantomData)
    }

    pub fn read(&self) -> &T {
        &self.0
    }

    pub fn read_mut(&self, _: K) -> &mut T {
        &mut self.0
    }
}