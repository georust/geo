use super::{has_disjoint_bboxes, Intersects};
use crate::*;

impl<T> Intersects<Coord<T>> for Triangle<T>
where
    T: GeoNum,
{
    fn intersects(&self, rhs: &Coord<T>) -> bool {
        let mut orientations = self
            .to_lines()
            .map(|l| T::Ker::orient2d(l.start, l.end, *rhs));

        orientations.sort();

        !orientations
            .windows(2)
            .any(|win| win[0] != win[1] && win[1] != Orientation::Collinear)

        // // neglecting robust predicates, hence faster
        // let p0x = self.0.x.to_f64().unwrap();
        // let p0y = self.0.y.to_f64().unwrap();
        // let p1x = self.1.x.to_f64().unwrap();
        // let p1y = self.1.y.to_f64().unwrap();
        // let p2x = self.2.x.to_f64().unwrap();
        // let p2y = self.2.y.to_f64().unwrap();

        // let px = rhs.x.to_f64().unwrap();
        // let py = rhs.y.to_f64().unwrap();

        // let s = (p0x - p2x) * (py - p2y) - (p0y - p2y) * (px - p2x);
        // let t = (p1x - p0x) * (py - p0y) - (p1y - p0y) * (px - p0x);

        // if (s < 0.) != (t < 0.) && s != 0. && t != 0. {
        //     return false;
        // }

        // let d = (p2x - p1x) * (py - p1y) - (p2y - p1y) * (px - p1x);
        // d == 0. || (d < 0.) == (s + t <= 0.)
    }
}

symmetric_intersects_impl!(Triangle<T>, LineString<T>);
symmetric_intersects_impl!(Triangle<T>, MultiLineString<T>);

symmetric_intersects_impl!(Triangle<T>, Line<T>);

symmetric_intersects_impl!(Triangle<T>, Point<T>);
symmetric_intersects_impl!(Triangle<T>, MultiPoint<T>);

impl<T> Intersects<Polygon<T>> for Triangle<T>
where
    T: GeoNum,
{
    fn intersects(&self, rhs: &Polygon<T>) -> bool {
        // simplified logic based on Polygon intersects Polygon

        if has_disjoint_bboxes(self, rhs) {
            return false;
        }

        // if any of the polygon's corners intersect the triangle
        rhs.coords_iter().take(1).any(|p| self.intersects(&p))

        // or any point of the triangle intersects the polygon
        || self.0.intersects(rhs)

        // or any of the polygon's lines intersect the triangle's lines
        || rhs.lines_iter().any(|rhs_line| {
            self.lines_iter()
                .any(|self_line| self_line.intersects(&rhs_line))
        })
    }
}

symmetric_intersects_impl!(Triangle<T>, MultiPolygon<T>);

symmetric_intersects_impl!(Triangle<T>, Rect<T>);

impl<T> Intersects<Triangle<T>> for Triangle<T>
where
    T: GeoNum,
{
    fn intersects(&self, rhs: &Triangle<T>) -> bool {
        self.to_polygon().intersects(&rhs.to_polygon())
    }
}

#[cfg(test)]
mod test_polygon {
    use super::*;

    #[test]
    fn test_disjoint() {
        let triangle = Triangle::from([(0., 0.), (10., 0.), (10., 10.)]);
        let polygon: Polygon<f64> = Rect::new((11, 11), (12, 12)).to_polygon().convert();
        assert!(!triangle.intersects(&polygon));
    }

    #[test]
    fn test_partial() {
        let triangle = Triangle::from([(0., 0.), (10., 0.), (10., 10.)]);
        let polygon: Polygon<f64> = Rect::new((9, 9), (12, 12)).to_polygon().convert();
        assert!(triangle.intersects(&polygon));
    }

    #[test]
    fn test_triangle_inside_polygon() {
        let triangle = Triangle::from([(1., 1.), (2., 1.), (2., 2.)]);
        let polygon: Polygon<f64> = Rect::new((0, 0), (10, 10)).to_polygon().convert();
        assert!(triangle.intersects(&polygon));
    }

    #[test]
    fn test_polygon_inside_triangle() {
        let triangle = Triangle::from([(0., 0.), (10., 0.), (10., 10.)]);
        let polygon: Polygon<f64> = Rect::new((1, 1), (2, 2)).to_polygon().convert();
        assert!(triangle.intersects(&polygon));
    }

    // Hole related tests

    #[test]
    fn test_rect_inside_polygon_hole() {
        let bound: Rect<f64> = Rect::new((0, 0), (10, 10)).convert();
        let hole = Rect::new((1, 1), (9, 9)).convert();
        let triangle = Triangle::from([(4., 4.), (4., 6.), (6., 6.)]);
        let polygon = Polygon::new(
            bound.exterior_coords_iter().collect(),
            vec![hole.exterior_coords_iter().collect()],
        );

        assert!(!triangle.intersects(&polygon));
    }

    #[test]
    fn test_rect_equals_polygon_hole() {
        let bound: Rect<f64> = Rect::new((0, 0), (10, 10)).convert();
        let triangle = Triangle::from([(4., 4.), (4., 6.), (6., 6.)]);
        let polygon = Polygon::new(
            bound.exterior_coords_iter().collect(),
            vec![triangle.exterior_coords_iter().collect()],
        );

        assert!(triangle.intersects(&polygon));
    }
}
