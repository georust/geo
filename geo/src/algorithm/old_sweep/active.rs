use std::{borrow::Borrow, cmp::Ordering, fmt::Debug, ops::Deref};

/// A segment currently active in the sweep.
///
/// As the sweep-line progresses from left to right, it intersects a subset of
/// the line-segments. These can be totally-ordered from bottom to top, and
/// efficient access to the neighbors of a segment is a key aspect of
/// planar-sweep algorithms.
///
/// We assert `Ord` even though the inner-type is typically only `T:
/// PartialOrd`. It is a logical error to compare two Active which cannot be
/// compared. This is ensured by the algorithm (and cannot be inferred by the
/// compiler?).
#[derive(Debug, Clone, Copy, PartialEq)]
#[repr(transparent)]
pub(in crate::algorithm) struct Active<T>(pub(in crate::algorithm) T);

impl<T> Active<T> {
    pub(in crate::algorithm) fn active_ref(t: &T) -> &Active<T> {
        unsafe { std::mem::transmute(t) }
    }
}

impl<T> Borrow<T> for Active<T> {
    fn borrow(&self) -> &T {
        &self.0
    }
}

impl<T> Deref for Active<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

/// Assert total equality.
impl<T: PartialEq> Eq for Active<T> {}

/// Assert total ordering of active segments.
impl<T: PartialOrd + Debug> Ord for Active<T> {
    fn cmp(&self, other: &Self) -> Ordering {
        if let Some(c) = T::partial_cmp(self, other) {
            c
        } else {
            warn!("could not compare segments:\n\t{self:?}\n\t{other:?}");
            panic!("unable to compare active segments!");
        }
        // T::partial_cmp(self, other).unwrap()
        // T::partial_cmp(self, other).unwrap()
    }
}

impl<T: PartialOrd + Debug> PartialOrd for Active<T> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

/// Trait abstracting a container of active segments.
#[allow(dead_code)]
pub(in crate::algorithm) trait ActiveSet: Default {
    type Seg;
    fn previous_find<F: FnMut(&Active<Self::Seg>) -> bool>(
        &self,
        segment: &Self::Seg,
        f: F,
    ) -> Option<&Active<Self::Seg>>;
    fn previous(&self, segment: &Self::Seg) -> Option<&Active<Self::Seg>> {
        self.previous_find(segment, |_| true)
    }
    fn next_find<F: FnMut(&Active<Self::Seg>) -> bool>(
        &self,
        segment: &Self::Seg,
        f: F,
    ) -> Option<&Active<Self::Seg>>;
    fn next(&self, segment: &Self::Seg) -> Option<&Active<Self::Seg>> {
        self.next_find(segment, |_| true)
    }
    fn insert_active(&mut self, segment: Self::Seg);
    fn remove_active(&mut self, segment: &Self::Seg);
}
