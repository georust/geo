use crate::{GeoFloat, Point};

/// The result of trying to find the closest spot on an object to a point.
#[cfg_attr(feature = "use-serde", derive(Serialize, Deserialize))]
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Closest<F: GeoFloat> {
    /// The point actually intersects with the object.
    Intersection(Point<F>),
    /// There is exactly one place on this object which is closest to the point.
    SinglePoint(Point<F>),
    /// There are two or more (possibly infinite or undefined) possible points.
    Indeterminate,
}

impl<F: GeoFloat> Closest<F> {
    /// Compare two `Closest`s relative to `p` and return a copy of the best
    /// one.
    pub fn best_of_two(&self, other: &Self, p: Point<F>) -> Self {
        use crate::algorithm::euclidean_distance::EuclideanDistance;

        let left = match *self {
            Closest::Indeterminate => return *other,
            Closest::Intersection(_) => return *self,
            Closest::SinglePoint(l) => l,
        };
        let right = match *other {
            Closest::Indeterminate => return *self,
            Closest::Intersection(_) => return *other,
            Closest::SinglePoint(r) => r,
        };

        if left.euclidean_distance(&p) <= right.euclidean_distance(&p) {
            *self
        } else {
            *other
        }
    }
}
