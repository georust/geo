use super::Segment;
use crate::geometry::Polygon;
use crate::relate::geomgraph::GeometryGraph;
use crate::GeoFloat;
use crate::GeometryCow;
use rstar::RTree;

// TODO: other types
impl<'a, F: GeoFloat> From<&'a Polygon<F>> for PreparedGeometry<'a, F> {
    fn from(polygon: &'a Polygon<F>) -> Self {
        // TODO: build tree
        let tree: RTree<Segment<F>> = todo!();
        Self {
            geometry: GeometryCow::from(polygon),
            tree,
        }
    }
}

#[derive(Debug)]
pub struct PreparedGeometry<'a, F: GeoFloat = f64> {
    geometry: GeometryCow<'a, F>,
    tree: RTree<Segment<F>>,
}

impl<'a, F: GeoFloat> PreparedGeometry<'a, F> {
    pub(crate) fn geometry_graph(&'a self, arg_index: usize) -> GeometryGraph<'a, F> {
        GeometryGraph::new(
            arg_index,
            &self.geometry,
            GeometryGraph::create_unprepared_edge_set_intersector(),
        )
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
