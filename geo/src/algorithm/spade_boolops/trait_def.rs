pub use crate::spade_boolops::error::{SpadeBoolopsError, SpadeBoolopsResult};
pub use crate::spade_boolops::helper::contains_triangle;
use geo_types::{Point, Triangle};

use crate::triangulate_spade::SpadeTriangulationFloat;
use crate::{Contains, Scale};

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
