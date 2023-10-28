use geo_types::{MultiPolygon, Point, Polygon, Triangle};

use crate::stitch::LineStitchingError;
use crate::triangulate_spade::{SpadeTriangulationFloat, TriangulationError};
use crate::{Centroid, Contains, Scale, Stitch, TriangulateSpade};

// ====== Error ========

#[derive(Debug)]
pub enum SpadeBoolopsError {
    TriangulationError(TriangulationError),
    StitchError(LineStitchingError),
}

impl std::fmt::Display for SpadeBoolopsError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl std::error::Error for SpadeBoolopsError {}

pub type SpadeBoolopsResult<T> = Result<MultiPolygon<T>, SpadeBoolopsError>;

// ======== Spade Boolops trait ============

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

// ========== trait impls ==========

impl<T> SpadeBoolops<T> for Polygon<T>
where
    T: SpadeTriangulationFloat,
{
    fn boolop<F: Fn(&Triangle<T>) -> bool>(
        p1: &Self,
        p2: &Self,
        op_pred: F,
    ) -> SpadeBoolopsResult<T> {
        vec![p1.clone(), p2.clone()]
            .unconstrained_triangulation()
            .map_err(SpadeBoolopsError::TriangulationError)?
            .into_iter()
            .filter(|tri| op_pred(tri))
            .map(|tri| tri.to_polygon())
            .collect::<Vec<_>>()
            .stitch_together()
            .map_err(SpadeBoolopsError::StitchError)
    }
}

impl<T> SpadeBoolops<T> for MultiPolygon<T>
where
    T: SpadeTriangulationFloat,
{
    fn boolop<F: Fn(&Triangle<T>) -> bool>(
        p1: &Self,
        p2: &Self,
        op_pred: F,
    ) -> SpadeBoolopsResult<T> {
        vec![p1.clone(), p2.clone()]
            .unconstrained_triangulation()
            .map_err(SpadeBoolopsError::TriangulationError)?
            .into_iter()
            .filter(|tri| op_pred(tri))
            .map(|tri| tri.to_polygon())
            .collect::<Vec<_>>()
            .stitch_together()
            .map_err(SpadeBoolopsError::StitchError)
    }
}

// ========== Helper =========

fn contains_triangle<P, T>(p: &P, tri: &Triangle<T>) -> bool
where
    P: Contains<Point<T>> + Scale<T>,
    T: SpadeTriangulationFloat,
{
    // this scaling is to prevent tiny triangles that can potentially occur on the outer boundary
    // of the poly to be included into the result
    p.scale(<T as From<f32>>::from(0.9999))
        .contains(&tri.centroid())
}

#[cfg(test)]
mod spade_boolops_tests {
    use super::*;
    use geo_types::*;

    #[test]
    fn basic_intersection_compiles() {
        let zero = Coord::zero();
        let one = Coord { x: 1.0, y: 1.0 };
        let rect1 = Rect::new(zero, one * 2.0);
        let rect2 = Rect::new(one, one * 3.0);

        SpadeBoolops::intersection(&rect1.to_polygon(), &rect2.to_polygon()).unwrap();
    }
}
