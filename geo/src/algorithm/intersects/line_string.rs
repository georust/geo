use super::{has_disjoint_bboxes, Intersects};
use crate::BoundingRect;
use crate::*;

// Blanket implementation using self.lines().any().
macro_rules! blanket_intersects_linestring {
    ($t:ty) => {
        impl<T> $crate::Intersects<$t> for LineString<T>
        where
            T: GeoNum,
            Line<T>: Intersects<$t>,
            $t: BoundingRect<T>,
        {
            fn intersects(&self, rhs: &$t) -> bool {
                if has_disjoint_bboxes(self, rhs) {
                    return false;
                }
                self.lines().any(|l| l.intersects(rhs))
            }
        }
    };
}

blanket_intersects_linestring!(Coord<T>);
blanket_intersects_linestring!(Point<T>);
blanket_intersects_linestring!(MultiPoint<T>);

blanket_intersects_linestring!(Line<T>);
blanket_intersects_linestring!(LineString<T>);
symmetric_intersects_impl!(LineString<T>, MultiLineString<T>);

impl<T> Intersects<Polygon<T>> for LineString<T>
where
    T: GeoNum,
    Line<T>: Intersects<Line<T>>,
    Coord<T>: Intersects<Polygon<T>>,
{
    fn intersects(&self, rhs: &Polygon<T>) -> bool {
        if self.is_empty() || rhs.is_empty() {
            return false;
        }
        if has_disjoint_bboxes(self, rhs) {
            return false;
        }
        // if no lines intersections, then linestring is either disjoint or within the polygon
        self.0[0].intersects(rhs)
            || self
                .lines()
                .any(|l| rhs.lines_iter().any(|other| l.intersects(&other)))
    }
}

impl<T> Intersects<MultiPolygon<T>> for LineString<T>
where
    T: GeoNum,
    Line<T>: Intersects<Line<T>>,
    Coord<T>: Intersects<Rect<T>>,
{
    fn intersects(&self, rhs: &MultiPolygon<T>) -> bool {
        if has_disjoint_bboxes(self, rhs) {
            return false;
        }
        // splitting into `LineString intersects Polygon`
        rhs.iter().any(|poly| self.intersects(poly))
    }
}

impl<T> Intersects<Rect<T>> for LineString<T>
where
    T: GeoNum,
    Line<T>: Intersects<Line<T>>,
    Coord<T>: Intersects<Rect<T>>,
{
    fn intersects(&self, rhs: &Rect<T>) -> bool {
        if self.is_empty() || rhs.is_empty() {
            return false;
        }
        if has_disjoint_bboxes(self, rhs) {
            return false;
        }
        // if no lines intersections, then linestring is either disjoint or within the polygon
        self.0[0].intersects(rhs)
            || self
                .lines()
                .any(|l| rhs.lines_iter().any(|other| l.intersects(&other)))
    }
}

impl<T> Intersects<Triangle<T>> for LineString<T>
where
    T: GeoNum,
    Line<T>: Intersects<Line<T>>,
    Coord<T>: Intersects<Triangle<T>>,
{
    fn intersects(&self, rhs: &Triangle<T>) -> bool {
        if self.is_empty() || rhs.is_empty() {
            return false;
        }
        if has_disjoint_bboxes(self, rhs) {
            return false;
        }
        // if no lines intersections, then linestring is either disjoint or within the polygon
        self.0[0].intersects(rhs)
            || self
                .lines()
                .any(|l| rhs.lines_iter().any(|other| l.intersects(&other)))
    }
}

// Blanket implementation from LineString<T>
impl<T, G> Intersects<G> for MultiLineString<T>
where
    T: CoordNum,
    LineString<T>: Intersects<G>,
    G: BoundingRect<T>,
{
    fn intersects(&self, rhs: &G) -> bool {
        if has_disjoint_bboxes(self, rhs) {
            return false;
        }
        self.iter().any(|p| p.intersects(rhs))
    }
}
