use super::{has_disjoint_bboxes, Intersects};
use crate::coordinate_position::CoordPos;
use crate::{BoundingRect, CoordinatePosition, CoordsIter, LinesIter};
use crate::{
    Coord, CoordNum, GeoNum, Line, LineString, MultiLineString, MultiPoint, MultiPolygon, Point,
    Polygon, Rect, Triangle,
};

impl<T> Intersects<Coord<T>> for Polygon<T>
where
    T: GeoNum,
{
    fn intersects(&self, p: &Coord<T>) -> bool {
        self.coordinate_position(p) != CoordPos::Outside
    }
}

symmetric_intersects_impl!(Polygon<T>, LineString<T>);
symmetric_intersects_impl!(Polygon<T>, MultiLineString<T>);

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

symmetric_intersects_impl!(Polygon<T>, Point<T>);
symmetric_intersects_impl!(Polygon<T>, MultiPoint<T>);

impl<T> Intersects<Polygon<T>> for Polygon<T>
where
    T: GeoNum,
{
    fn intersects(&self, polygon: &Polygon<T>) -> bool {
        if has_disjoint_bboxes(self, polygon) {
            return false;
        }

        // if there are no line intersections,
        // then either one fully contains the other
        // or they are disjoint

        // exterior exterior
        // using lines_iter() skips the bounds check for ls intersections gives a ~33% speedup
        // we already know that exteriors are not disjoint
        // and avoid recalcuating bounds of self.exterior() for every linestring-line itersects check
        self.exterior().lines_iter().any(|line| polygon.exterior().lines_iter().any(|other_line| line.intersects(&other_line)))

        // exterior inner
        || self.interiors().iter().any(|inner_line_string| polygon.exterior().lines_iter().any(|other_line| inner_line_string.intersects(&other_line)))
        || polygon.interiors().iter().any(|inner_line_string| self.exterior().lines_iter().any(|other_line| inner_line_string.intersects(&other_line)))

        // inner inner 
        || self.interiors().iter().any(|inner_line_string| polygon.interiors().iter().any(|other_line_string| inner_line_string.intersects(other_line_string)))

        // check 1 point of each polygon being within the other
        || self.exterior().coords_iter().take(1).any(|p|polygon.intersects(&p))
        || polygon.exterior().coords_iter().take(1).any(|p|self.intersects(&p))
    }
}

symmetric_intersects_impl!(Polygon<T>, MultiPolygon<T>);

impl<T> Intersects<Rect<T>> for Polygon<T>
where
    T: GeoNum,
{
    fn intersects(&self, rect: &Rect<T>) -> bool {
        self.intersects(&rect.to_polygon())
    }
}

impl<T> Intersects<Triangle<T>> for Polygon<T>
where
    T: GeoNum,
{
    fn intersects(&self, rect: &Triangle<T>) -> bool {
        self.intersects(&rect.to_polygon())
    }
}

// Blanket implementation for MultiPolygon
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

#[cfg(test)]
mod tests {
    use crate::*;
    #[test]
    fn geom_intersects_geom() {
        let a = Geometry::<f64>::from(polygon![]);
        let b = Geometry::from(polygon![]);
        assert!(!a.intersects(&b));
    }
}
