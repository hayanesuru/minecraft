use core::cmp::Ordering;
use core::num::NonZeroUsize;
use std::collections::BinaryHeap;

#[derive(Clone)]
pub struct KdTreeQueue<N, T, const D: usize>(
    BinaryHeap<HeapElement<N, *const KdTree<N, T, D>>>,
    BinaryHeap<HeapElement<N, T>>,
);

impl<N: Num, T, const D: usize> Default for KdTreeQueue<N, T, D> {
    #[inline]
    fn default() -> Self {
        Self(BinaryHeap::new(), BinaryHeap::new())
    }
}

impl<N: Num, T, const D: usize> KdTreeQueue<N, T, D> {
    #[inline]
    pub fn pop(&mut self) -> Option<(N, T)> {
        match self.1.pop() {
            Some(x) => Some((x.distance, x.element)),
            None => None,
        }
    }
}

#[derive(Clone, Copy)]
struct HeapElement<N, T> {
    pub distance: N,
    pub element: T,
}

impl<A: PartialEq, T> PartialEq for HeapElement<A, T> {
    #[inline]
    fn eq(&self, other: &Self) -> bool {
        self.distance.eq(&other.distance)
    }
}

impl<A: PartialEq, T> PartialEq<A> for HeapElement<A, T> {
    #[inline]
    fn eq(&self, other: &A) -> bool {
        self.distance.eq(other)
    }
}

impl<A: PartialEq, T> Eq for HeapElement<A, T> {}

impl<A: PartialOrd, T> PartialOrd for HeapElement<A, T> {
    #[inline]
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl<A: PartialOrd, T> Ord for HeapElement<A, T> {
    #[inline]
    fn cmp(&self, other: &Self) -> Ordering {
        match self.distance.partial_cmp(&other.distance) {
            None => Ordering::Equal,
            Some(x) => x,
        }
    }
}

pub trait Num: Sized + Copy + PartialEq + PartialOrd {
    const MAX: Self;
    const MIN: Self;
    const NEG_ONE: Self;
    const ZERO: Self;

    fn distance<const L: usize>(a: &[Self; L], b: &[Self; L]) -> Self;
    fn distance_to_space<const L: usize>(
        p1: &[Self; L],
        min_bounds: &[Self; L],
        max_bounds: &[Self; L],
    ) -> Self;
    fn diff_max(self, min: Self, max: Self) -> Self;
    fn avg2(self, small: Self) -> Self;
    fn eq_arr<const L: usize>(x: &[Self; L], y: &[Self; L]) -> bool;
    fn min(self, other: Self) -> Self;
    fn mul(self, x: Self) -> Self;
}

impl Num for f64 {
    const MAX: Self = Self::INFINITY;
    const MIN: Self = Self::NEG_INFINITY;
    const NEG_ONE: Self = -1.0;
    const ZERO: Self = 0.0;

    #[inline]
    fn distance<const L: usize>(a: &[Self; L], b: &[Self; L]) -> Self {
        let mut x = 0f64;
        for i in 0..L {
            unsafe {
                let a = *a.get_unchecked(i);
                let b = *b.get_unchecked(i);
                x += (a - b) * (a - b);
            }
        }
        x
    }

    fn distance_to_space<const L: usize>(
        p1: &[Self; L],
        min_bounds: &[Self; L],
        max_bounds: &[Self; L],
    ) -> Self {
        let mut x = 0.0;
        for i in 0..L {
            let a = unsafe { *p1.get_unchecked(i) };
            let b = if a > max_bounds[i] {
                unsafe { a - *max_bounds.get_unchecked(i) }
            } else if a < min_bounds[i] {
                unsafe { a - *min_bounds.get_unchecked(i) }
            } else {
                0.0
            };
            x += b * b;
        }
        x
    }

    #[inline]
    fn diff_max(self, min: Self, max: Self) -> Self {
        let diff = max - min;
        if !diff.is_nan() && diff > max {
            diff
        } else {
            self
        }
    }

    #[inline]
    fn avg2(self, small: Self) -> Self {
        small + (self - small) / 2.0
    }

    #[inline]
    fn eq_arr<const L: usize>(x: &[Self; L], y: &[Self; L]) -> bool {
        x == y
    }

    #[inline]
    fn min(self, other: Self) -> Self {
        self.min(other)
    }

    #[inline]
    fn mul(self, x: Self) -> Self {
        self * x
    }
}

#[derive(Clone)]
pub struct KdTree<N, T, const D: usize> {
    // node
    left: Option<Box<KdTree<N, T, D>>>,
    right: Option<Box<KdTree<N, T, D>>>,
    // common
    capacity: NonZeroUsize,
    size: usize,
    min_bounds: [N; D],
    max_bounds: [N; D],
    // stem
    split_value: Option<N>,
    split_dimension: Option<usize>,
    // leaf
    points: Option<Vec<[N; D]>>,
    bucket: Option<Vec<T>>,
}

impl<N: Num, T: Clone, const D: usize> Default for KdTree<N, T, D> {
    #[inline]
    fn default() -> Self {
        Self::new()
    }
}

impl<N: Num, T: Clone, const D: usize> KdTree<N, T, D> {
    #[inline]
    pub const fn new() -> Self {
        unsafe { KdTree::with_capacity(NonZeroUsize::new_unchecked(16)) }
    }

    #[inline]
    pub const fn with_capacity(capacity: NonZeroUsize) -> Self {
        let min_bounds = [N::MAX; D];
        let max_bounds = [N::MIN; D];
        Self {
            left: None,
            right: None,
            capacity,
            size: 0,
            min_bounds,
            max_bounds,
            split_value: None,
            split_dimension: None,
            points: Some(Vec::new()),
            bucket: Some(Vec::new()),
        }
    }

    #[inline]
    #[must_use]
    fn is_leaf(&self) -> bool {
        self.bucket.is_some()
            && self.points.is_some()
            && self.split_value.is_none()
            && self.split_dimension.is_none()
            && self.left.is_none()
            && self.right.is_none()
    }

    fn split(&mut self, mut points: Vec<[N; D]>, mut bucket: Vec<T>) {
        let mut max = N::ZERO;
        for dim in 0..D {
            let max_b = unsafe { *self.max_bounds.get_unchecked(dim) };
            let min_b = unsafe { *self.min_bounds.get_unchecked(dim) };
            let max2 = max.diff_max(min_b, max_b);
            if max2 != max {
                self.split_dimension = Some(dim);
                max = max2;
            }
        }
        match self.split_dimension {
            None => {
                self.points = Some(points);
                self.bucket = Some(bucket);
                return;
            }
            Some(dim) => unsafe {
                let min = *self.min_bounds.get_unchecked(dim);
                let max = *self.max_bounds.get_unchecked(dim);
                self.split_value = Some(max.avg2(min));
            },
        };
        let mut left = Box::new(Self::with_capacity(self.capacity));
        let mut right = Box::new(Self::with_capacity(self.capacity));
        while !points.is_empty() {
            let point = points.swap_remove(0);
            let data = bucket.swap_remove(0);
            if self.belongs_in_left(&point) {
                left.add_to_bucket(point, data);
            } else {
                right.add_to_bucket(point, data);
            }
        }
        self.left = Some(left);
        self.right = Some(right);
    }

    pub fn insert(mut self: &mut Self, point: [N; D], data: T) -> bool {
        loop {
            if self.is_leaf() {
                self.add_to_bucket(point, data);
                return true;
            }
            self.extend(&point);
            self.size += 1;
            let next = if self.belongs_in_left(point.as_ref()) {
                self.left.as_mut()
            } else {
                self.right.as_mut()
            };
            self = next.unwrap();
        }
    }

    fn add_to_bucket(&mut self, point: [N; D], data: T) {
        self.extend(&point);
        let mut points = self.points.take().unwrap();
        let mut bucket = self.bucket.take().unwrap();
        points.push(point);
        bucket.push(data);
        self.size += 1;
        if self.size > self.capacity.get() {
            self.split(points, bucket);
        } else {
            self.points = Some(points);
            self.bucket = Some(bucket);
        }
    }

    pub fn remove(&mut self, point: [N; D], e: &T) -> usize
    where
        T: Eq,
    {
        let mut removed = 0usize;

        if let (Some(points), Some(bucket)) = (self.points.as_mut(), self.bucket.as_mut()) {
            if let Some(p_index) = points
                .iter()
                .enumerate()
                .position(|(idx, x)| N::eq_arr(x, &point) && e.eq(&bucket[idx]))
            {
                points.remove(p_index);
                bucket.remove(p_index);
                removed += 1;
                self.size -= 1;
            }
        } else {
            let r_or_l = match self.right.as_mut() {
                None => self.left.as_mut(),
                Some(x) => Some(x),
            };
            if let Some(r_or_l) = r_or_l {
                let r_or_l_removed = r_or_l.remove(point, e);
                if r_or_l_removed > 0 {
                    self.size -= r_or_l_removed;
                    removed += r_or_l_removed;
                }
            }
        }
        removed
    }

    fn extend(&mut self, point: &[N; D]) {
        for x in 0..D {
            unsafe {
                let min = self.min_bounds.get_unchecked_mut(x);
                let max = self.max_bounds.get_unchecked_mut(x);
                let val = *point.get_unchecked(x);

                if val < *min {
                    *min = val;
                }
                if val > *max {
                    *max = val;
                }
            }
        }
    }

    fn belongs_in_left(&self, point: &[N]) -> bool {
        let dim = unsafe { self.split_dimension.unwrap_unchecked() };
        let val = unsafe { self.split_value.unwrap_unchecked() };
        if self.min_bounds[dim] == val {
            point[dim] <= val
        } else {
            point[dim] < val
        }
    }

    pub fn nearest(&self, point: &[N; D], num: usize, queue: &mut KdTreeQueue<N, T, D>) {
        let pending = &mut queue.0;
        let evaluated = &mut queue.1;
        pending.clear();
        evaluated.clear();

        let num = num.min(self.size);
        if num == 0 {
            return;
        }

        pending.push(HeapElement {
            distance: N::ZERO,
            element: self,
        });
        while !pending.is_empty()
            && (evaluated.len() < num
                || (pending.peek().unwrap().distance.mul(N::NEG_ONE)
                    <= evaluated.peek().unwrap().distance))
        {
            self.nearest_step(point, num, N::MAX, pending, evaluated);
        }
    }

    fn nearest_step(
        &self,
        point: &[N; D],
        num: usize,
        max_dist: N,
        pending: &mut BinaryHeap<HeapElement<N, *const Self>>,
        evaluated: &mut BinaryHeap<HeapElement<N, T>>,
    ) {
        let mut curr = unsafe { &*pending.pop().unwrap().element };
        debug_assert!(evaluated.len() <= num);
        let evaluated_dist = if evaluated.len() == num {
            // We only care about the nearest `num` points, so if we already have `num` points,
            // any more point we add to `evaluated` must be nearer then one of the point already in
            // `evaluated`.
            max_dist.min(evaluated.peek().unwrap().distance)
        } else {
            max_dist
        };

        while !curr.is_leaf() {
            let candidate;
            if curr.belongs_in_left(point) {
                candidate = curr.right.as_ref().unwrap();
                curr = curr.left.as_ref().unwrap();
            } else {
                candidate = curr.left.as_ref().unwrap();
                curr = curr.right.as_ref().unwrap();
            }
            let candidate_to_space =
                N::distance_to_space(point, &candidate.min_bounds, &candidate.max_bounds);
            if candidate_to_space <= evaluated_dist {
                pending.push(HeapElement {
                    distance: candidate_to_space.mul(N::NEG_ONE),
                    element: &**candidate,
                });
            }
        }

        let points = curr.points.as_ref().unwrap().iter();
        let bucket = curr.bucket.as_ref().unwrap().iter();
        let iter = points.zip(bucket).map(|(p, d)| HeapElement {
            distance: N::distance(point, p),
            element: d.clone(),
        });
        for element in iter {
            if element.distance.le(&max_dist) {
                if evaluated.len() < num {
                    evaluated.push(element);
                } else if element < *evaluated.peek().unwrap() {
                    evaluated.pop();
                    evaluated.push(element);
                }
            }
        }
    }

    pub fn within(&self, point: &[N; D], radius: N, queue: &mut KdTreeQueue<N, T, D>) {
        let pending = &mut queue.0;
        let evaluated = &mut queue.1;

        pending.clear();
        evaluated.clear();

        if self.size == 0 {
            return;
        }

        pending.push(HeapElement {
            distance: N::ZERO,
            element: self,
        });
        while !pending.is_empty() && (pending.peek().unwrap().distance.mul(N::NEG_ONE) <= radius) {
            self.nearest_step(point, self.size, radius, pending, evaluated);
        }
    }
}
