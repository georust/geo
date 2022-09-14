use super::{Active, ActiveSet};
use std::fmt::Debug;

/// A simple ordered set implementation backed by a `Vec`.
#[derive(Debug, Clone)]
pub struct VecSet<T: Ord> {
    data: Vec<T>,
}

impl<T: Ord> Default for VecSet<T> {
    fn default() -> Self {
        Self { data: Default::default() }
    }
}

impl<T: PartialOrd + Debug> VecSet<Active<T>> {
}

impl<T: PartialOrd + Debug> ActiveSet for VecSet<Active<T>> {
    type Seg = T;

    fn previous_find<F: FnMut(&Active<Self::Seg>) -> bool>(
        &self,
        segment: &Self::Seg,
        mut f: F,
    ) -> Option<&Active<Self::Seg>> {
        let segment = Active::active_ref(segment);
        let ub = match self.data.binary_search(segment) {
            Ok(i) => i,
            Err(i) => i,
        };
        self.data[..ub].iter().rev().find(|s| f(s))
    }

    fn next_find<F: FnMut(&Active<Self::Seg>) -> bool>(
        &self,
        segment: &Self::Seg,
        mut f: F,
    ) -> Option<&Active<Self::Seg>> {
        let segment = Active::active_ref(segment);
        let start = match self.data.binary_search(segment) {
            Ok(i) => i + 1,
            Err(i) => i,
        };
        self.data[start..].iter().find(|s| f(s))
    }

    fn insert_active(&mut self, segment: Self::Seg) {
        let idx = {
            let segment = Active::active_ref(&segment);
            self.data.binary_search(segment).expect_err("element already in active-vec-set")
        };
        self.data.insert(idx, Active(segment));
    }

    fn remove_active(&mut self, segment: &Self::Seg) {
        let segment = Active::active_ref(segment);
        let idx = self.data.binary_search(segment).expect("element not found in active-vec-set");
        self.data.remove(idx);
    }
}
