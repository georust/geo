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

    fn difference(p1: &Self, p2: &Self) -> SpadeBoolopsResult<T> {
        Self::boolop(p1, p2, |tri| {
            contains_triangle(p1, tri) && !contains_triangle(p2, tri)
        })
    }

    fn intersection(p1: &Self, p2: &Self) -> SpadeBoolopsResult<T> {
        Self::boolop(p1, p2, |tri| {
            contains_triangle(p1, tri) && contains_triangle(p2, tri)
        })
    }

    fn union(p1: &Self, p2: &Self) -> SpadeBoolopsResult<T> {
        Self::boolop(p1, p2, |tri| {
            contains_triangle(p1, tri) || contains_triangle(p2, tri)
        })
    }
}
