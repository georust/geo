use crate::geometry::{Geometry, Polygon};
use crate::GeoFloat;
use crate::GeometryCow;

impl<'a, F: GeoFloat> From<&'a Polygon<F>> for PreparedGeometry<'a, F> {
    fn from(polygon: &'a Polygon<F>) -> Self {
        // TODO: build tree
        // TODO: from GeometryCoW?
        Self {
            geometry: GeometryCow::from(polygon),
        }
    }
}

#[derive(PartialEq, Debug, Hash)]
pub struct PreparedGeometry<'a, F: GeoFloat = f64> {
    geometry: GeometryCow<'a, F>,
}

impl<'a, F: GeoFloat> PreparedGeometry<'a, F> {
    pub(crate) fn geometry(&self) -> &GeometryCow<'a, F> {
        &self.geometry
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::algorithm::Relate;
    use crate::polygon;

    #[test]
    fn relate() {
        let p1 = polygon![(x: 0.0, y: 0.0), (x: 2.0, y: 0.0), (x: 1.0, y: 1.0)];
        let p2 = polygon![(x: 0.5, y: 0.0), (x: 2.0, y: 0.0), (x: 1.0, y: 1.0)];
        let prepared_1 = PreparedGeometry::from(&p1);
        let prepared_2 = PreparedGeometry::from(&p2);
        assert!(prepared_1.relate(&prepared_2).is_contains())
    }
}
