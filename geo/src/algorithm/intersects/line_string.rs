use super::{has_disjoint_bboxes, Intersects};
use crate::BoundingRect;
use crate::*;

// Blanket implementation using self.lines().any().
macro_rules! intersects_line_string_impl {
    ($t:ty) => {
        impl<T> $crate::Intersects<$t> for LineString<T>
        where
            T: GeoNum,
        {
            fn intersects(&self, rhs: &$t) -> bool {
                if has_disjoint_bboxes(self, rhs) {
                    return false;
                }
                self.lines().any(|l| l.intersects(rhs))
            }
        }
    };
    (area: $t:ty) => {
        impl<T> $crate::Intersects<$t> for LineString<T>
        where
            T: GeoNum,
        {
            fn intersects(&self, rhs: &$t) -> bool {
                if has_disjoint_bboxes(self, rhs) {
                    return false;
                }

                // if no lines intersections, then linestring is either disjoint or within the polygon
                // therefore sufficient to check any one point
                let Some(coord) = self.0.first() else {
                    return false;
                };
                coord.intersects(rhs)
                    || self
                        .lines()
                        .any(|l| rhs.lines_iter().any(|other| l.intersects(&other)))
            }
        }
    };
}

intersects_line_string_impl!(Coord<T>);
intersects_line_string_impl!(Point<T>);
intersects_line_string_impl!(MultiPoint<T>);

intersects_line_string_impl!(Line<T>);
intersects_line_string_impl!(LineString<T>);
symmetric_intersects_impl!(LineString<T>, MultiLineString<T>);

intersects_line_string_impl!(area: Polygon<T>);
impl<T> Intersects<MultiPolygon<T>> for LineString<T>
where
    T: GeoNum,
{
    fn intersects(&self, rhs: &MultiPolygon<T>) -> bool {
        if has_disjoint_bboxes(self, rhs) {
            return false;
        }
        // splitting into `LineString intersects Polygon`
        rhs.iter().any(|poly| self.intersects(poly))
    }
}

intersects_line_string_impl!(area: Rect<T>);
intersects_line_string_impl!(area: Triangle<T>);

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

#[cfg(test)]
mod test {
    use super::*;
    use crate::wkt;

    #[test]
    fn test_linestring_inside_polygon() {
        let ls: LineString<f64> = wkt! {LINESTRING(1 1, 2 2)}.convert();
        let poly: Polygon<f64> = Rect::new((0, 0), (10, 10)).to_polygon().convert();
        assert!(ls.intersects(&poly));
    }

    #[test]
    fn test_linestring_partial_polygon() {
        let ls: LineString<f64> = wkt! {LINESTRING(-1 -1, 2 2)}.convert();
        let poly: Polygon<f64> = Rect::new((0, 0), (10, 10)).to_polygon().convert();
        assert!(ls.intersects(&poly));
    }
    #[test]
    fn test_linestring_disjoint_polygon() {
        let ls: LineString<f64> = wkt! {LINESTRING(-1 -1, -2 -2)}.convert();
        let poly: Polygon<f64> = Rect::new((0, 0), (10, 10)).to_polygon().convert();
        assert!(!ls.intersects(&poly));
    }
    #[test]
    fn test_linestring_in_polygon_hole() {
        let ls: LineString<f64> = wkt! {LINESTRING(4 4, 6 6)}.convert();
        let bound = Rect::new((0, 0), (10, 10)).convert();
        let hole = Rect::new((1, 1), (9, 9)).convert();
        let poly = Polygon::new(
            bound.exterior_coords_iter().collect(),
            vec![hole.exterior_coords_iter().collect()],
        );

        assert!(!ls.intersects(&poly));
    }

    // ls_rect
    #[test]
    fn test_linestring_inside_rect() {
        let ls: LineString<f64> = wkt! {LINESTRING(1 1, 2 2)}.convert();
        let poly: Rect<f64> = Rect::new((0, 0), (10, 10)).convert();
        assert!(ls.intersects(&poly));
    }
    #[test]
    fn test_linestring_partial_rect() {
        let ls: LineString<f64> = wkt! {LINESTRING(-1 -1, 2 2)}.convert();
        let poly: Rect<f64> = Rect::new((0, 0), (10, 10)).convert();
        assert!(ls.intersects(&poly));
    }
    #[test]
    fn test_linestring_disjoint_rect() {
        let ls: LineString<f64> = wkt! {LINESTRING(-1 -1, -2 -2)}.convert();
        let poly: Rect<f64> = Rect::new((0, 0), (10, 10)).convert();
        assert!(!ls.intersects(&poly));
    }

    // ls_triangle
    #[test]
    fn test_linestring_inside_triangle() {
        let ls: LineString<f64> = wkt! {LINESTRING(5 5, 5 4)}.convert();
        let poly: Triangle<f64> = wkt! {TRIANGLE(0 0, 10 0, 5 10)}.convert();
        assert!(ls.intersects(&poly));
    }
    #[test]
    fn test_linestring_partial_triangle() {
        let ls: LineString<f64> = wkt! {LINESTRING(5 5, 5 -4)}.convert();
        let poly: Triangle<f64> = wkt! {TRIANGLE(0 0, 10 0, 5 10)}.convert();
        assert!(ls.intersects(&poly));
    }
    #[test]
    fn test_linestring_disjoint_triangle() {
        let ls: LineString<f64> = wkt! {LINESTRING(5 -5, 5 -4)}.convert();
        let poly: Triangle<f64> = wkt! {TRIANGLE(0 0, 10 0, 5 10)}.convert();
        assert!(!ls.intersects(&poly));
    }
}
