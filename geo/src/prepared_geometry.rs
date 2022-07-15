use crate::geometry::{Geometry, Polygon};
use crate::relate::{IntersectionMatrix, Relate};
use crate::GeoFloat;

impl<F: GeoFloat> From<Polygon<F>> for PreparedGeometry<F> {
    fn from(polygon: Polygon<F>) -> Self {
        // TODO: build tree
        // TODO: from GeometryCoW?
        Self {
            geometry: Geometry::from(polygon),
        }
    }
}

#[derive(PartialEq, Debug, Hash)]
pub struct PreparedGeometry<F: GeoFloat = f64> {
    geometry: Geometry<F>,
}

impl<F: GeoFloat> Relate<F, PreparedGeometry<F>> for PreparedGeometry<F> {
    fn relate(&self, other: &PreparedGeometry<F>) -> IntersectionMatrix {
        todo!()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::polygon;

    #[test]
    fn relate() {
        let p1 = polygon![(x: 0.0, y: 0.0), (x: 2.0, y: 0.0), (x: 1.0, y: 1.0)];
        let p2 = polygon![(x: 0.5, y: 0.0), (x: 2.0, y: 0.0), (x: 1.0, y: 1.0)];
        let prepared_1 = PreparedGeometry::from(p1);
        let prepared_2 = PreparedGeometry::from(p2);
        assert!(prepared_1.relate(&prepared_2).is_contains())
    }
}
