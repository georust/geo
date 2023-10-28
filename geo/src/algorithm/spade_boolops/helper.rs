use geo_types::{Point, Triangle};

use crate::triangulate_spade::SpadeTriangulationFloat;
use crate::{Centroid, Contains, Scale};

pub fn contains_triangle<P, T>(p: &P, tri: &Triangle<T>) -> bool
where
    P: Contains<Point<T>> + Scale<T>,
    T: SpadeTriangulationFloat,
{
    // this scaling is to prevent tiny triangles that can potentially occur on the outer boundary
    // of the poly to be included into the result
    p.scale(<T as From<f32>>::from(0.9999))
        .contains(&tri.centroid())
}
