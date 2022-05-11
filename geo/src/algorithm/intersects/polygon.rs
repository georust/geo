use super::{has_disjoint_bboxes, Intersects};
use crate::utils::{coord_pos_relative_to_ring, CoordPos};
use crate::BoundingRect;
use crate::{
    CoordNum, Coordinate, GeoNum, Line, LineString, MultiLineString, MultiPolygon, Point, Polygon,
    Rect,
};

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
{
    fn intersects(&self, rect: &Rect<T>) -> bool {
        self.intersects(&rect.to_polygon())
    }
}
symmetric_intersects_impl!(Rect<T>, Polygon<T>);

impl<T> Intersects<Polygon<T>> for Polygon<T>
where
    T: GeoNum,
{
    fn intersects(&self, polygon: &Polygon<T>) -> bool {
        if has_disjoint_bboxes(self, polygon) {
            return false;
        }

        // self intersects (or contains) any line in polygon
        self.intersects(polygon.exterior()) ||
            polygon.interiors().iter().any(|inner_line_string| self.intersects(inner_line_string)) ||
            // self is contained inside polygon
            polygon.intersects(self.exterior())
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
