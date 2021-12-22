use super::{has_disjoint_bboxes, Intersects};
use crate::utils::{coord_pos_relative_to_ring, CoordPos};
use crate::BoundingRect;
use crate::{
    CoordNum, Coordinate, GeoNum, Line, LineString, MultiLineString, MultiPolygon, Point, Polygon,
    Rect,
};
use rstar::{RTree, RTreeObject};
// the largest total number of segments geometries can have before the algorithm switches
// to using an R*-tree for queries
const MAX_NAIVE_SEGMENTS: usize = 10000;

impl<T> Intersects<Coordinate<T>> for Polygon<T>
where
    T: GeoNum,
{
    fn intersects(&self, p: &Coordinate<T>) -> bool {
        coord_pos_relative_to_ring(*p, self.exterior()) != CoordPos::Outside
            && self
                .interiors()
                .iter()
                .all(|int| coord_pos_relative_to_ring(*p, int) != CoordPos::Inside)
    }
}
symmetric_intersects_impl!(Coordinate<T>, Polygon<T>);
symmetric_intersects_impl!(Polygon<T>, Point<T>);

impl<T> Intersects<Line<T>> for Polygon<T>
where
    T: GeoNum,
    geo_types::Line<T>: RTreeObject,
{
    fn intersects(&self, line: &Line<T>) -> bool {
        self.exterior().intersects(line)
            || self.interiors().iter().any(|inner| inner.intersects(line))
            || self.intersects(&line.start)
            || self.intersects(&line.end)
    }
}
symmetric_intersects_impl!(Line<T>, Polygon<T>);
symmetric_intersects_impl!(Polygon<T>, LineString<T>);
symmetric_intersects_impl!(Polygon<T>, MultiLineString<T>);

impl<T> Intersects<Rect<T>> for Polygon<T>
where
    T: GeoNum,
    geo_types::Line<T>: RTreeObject,
{
    fn intersects(&self, rect: &Rect<T>) -> bool {
        self.intersects(&rect.to_polygon())
    }
}
symmetric_intersects_impl!(Rect<T>, Polygon<T>);

impl<T> Intersects<Polygon<T>> for Polygon<T>
where
    T: GeoNum,
    geo_types::Line<T>: RTreeObject,
{
    fn intersects(&self, polygon: &Polygon<T>) -> bool {
        if has_disjoint_bboxes(self, polygon) {
            return false;
        }
        // switch to querying trees above some threshold x: polygons' combined segment count is higher than x
        if (self.exterior().0.len()
            + self.interiors().iter().map(|ls| ls.0.len()).sum::<usize>())
        *
        (polygon.exterior().0.len()
            + polygon
                .interiors()
                .iter()
                .map(|ls| ls.0.len())
                .sum::<usize>())
            > MAX_NAIVE_SEGMENTS
        {
            let lines_a: Vec<_> = self
                .exterior()
                .lines()
                .chain(self.interiors().iter().flat_map(|ls| ls.lines()))
                .collect();
            let tree_a = RTree::bulk_load(lines_a);

            let lines_b: Vec<_> = polygon
                .exterior()
                .lines()
                .chain(polygon.interiors().iter().flat_map(|ls| ls.lines()))
                .collect();
            let tree_b = RTree::bulk_load(lines_b);
            let mut candidates = tree_a.intersection_candidates_with_other_tree(&tree_b);
            candidates.any(|line_pairs| line_pairs.0.intersects(line_pairs.1))
        } else {
            self.intersects(polygon.exterior()) ||
            polygon.interiors().iter().any(|inner_line_string| self.intersects(inner_line_string)) ||
            // self is contained inside polygon
            polygon.intersects(self.exterior())
        }
    }
}

// Implementations for MultiPolygon

impl<G, T> Intersects<G> for MultiPolygon<T>
where
    T: GeoNum,
    Polygon<T>: Intersects<G>,
    G: BoundingRect<T>,
{
    fn intersects(&self, rhs: &G) -> bool {
        if has_disjoint_bboxes(self, rhs) {
            return false;
        }
        self.iter().any(|p| p.intersects(rhs))
    }
}

symmetric_intersects_impl!(Point<T>, MultiPolygon<T>);
symmetric_intersects_impl!(Line<T>, MultiPolygon<T>);
symmetric_intersects_impl!(Rect<T>, MultiPolygon<T>);
symmetric_intersects_impl!(Polygon<T>, MultiPolygon<T>);
