use super::{impl_contains_from_relate, impl_contains_geometry_for, Contains};
use crate::geometry::*;
use crate::{kernels::Kernel, GeoFloat, GeoNum, Intersects, LinesIter, Orientation};

// ┌──────────────────────────────┐
// │ Implementations for Triangle │
// └──────────────────────────────┘

impl<T> Contains<Coord<T>> for Triangle<T>
where
    T: GeoNum,
{
    fn contains(&self, coord: &Coord<T>) -> bool {
        // leverageing robust predicates
        self.to_lines()
            .map(|l| T::Ker::orient2d(l.start, l.end, *coord))
            .windows(2)
            .all(|win| win[0] == win[1] && win[0] != Orientation::Collinear)

        // // neglecting robust prdicates, hence faster
        // let p0x = self.0.x.to_f64().unwrap();
        // let p0y = self.0.y.to_f64().unwrap();
        // let p1x = self.1.x.to_f64().unwrap();
        // let p1y = self.1.y.to_f64().unwrap();
        // let p2x = self.2.x.to_f64().unwrap();
        // let p2y = self.2.y.to_f64().unwrap();

        // let px = coord.x.to_f64().unwrap();
        // let py = coord.y.to_f64().unwrap();

        // let a = 0.5 * (-p1y * p2x + p0y * (-p1x + p2x) + p0x * (p1y - p2y) + p1x * p2y);

        // let sign = a.signum();

        // let s = (p0y * p2x - p0x * p2y + (p2y - p0y) * px + (p0x - p2x) * py) * sign;
        // let t = (p0x * p1y - p0y * p1x + (p0y - p1y) * px + (p1x - p0x) * py) * sign;

        // s > 0. && t > 0. && (s + t) < 2. * a * sign
    }
}

impl<T> Contains<Point<T>> for Triangle<T>
where
    T: GeoNum,
{
    fn contains(&self, point: &Point<T>) -> bool {
        self.contains(&point.0)
    }
}

impl<T> Contains<Line<T>> for Triangle<T>
where
    T: GeoNum,
    Line<T>: Contains<Line<T>>,
    Triangle<T>: Intersects<Coord<T>>,
{
    fn contains(&self, rhs: &Line<T>) -> bool {
        // self intersects all points && line does not sit on any of the self's edges

        self.intersects(&rhs.start)
            && self.intersects(&rhs.end)
            && (self.contains(&rhs.start)
                || self.contains(&rhs.end)
                || !self.lines_iter().any(|edge| edge.contains(rhs)))
    }
}

impl_contains_from_relate!(Triangle<T>, [LineString<T>, Polygon<T>, MultiPoint<T>, MultiLineString<T>, MultiPolygon<T>, GeometryCollection<T>, Rect<T>, Triangle<T>]);
impl_contains_geometry_for!(Triangle<T>);

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{coord, Line, Point, Relate, Triangle};

    #[test]
    fn triangle_contains_line() {
        let tri = Triangle::new(
            coord! {x:0., y:0.},
            coord! {x:10., y:0.},
            coord! {x:10., y:5.},
        );

        let ln_within = Line::new(Point::new(3., 1.), Point::new(9., 3.));
        let ln_in_cross = Line::new(Point::new(0., 0.), Point::new(10., 4.));

        let ln_boundary = Line::new(Point::new(0., 0.), Point::new(10., 0.));
        let ln_boundary_partial = Line::new(Point::new(1., 0.), Point::new(9., 0.));

        let ln_disjoint = Line::new(Point::new(0., 6.), Point::new(10., 6.));
        let ln_out_cross = Line::new(Point::new(0., 0.), Point::new(9., 6.));

        assert!(tri.contains(&ln_within));
        assert!(tri.relate(&ln_within).is_contains());

        assert!(tri.contains(&ln_in_cross));
        assert!(tri.relate(&ln_in_cross).is_contains());

        assert!(!tri.contains(&ln_boundary));
        assert!(!tri.relate(&ln_boundary).is_contains());

        assert!(!tri.contains(&ln_boundary_partial));
        assert!(!tri.relate(&ln_boundary_partial).is_contains());

        assert!(!tri.contains(&ln_disjoint));
        assert!(!tri.relate(&ln_disjoint).is_contains());

        assert!(!tri.contains(&ln_out_cross));
        assert!(!tri.relate(&ln_out_cross).is_contains());
    }

    /**
     *  Implementation of Triangle contains Line requires line contains self to be true
     */
    #[test]
    fn line_contains_self() {
        let ln = Line::new(Point::new(0., 0.), Point::new(10., 0.));
        assert!(ln.contains(&ln));
        assert!(ln.relate(&ln).is_contains());
    }
}
