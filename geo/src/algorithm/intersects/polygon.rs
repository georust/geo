use super::{has_disjoint_bboxes, Intersects};
use crate::coordinate_position::CoordPos;
use crate::geometry::*;
use crate::{BoundingRect, CoordinatePosition, CoordsIter, LinesIter};
use crate::{CoordNum, GeoNum};

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

        // if there are no line intersections among exteriors and interiors,
        // then either one fully contains the other
        // or they are disjoint

        // check 1 point of each polygon being within the other
        self.exterior().coords_iter().take(1).any(|p|polygon.intersects(&p))
        || polygon.exterior().coords_iter().take(1).any(|p|self.intersects(&p))
        // exterior exterior
        || self.exterior().lines_iter().any(|self_line| polygon.exterior().lines_iter().any(|poly_line| self_line.intersects(&poly_line)))
        // exterior interior
        ||self.interiors().iter().any(|inner_line_string| polygon.exterior().intersects(inner_line_string))
        ||polygon.interiors().iter().any(|inner_line_string| self.exterior().intersects(inner_line_string))

        // interior interior (not needed)
        /*
           suppose interior-interior is a required check
           this requires that there are no ext-ext intersections
           and that there are no ext-int intersections
           and that self-ext[0] not intersects other
           and other-ext[0] not intersects self
           and there is some intersection between self and other

           if ext-ext disjoint, then one ext ring must be within the other ext ring

           suppose self-ext is within other-ext and self-ext[0] is not intersects other
           then self-ext[0] must be within an interior hole of other-ext
           if self-ext does not intersect the interior ring which contains self-ext[0],
           then self is contained within other interior hole
           and hence self and other cannot intersect
           therefore for self to intersect other, some part of the self-ext must intersect the other-int ring
           However, this is a contradiction because one of the premises for requiring this check is that self-ext ring does not intersect any other-int ring

           By symmetry, the mirror case of other-ext ring within self-ext ring is also true

           therefore, if there cannot exist and int-int intersection when all the prior checks are false
           and so we can skip the interior-interior check
        */
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

macro_rules! intersects_multipolygon_impl {
    ( [$($target:ty),*]) => {
        $(
            impl<T> Intersects<$target> for MultiPolygon<T>
            where
                T: GeoNum,
                $target: BoundingRect<T>,
            {
                fn intersects(&self, rhs: &$target) -> bool {
                    if has_disjoint_bboxes(self, rhs) {
                        return false;
                    }
                    self.iter().any(|p| p.intersects(rhs))
                }
            }
        )*
    };
}

impl<T> Intersects<Coord<T>> for MultiPolygon<T>
where
    T: GeoNum,
{
    fn intersects(&self, rhs: &Coord<T>) -> bool {
        if has_disjoint_bboxes(self, rhs) {
            return false;
        }
        self.coordinate_position(rhs) !=  CoordPos::Outside
    }
}

impl<T> Intersects<Point<T>> for MultiPolygon<T>
where
    T: GeoNum,
{
    fn intersects(&self, rhs: &Point<T>) -> bool {
        self.intersects(&rhs.0)
    }
}

intersects_multipolygon_impl!([MultiPoint<T>]);
intersects_multipolygon_impl!([Line<T>, LineString<T>, MultiLineString<T>]);
intersects_multipolygon_impl!([Polygon<T>, MultiPolygon<T>, Rect<T>, Triangle<T>]);
intersects_multipolygon_impl!([Geometry<T>, GeometryCollection<T>]);

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
