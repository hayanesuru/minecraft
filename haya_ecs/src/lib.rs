#![no_std]

extern crate alloc;

use alloc::collections::VecDeque;
use alloc::vec::Vec;

#[derive(Debug, Clone)]
pub struct EntityAllocator {
    pub next: u32,
    pub free: VecDeque<Entity>,
}

impl EntityAllocator {
    pub fn alloc(&mut self) -> Entity {
        if let Some(entity) = self.free.pop_front() {
            entity
        } else {
            let entity = Entity { index: self.next };
            self.next += 1;
            entity
        }
    }

    pub fn dealloc(&mut self, entity: Entity) {
        self.free.push_back(entity);
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug, Hash, Default)]
#[must_use]
pub struct Entity {
    pub index: u32,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Dense {
    index: u32,
}

impl Dense {
    const NONE: Self = Self { index: u32::MAX };

    #[inline]
    const fn is_none(self) -> bool {
        self.index == u32::MAX
    }

    #[inline]
    pub const fn index(self) -> usize {
        self.index as usize
    }
}

impl From<Dense> for usize {
    #[inline]
    fn from(value: Dense) -> Self {
        value.index as usize
    }
}

#[derive(Clone, Debug, Default)]
pub struct SparseSet {
    entities: Vec<Entity>,
    sparse: Vec<Dense>,
}

impl SparseSet {
    #[inline]
    pub const fn new() -> Self {
        Self {
            entities: Vec::new(),
            sparse: Vec::new(),
        }
    }

    #[inline]
    pub const fn len(&self) -> usize {
        self.entities.len()
    }

    #[inline]
    pub const fn is_empty(&self) -> bool {
        self.entities.is_empty()
    }

    #[inline]
    pub const fn capacity(&self) -> usize {
        self.entities.capacity()
    }

    #[inline]
    pub fn contains(&self, entity: Entity) -> bool {
        self.sparse
            .get(entity.index as usize)
            .map(|x| !x.is_none())
            .unwrap_or(false)
    }

    #[inline]
    pub fn get(&self, entity: Entity) -> Dense {
        match self.sparse.get(entity.index as usize).copied() {
            Some(x) => x,
            None => Dense::NONE,
        }
    }

    #[inline]
    pub fn insert(&mut self, entity: Entity) -> Dense {
        let sparse = entity.index as usize;
        if sparse >= self.sparse.len() {
            let new_len = sparse + 1;
            self.sparse.resize(
                new_len.checked_next_power_of_two().unwrap_or(new_len),
                Dense::NONE,
            );
        }
        let dense_ref = unsafe { self.sparse.get_unchecked_mut(sparse) };
        if dense_ref.is_none() {
            let dense = Dense {
                index: self.entities.len() as u32,
            };
            self.entities.push(entity);
            *dense_ref = dense;
            dense
        } else {
            Dense::NONE
        }
    }

    #[inline]
    pub fn remove(&mut self, entity: Entity) -> Dense {
        let sparse = entity.index;
        let dense = match self.sparse.get_mut(sparse as usize) {
            Some(x) => x,
            None => return Dense::NONE,
        };
        if dense.is_none() {
            return Dense::NONE;
        }
        let dense_index = dense.index;
        *dense = Dense::NONE;
        let last_sparse = unsafe { self.entities.get_unchecked(self.entities.len() - 1).index };
        let _ = self.entities.swap_remove(dense_index as usize);
        if !self.entities.is_empty() {
            unsafe {
                *self.sparse.get_unchecked_mut(last_sparse as usize) = Dense { index: dense_index };
            }
        }
        Dense { index: dense_index }
    }

    #[inline]
    pub fn shrink_to_fit(&mut self) {
        self.entities.shrink_to_fit();
        self.sparse.shrink_to_fit();
    }

    #[inline]
    pub fn reserve(&mut self, additional: usize) {
        self.entities.reserve(additional);
        self.sparse.reserve(additional);
    }

    #[inline]
    pub fn clear(&mut self) {
        self.entities.clear();
        self.sparse.clear();
    }

    #[inline]
    pub fn entities(&self) -> &[Entity] {
        &self.entities
    }
}

#[derive(Clone, Debug, Default)]
pub struct Component<T>(pub Vec<T>);

impl<T> Component<T> {
    #[inline]
    pub fn new() -> Self {
        Self(Vec::new())
    }

    #[inline]
    pub fn insert(&mut self, dense: usize, value: T) -> Option<T> {
        if dense == self.0.len() {
            self.0.push(value);
            None
        } else {
            unsafe { Some(core::mem::replace(self.0.get_unchecked_mut(dense), value)) }
        }
    }

    #[inline]
    pub fn remove(&mut self, dense: usize) {
        self.0.swap_remove(dense);
    }

    #[inline]
    pub fn clear(&mut self) {
        self.0.clear();
    }

    #[inline]
    pub fn len(&self) -> usize {
        self.0.len()
    }

    #[inline]
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    #[inline]
    pub fn components(&self) -> &[T] {
        &self.0
    }

    #[inline]
    pub fn components_mut(&mut self) -> &mut [T] {
        &mut self.0
    }

    #[inline]
    pub fn get(&self, dense: usize) -> Option<&T> {
        self.0.get(dense)
    }

    #[inline]
    pub fn get_mut(&mut self, dense: usize) -> Option<&mut T> {
        self.0.get_mut(dense)
    }

    #[inline]
    pub fn iter<'a>(&'a self, sparse: &'a SparseSet) -> Iter<'a, T> {
        Iter {
            comp: self.0.iter(),
            sparse: sparse.entities().iter(),
        }
    }

    #[inline]
    pub fn iter_mut<'a>(&'a mut self, sparse: &'a SparseSet) -> IterMut<'a, T> {
        IterMut {
            comp: self.0.iter_mut(),
            sparse: sparse.entities().iter(),
        }
    }
}

#[must_use]
#[derive(Debug)]
pub struct Iter<'a, T> {
    comp: core::slice::Iter<'a, T>,
    sparse: core::slice::Iter<'a, Entity>,
}

#[must_use]
#[derive(Debug)]
pub struct IterMut<'a, T> {
    comp: core::slice::IterMut<'a, T>,
    sparse: core::slice::Iter<'a, Entity>,
}

impl<'a, T> Iterator for IterMut<'a, T> {
    type Item = (&'a mut T, Entity);

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        self.comp
            .next()
            .map(|x| unsafe { (x, self.sparse.next().copied().unwrap_unchecked()) })
    }
}

impl<'a, T> DoubleEndedIterator for IterMut<'a, T> {
    #[inline]
    fn next_back(&mut self) -> Option<Self::Item> {
        self.comp
            .next_back()
            .map(|x| unsafe { (x, self.sparse.next_back().copied().unwrap_unchecked()) })
    }
}

impl<'a, T> Iterator for Iter<'a, T> {
    type Item = (&'a T, Entity);

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        self.comp
            .next()
            .map(|x| unsafe { (x, self.sparse.next().copied().unwrap_unchecked()) })
    }
}

impl<'a, T> DoubleEndedIterator for Iter<'a, T> {
    #[inline]
    fn next_back(&mut self) -> Option<Self::Item> {
        self.comp
            .next_back()
            .map(|x| unsafe { (x, self.sparse.next_back().copied().unwrap_unchecked()) })
    }
}

impl<'a, T> ExactSizeIterator for Iter<'a, T> {
    #[inline]
    fn len(&self) -> usize {
        self.comp.len()
    }
}

impl<'a, T> ExactSizeIterator for IterMut<'a, T> {
    #[inline]
    fn len(&self) -> usize {
        self.comp.len()
    }
}
