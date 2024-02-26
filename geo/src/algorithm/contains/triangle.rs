use super::{impl_contains_from_relate, impl_contains_geometry_for, Contains};
use crate::geometry::*;
use crate::{kernels::Kernel, GeoFloat, GeoNum, Orientation};

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

impl_contains_from_relate!(Triangle<T>, [Line<T>, LineString<T>, Polygon<T>, MultiPoint<T>, MultiLineString<T>, MultiPolygon<T>, GeometryCollection<T>, Rect<T>, Triangle<T>]);
impl_contains_geometry_for!(Triangle<T>);
