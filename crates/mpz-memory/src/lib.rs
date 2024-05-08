//! Memory abstractions used in `mpz`.

pub mod repr;

/// A memory store.
pub trait Memory<T> {
    /// An identifier for a value in memory.
    type Id;

    /// Gets a value from memory if it exists.
    fn get(&self, id: &Self::Id) -> Option<&T>;

    /// Allocates a value in memory.
    fn alloc(&mut self, value: T) -> Self::Id;
}

impl<T> Memory<T> for Vec<T> {
    type Id = usize;

    fn get(&self, id: &Self::Id) -> Option<&T> {
        self.as_slice().get(*id)
    }

    fn alloc(&mut self, value: T) -> Self::Id {
        let id = self.len();
        self.push(value);
        id
    }
}

/// A mutable memory store.
pub trait MemoryMut<T>: Memory<T> {
    /// Sets a value in memory.
    fn set(&mut self, id: &Self::Id, value: T);
}

impl<T> MemoryMut<T> for Vec<T> {
    fn set(&mut self, id: &Self::Id, value: T) {
        self[*id] = value;
    }
}
