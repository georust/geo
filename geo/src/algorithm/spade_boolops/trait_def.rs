pub use crate::spade_boolops::error::{SpadeBoolopsError, SpadeBoolopsResult};
pub use crate::spade_boolops::helper::contains_triangle;
use geo_types::{Point, Triangle};

use crate::triangulate_spade::SpadeTriangulationFloat;
use crate::{Contains, Scale};

/// Boolean Operations on geometry.
///
/// Boolean operations are set operations on geometries considered as a
/// subset of the 2-D plane. The operations supported are: intersection,
/// union and set-difference on pairs of 2-D geometries.
///
/// These operations are implemented on any existing geo type that
/// implements [`crate::LinesIter`] and [`crate::CoordsIter`] as well
/// as [`crate::Contains`] for a [`crate::Point`]. Further, if the
/// operations exist for a type T, they also exist for Vec<T> and &[T]
///
/// # Performance
///
/// Note that the algorithm is based on a somewhat non-trivial Delaunay
/// Triangulation which can create performance hits if you use the
/// operations on huge geometries (~ 1000 vertices per geometry).
/// On a smaller scale, the algorithm works reasonably fast and can even
/// be used for some real-time applications
pub trait SpadeBoolops<T>
where
    T: SpadeTriangulationFloat,
    Self: Contains<Point<T>> + Scale<T> + Sized,
{
    fn boolop<F: Fn(&Triangle<T>) -> bool>(
        p1: &Self,
        p2: &Self,
        op_pred: F,
    ) -> SpadeBoolopsResult<T>;

    #[must_use = "Use the difference of these two geometries by binding the result to a variable! (`let result = ...`)"]
    fn difference(p1: &Self, p2: &Self) -> SpadeBoolopsResult<T> {
        Self::boolop(p1, p2, |tri| {
            contains_triangle(p1, tri) && !contains_triangle(p2, tri)
        })
    }

    #[must_use = "Use the intersection of these two geometries by binding the result to a variable! (`let result = ...`)"]
    fn intersection(p1: &Self, p2: &Self) -> SpadeBoolopsResult<T> {
        Self::boolop(p1, p2, |tri| {
            contains_triangle(p1, tri) && contains_triangle(p2, tri)
        })
    }

    #[must_use = "Use the union of these two geometries by binding the result to a variable! (`let result = ...`)"]
    fn union(p1: &Self, p2: &Self) -> SpadeBoolopsResult<T> {
        Self::boolop(p1, p2, |tri| {
            contains_triangle(p1, tri) || contains_triangle(p2, tri)
        })
    }
}
