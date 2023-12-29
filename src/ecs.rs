use core::hash::Hash;
use core::num::NonZeroU32;

#[derive(Default, Clone)]
pub struct EntityAllocator {
    pub next: u32,
    pub free: Vec<Entity>,
}

impl EntityAllocator {
    pub fn alloc(&mut self) -> Entity {
        if let Some(entity) = self.free.pop() {
            entity
        } else {
            let entity = Entity {
                index: self.next,
                version: NonZeroU32::MIN,
            };
            self.next += 1;
            entity
        }
    }

    pub fn dealloc(&mut self, entity: Entity) {
        self.free.push(Entity {
            index: entity.index,
            version: match NonZeroU32::new(entity.version.get() + 1) {
                None => NonZeroU32::MIN,
                Some(x) => x,
            },
        });
    }

    pub fn clear(&mut self) {
        self.next = 0;
        self.free.clear();
    }
}

#[repr(align(8))]
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Entity {
    version: NonZeroU32,
    index: u32,
}

impl Hash for Entity {
    #[inline]
    fn hash<H: core::hash::Hasher>(&self, state: &mut H) {
        let x = unsafe { core::mem::transmute::<Self, u64>(*self) };
        x.hash(state);
    }
}

impl Entity {
    #[inline]
    pub fn version(self) -> NonZeroU32 {
        self.version
    }

    #[inline]
    pub fn index(self) -> u32 {
        self.index
    }
}

#[derive(Clone)]
pub struct Comp<T> {
    pub entities: Vec<Entity>,
    pub components: Vec<T>,
    pub sparse: Vec<Option<Entity>>,
}

impl<T> Default for Comp<T> {
    #[inline]
    fn default() -> Self {
        Self::new()
    }
}

impl<T> core::ops::Index<Entity> for Comp<T> {
    type Output = T;

    #[inline]
    fn index(&self, index: Entity) -> &Self::Output {
        unsafe { self.get_unchecked(index) }
    }
}

impl<T> core::ops::IndexMut<Entity> for Comp<T> {
    #[inline]
    fn index_mut(&mut self, index: Entity) -> &mut Self::Output {
        unsafe { self.get_unchecked_mut(index) }
    }
}

impl<T> Comp<T> {
    #[inline]
    pub const fn new() -> Self {
        Self {
            entities: Vec::new(),
            components: Vec::new(),
            sparse: Vec::new(),
        }
    }

    pub fn len(&self) -> usize {
        self.entities.len()
    }

    pub fn is_empty(&self) -> bool {
        self.entities.is_empty()
    }

    #[inline]
    pub fn get(&self, entity: Entity) -> Option<&T> {
        let index = (*self.sparse.get(entity.index as usize)?)?;
        if index.version == entity.version {
            Some(unsafe { self.components.get_unchecked(index.index as usize) })
        } else {
            None
        }
    }

    #[allow(clippy::missing_safety_doc)]
    #[inline]
    #[cfg(not(debug_assertions))]
    pub unsafe fn get_unchecked(&self, entity: Entity) -> &T {
        self.components.get_unchecked(
            self.sparse
                .get_unchecked(entity.index as usize)
                .unwrap_unchecked()
                .index as usize,
        )
    }

    #[allow(clippy::missing_safety_doc)]
    #[inline]
    #[cfg(debug_assertions)]
    pub unsafe fn get_unchecked(&self, entity: Entity) -> &T {
        self.get(entity).unwrap()
    }

    #[allow(clippy::missing_safety_doc)]
    #[inline]
    #[cfg(not(debug_assertions))]
    pub unsafe fn get_unchecked_mut(&mut self, entity: Entity) -> &mut T {
        self.components.get_unchecked_mut(
            self.sparse
                .get_unchecked(entity.index as usize)
                .unwrap_unchecked()
                .index as usize,
        )
    }

    #[allow(clippy::missing_safety_doc)]
    #[inline]
    #[cfg(debug_assertions)]
    pub unsafe fn get_unchecked_mut(&mut self, entity: Entity) -> &mut T {
        self.get_mut(entity).unwrap()
    }

    #[inline]
    pub fn get_mut(&mut self, entity: Entity) -> Option<&mut T> {
        let index = (*self.sparse.get(entity.index as usize)?)?;
        if index.version == entity.version {
            Some(unsafe { self.components.get_unchecked_mut(index.index as usize) })
        } else {
            None
        }
    }

    pub fn insert(&mut self, e: Entity, component: T) -> Option<T> {
        let len = self.len();

        let index = e.index as usize;
        if index >= self.sparse.len() {
            let extra_len =
                index.checked_next_power_of_two().unwrap_or(index) - self.entities.len() + 1;

            self.sparse.resize(self.sparse.len() + extra_len, None);
        }

        let dense_entity = unsafe { self.sparse.get_unchecked_mut(index) };

        if let Some(dense_entity) = dense_entity.as_mut() {
            unsafe {
                dense_entity.version = e.version;
                let index = dense_entity.index as usize;

                *self.entities.get_unchecked_mut(index) = e;
                Some(core::mem::replace(
                    self.components.get_unchecked_mut(index),
                    component,
                ))
            }
        } else {
            *dense_entity = Some(Entity {
                index: len as u32,
                version: e.version,
            });

            self.entities.push(e);
            self.components.push(component);
            None
        }
    }

    pub fn remove(&mut self, e: Entity) -> Option<T> {
        let x = match self.sparse.get_mut(e.index as usize) {
            Some(y) => match *y {
                Some(x) => {
                    if x.version == e.version {
                        *y = None;
                        x
                    } else {
                        return None;
                    }
                }
                None => return None,
            },
            _ => return None,
        };

        let index = x.index as usize;
        let last_entity = unsafe { *self.entities.get_unchecked(self.entities.len() - 1) };

        let _entity = self.entities.swap_remove(index);
        let comp = self.components.swap_remove(index);
        debug_assert_eq!(_entity, e);

        if index < self.entities.len() {
            let e = Entity {
                index: index as u32,
                version: last_entity.version,
            };
            unsafe {
                *self.sparse.get_unchecked_mut(last_entity.index as usize) = Some(e);
            }
        }

        Some(comp)
    }
}

impl<'a, T> IntoIterator for &'a Comp<T> {
    type Item = (Entity, &'a T);
    type IntoIter = Iter<'a, T>;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        Iter { comp: self, pos: 0 }
    }
}

pub struct Iter<'a, T> {
    comp: &'a Comp<T>,
    pos: usize,
}

impl<'a, T> IntoIterator for &'a mut Comp<T> {
    type Item = (Entity, &'a mut T);
    type IntoIter = IterMut<'a, T>;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        IterMut { comp: self, pos: 0 }
    }
}

pub struct IterMut<'a, T> {
    comp: &'a mut Comp<T>,
    pos: usize,
}

impl<'a, T> Iterator for Iter<'a, T> {
    type Item = (Entity, &'a T);

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        if self.pos == self.comp.len() {
            None
        } else {
            unsafe {
                let e = *self.comp.entities.get_unchecked(self.pos);
                let s = self.comp.sparse.get_unchecked(e.index as usize);
                let s = s.unwrap_unchecked().index as usize;
                self.pos += 1;
                Some((e, &*self.comp.components.as_ptr().add(s)))
            }
        }
    }
}

impl<'a, T> Iterator for IterMut<'a, T> {
    type Item = (Entity, &'a mut T);

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        if self.pos == self.comp.len() {
            None
        } else {
            unsafe {
                let e = *self.comp.entities.get_unchecked(self.pos);
                let s = self.comp.sparse.get_unchecked(e.index as usize);
                let s = s.unwrap_unchecked().index as usize;
                self.pos += 1;
                Some((e, &mut *self.comp.components.as_mut_ptr().add(s)))
            }
        }
    }
}

impl<T> ExactSizeIterator for Iter<'_, T> {
    #[inline]
    fn len(&self) -> usize {
        self.comp.len() - self.pos
    }
}

impl<T> ExactSizeIterator for IterMut<'_, T> {
    #[inline]
    fn len(&self) -> usize {
        self.comp.len() - self.pos
    }
}
