use std::{cmp::Ordering, ops::Deref};

use geo_types::Coord;

use crate::GeoNum;

/// A lexicographically ordered point.
///
/// A wrapper around [`Coord`] to order the point by `x`, and then by `y`.
/// Implements `Ord` and `Eq`, allowing usage in ordered collections such as
/// `BinaryHeap`.
///
/// Note that the scalar type `T` is only required to implement `PartialOrd`.
/// Thus, it is a logical error to construct this struct unless the coords are
/// guaranteed to be orderable.
#[derive(PartialEq, Clone, Copy)]
pub struct SweepPoint<T: GeoNum>(Coord<T>);

impl<T: GeoNum> std::fmt::Debug for SweepPoint<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_tuple("SPt")
            .field(&self.0.x)
            .field(&self.0.y)
            .finish()
    }
}

/// Implement lexicographic ordering by `x` and then by `y`
/// coordinate.
impl<T: GeoNum> PartialOrd for SweepPoint<T> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

/// Derive `Ord` from `PartialOrd` and expect to not fail.
impl<T: GeoNum> Ord for SweepPoint<T> {
    fn cmp(&self, other: &Self) -> Ordering {
        match self.0.x.total_cmp(&other.0.x) {
            Ordering::Equal => self.0.y.total_cmp(&other.0.y),
            o => o,
        }
    }
}

/// We derive `Eq` manually to not require `T: Eq`.
impl<T: GeoNum> Eq for SweepPoint<T> {}

/// Conversion from type that can be converted to a `Coord`.
impl<T: GeoNum, X: Into<Coord<T>>> From<X> for SweepPoint<T> {
    fn from(pt: X) -> Self {
        SweepPoint(pt.into())
    }
}

impl<T: GeoNum> Deref for SweepPoint<T> {
    type Target = Coord<T>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

// Note: We keep it immutable for now, for better hygiene.
// impl<T: GeoNum> DerefMut for SweepPoint<T> {
//     fn deref_mut(&mut self) -> &mut Self::Target {
//         &mut self.0
//     }
// }

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sweep_point_ordering() {
        let p1 = SweepPoint::from(Coord { x: 0., y: 0. });
        let p2 = SweepPoint::from(Coord { x: 1., y: 0. });
        let p3 = SweepPoint::from(Coord { x: 1., y: 1. });
        let p4 = SweepPoint::from(Coord { x: 1., y: 1. });

        assert!(p1 < p2);
        assert!(p1 < p3);
        assert!(p2 < p3);
        assert!(p3 <= p4);
    }
}
