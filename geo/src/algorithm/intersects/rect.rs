use super::{Intersects, has_disjoint_bboxes};
use crate::*;

impl<T> Intersects<Coord<T>> for Rect<T>
where
    T: CoordNum,
{
    fn intersects(&self, rhs: &Coord<T>) -> bool {
        rhs.x >= self.min().x
            && rhs.y >= self.min().y
            && rhs.x <= self.max().x
            && rhs.y <= self.max().y
    }
}

symmetric_intersects_impl!(Rect<T>, LineString<T>);
symmetric_intersects_impl!(Rect<T>, MultiLineString<T>);

// Same logic as Polygon<T>: Intersects<Line<T>>, but avoid
// an allocation.
impl<T> Intersects<Line<T>> for Rect<T>
where
    T: GeoNum,
{
    fn intersects(&self, rhs: &Line<T>) -> bool {
        if !self.intersects(&rhs.bounding_rect()) {
            return false;
        }

        /*
        o3--o2
        |    |
        |    |
        o0--o1
        check if `rhs` line extended to infinity crosses any of the 4 edges of the `Rect`
        */

        let c0 = self.min();
        let c1 = coord! {x: self.max().x, y: self.min().y};

        let o0 = T::Ker::orient2d(rhs.start, rhs.end, c0);
        let o1 = T::Ker::orient2d(rhs.start, rhs.end, c1);
        if o0 != o1 {
            return true;
        }

        let c2 = self.max();
        let o2 = T::Ker::orient2d(rhs.start, rhs.end, c2);
        if o0 != o2 {
            // equivalent to o1 != o2
            return true;
        }

        let c3 = coord! {x: self.min().x, y: self.max().y};
        let o3 = T::Ker::orient2d(rhs.start, rhs.end, c3);
        if o0 != o3 {
            // equivalent to o2 != o3
            return true;
        }

        // safe to use n-1 comparisons because we know that if there is a different orientation,
        // then there must be at least two edges along which the orientation of its points is different
        debug_assert!(o3 == o0);

        // At this point we know all the orientations are equal and that the bounding boxes overlap.
        // The only ways there could be an intersection is if
        // 1. `self` (Rect) has degenerated to a line, and `rhs` (Line) is on that line.
        // 2. `self` (Rect) has degenerated to a point, and lies on `rhs` (Line).
        // 3. `rhs` (Line) is degenerated to a point.
        o0 == Orientation::Collinear
    }
}

symmetric_intersects_impl!(Rect<T>, Point<T>);
symmetric_intersects_impl!(Rect<T>, MultiPoint<T>);

impl<T> Intersects<Polygon<T>> for Rect<T>
where
    T: GeoNum,
{
    fn intersects(&self, rhs: &Polygon<T>) -> bool {
        // simplified logic based on Polygon intersects Polygon

        if has_disjoint_bboxes(self, rhs) {
            return false;
        }

        // if any of the polygon's corners intersect the rectangle
        rhs.coords_iter().take(1).any(|p| self.intersects(&p))

        // or any point of the rectangle intersects the polygon
        || self.min().intersects(rhs)

        // or any of the polygon's lines intersect the rectangle's lines
        || rhs.lines_iter().any(|rhs_line| {
            self.lines_iter()
                .any(|self_line| self_line.intersects(&rhs_line))
        })
    }
}

symmetric_intersects_impl!(Rect<T>, MultiPolygon<T>);

impl<T> Intersects<Rect<T>> for Rect<T>
where
    T: CoordNum,
{
    fn intersects(&self, other: &Rect<T>) -> bool {
        if self.max().x < other.min().x {
            return false;
        }

        if self.max().y < other.min().y {
            return false;
        }

        if self.min().x > other.max().x {
            return false;
        }

        if self.min().y > other.max().y {
            return false;
        }

        true
    }
}

impl<T> Intersects<Triangle<T>> for Rect<T>
where
    T: GeoNum,
{
    fn intersects(&self, rhs: &Triangle<T>) -> bool {
        // simplified logic based on Polygon intersects Polygon

        if has_disjoint_bboxes(self, rhs) {
            return false;
        }

        // if any of the triangle's corners intersect the rectangle
        self.intersects(&rhs.0)

        // or some corner of the triangle intersects the rectangle
        || self.min().intersects(rhs)

        // or any of the triangle's lines intersect the rectangle's lines
        || rhs.lines_iter().any(|rhs_line| {
            self.lines_iter()
                .any(|self_line| self_line.intersects(&rhs_line))
        })
    }
}

#[cfg(test)]
mod test_line {
    use super::*;
    use crate::wkt;

    #[test]
    fn test_overlap_bbox_no_overlap() {
        let rect = wkt! {RECT(6 4, 10 0)};
        let line = wkt! {LINE(0 0, 10 10)};

        assert!(!rect.intersects(&line));
    }

    #[test]
    fn test_degen_line() {
        let rect = wkt! {RECT(0 0, 10 10)};
        let line = wkt! {LINE(0 0, 0 0)};

        assert!(rect.intersects(&line));
    }

    #[test]
    fn test_degen_rect() {
        let rect_pt = wkt! {RECT(0 0, 0 10)};
        let rect_line1 = wkt! {RECT(0 0, 0 10)};
        let rect_line2 = wkt! {RECT(0 0, 10 0)};
        let line = wkt! {LINE(0 0, 10 0)};

        assert!(rect_pt.intersects(&line));
        assert!(rect_line1.intersects(&line));
        assert!(rect_line2.intersects(&line));
    }
}

#[cfg(test)]
mod test_triangle {
    use super::*;

    #[test]
    fn test_rhs_degenerate_line() {
        let s: Rect<f64> = Rect::new((0, 0), (0, 0)).convert();
        let l = Line::new(coord! {x: 0.0, y: 0.0}, coord! { x: 0.0, y: 0.0 });

        assert!(s.intersects(&l));
    }

    #[test]
    fn test_disjoint() {
        let rect: Rect<f64> = Rect::new((0, 0), (10, 10)).convert();
        let triangle = Triangle::from([(0., 11.), (1., 11.), (1., 12.)]);
        assert!(!rect.intersects(&triangle));
    }

    #[test]
    fn test_partial() {
        let rect: Rect<f64> = Rect::new((0, 0), (10, 10)).convert();
        let triangle = Triangle::from([(1., 1.), (1., 2.), (2., 1.)]);
        assert!(rect.intersects(&triangle));
    }

    #[test]
    fn test_triangle_inside_rect() {
        let rect: Rect<f64> = Rect::new((0, 0), (10, 10)).convert();
        let triangle = Triangle::from([(1., 1.), (1., 2.), (2., 1.)]);
        assert!(rect.intersects(&triangle));
    }

    #[test]
    fn test_rect_inside_triangle() {
        let rect: Rect<f64> = Rect::new((1, 1), (2, 2)).convert();
        let triangle = Triangle::from([(0., 10.), (10., 0.), (0., 0.)]);
        assert!(rect.intersects(&triangle));
    }
}

#[cfg(test)]
mod test_polygon {
    use super::*;

    #[test]
    fn test_disjoint() {
        let rect: Rect<f64> = Rect::new((0, 0), (10, 10)).convert();
        let polygon: Polygon<f64> = Rect::new((11, 11), (12, 12)).to_polygon().convert();
        assert!(!rect.intersects(&polygon));
    }

    #[test]
    fn test_partial() {
        let rect: Rect<f64> = Rect::new((0, 0), (10, 10)).convert();
        let polygon: Polygon<f64> = Rect::new((9, 9), (12, 12)).to_polygon().convert();
        assert!(rect.intersects(&polygon));
    }

    #[test]
    fn test_rect_inside_polygon() {
        let rect: Rect<f64> = Rect::new((1, 1), (2, 2)).convert();
        let polygon: Polygon<f64> = Rect::new((0, 0), (10, 10)).to_polygon().convert();
        assert!(rect.intersects(&polygon));
    }

    #[test]
    fn test_polygon_inside_rect() {
        let rect: Rect<f64> = Rect::new((0, 0), (10, 10)).convert();
        let polygon: Polygon<f64> = Rect::new((1, 1), (2, 2)).to_polygon().convert();
        assert!(rect.intersects(&polygon));
    }

    // Hole related tests

    #[test]
    fn test_rect_inside_polygon_hole() {
        let bound: Rect<f64> = Rect::new((0, 0), (10, 10)).convert();
        let hole = Rect::new((1, 1), (9, 9)).convert();
        let rect = Rect::new((4, 4), (6, 6)).convert();
        let polygon = Polygon::new(
            bound.exterior_coords_iter().collect(),
            vec![hole.exterior_coords_iter().collect()],
        );

        assert!(!rect.intersects(&polygon));
    }

    #[test]
    fn test_rect_equals_polygon_hole() {
        let bound: Rect<f64> = Rect::new((0, 0), (10, 10)).convert();
        let rect: Rect = Rect::new((4, 4), (6, 6)).convert();
        let polygon = Polygon::new(
            bound.exterior_coords_iter().collect(),
            vec![rect.exterior_coords_iter().collect()],
        );

        assert!(rect.intersects(&polygon));
    }
}
