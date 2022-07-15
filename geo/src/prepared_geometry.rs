use crate::geometry::Polygon;
use crate::relate::{IntersectionMatrix, Relate};
use crate::{GeoFloat, GeoNum};
use std::marker::PhantomData;

pub struct PreparedPolygon<F: GeoFloat = f64> {
    _marker: PhantomData<F>,
}

impl<F: GeoFloat> From<Polygon<F>> for PreparedPolygon<F> {
    fn from(_: Polygon<F>) -> Self {
        todo!()
    }
}

trait PreparedGeometry<F: GeoFloat = f64> {}

impl<F: GeoFloat> PreparedGeometry<F> for PreparedPolygon<F> {}

impl<F, G1, G2> Relate<F, G2> for G1
where
    G1: PreparedGeometry<F>,
    F: GeoFloat,
    G2: PreparedGeometry<F>,
{
    fn relate(&self, other: &G2) -> IntersectionMatrix {
        todo!()
    }
}
