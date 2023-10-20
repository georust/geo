use geo_types::LineString;
use num_traits::Bounded;

use crate::{algorithm::CoordsIter, ConvexHull, CoordFloat, GeoFloat, GeoNum, Polygon};
/// Return the minimum bounding rectangle(MBR) of geometry
/// reference: <https://en.wikipedia.org/wiki/Minimum_bounding_box>
/// minimum rotated rect is the rectangle that can enclose all points given
/// and have smallest area of all enclosing rectangles
/// the rect can be any-oriented, not only axis-aligned.
///
/// # Examples
///
/// ```
/// use geo_types::{line_string, polygon, LineString, Polygon};
/// use geo::MinimumRotatedRect;
/// let poly: Polygon<f64> = polygon![(x: 3.3, y: 30.4), (x: 1.7, y: 24.6), (x: 13.4, y: 25.1), (x: 14.4, y: 31.0), (x:3.3, y:30.4)];
/// let mbr = MinimumRotatedRect::minimum_rotated_rect(&poly).unwrap();
/// assert_eq!(
///     mbr.exterior(),
///     &LineString::from(vec![
///         (1.7000000000000004, 24.600000000000005),
///         (14.649854163628412, 25.153412571095238),
///         (14.400000000000002, 31.000000000000007),
///         (1.4501458363715916, 30.446587428904774),
///         (1.7000000000000004, 24.600000000000005),
///     ])
/// );
/// ```
pub trait MinimumRotatedRect<T> {
    type Scalar: GeoNum;
    fn minimum_rotated_rect(&self) -> Option<Polygon<Self::Scalar>>;
}

impl<T, G> MinimumRotatedRect<T> for G
where
    T: CoordFloat + GeoFloat + GeoNum,
    G: CoordsIter<Scalar = T>,
{
    type Scalar = T;

    fn minimum_rotated_rect(&self) -> Option<Polygon<Self::Scalar>> {
        let hull = ConvexHull::convex_hull(self);

        // We take unit vectors along and perpendicular to each edge, then use
        // them to build a rotation matrix and apply it to the convex hull,
        // keeping track of the best AABB found.
        //
        // See also the discussion at
        // https://gis.stackexchange.com/questions/22895/finding-minimum-area-rectangle-for-given-points/22934
        let mut min_area = <T as Bounded>::max_value();
        let mut best_edge = None;
        let (mut best_min_x, mut best_max_x) = (T::zero(), T::zero());
        let (mut best_min_y, mut best_max_y) = (T::zero(), T::zero());
        for edge in hull.exterior().lines() {
            let (dx, dy) = edge.delta().x_y();
            let norm = dx.hypot(dy);
            if norm.is_zero() {
                continue;
            }
            let edge = (dx / norm, dy / norm);

            let (mut min_x, mut max_x) = (<T as Bounded>::max_value(), <T as Bounded>::min_value());
            let (mut min_y, mut max_y) = (<T as Bounded>::max_value(), <T as Bounded>::min_value());
            for point in hull.exterior().points() {
                let x = point.y() * edge.1 + point.x() * edge.0;
                let y = point.y() * edge.0 - point.x() * edge.1;

                min_x = min_x.min(x);
                max_x = max_x.max(x);
                min_y = min_y.min(y);
                max_y = max_y.max(y);
            }

            let area = (max_y - min_y) * (max_x - min_x);
            if area < min_area {
                min_area = area;
                best_edge = Some(edge);
                best_min_x = min_x;
                best_max_x = max_x;
                best_min_y = min_y;
                best_max_y = max_y;
            }
        }

        if let Some(e) = best_edge {
            let p1 = (
                best_min_x * e.0 - best_min_y * e.1,
                best_min_x * e.1 + best_min_y * e.0,
            );
            let p2 = (
                best_max_x * e.0 - best_min_y * e.1,
                best_max_x * e.1 + best_min_y * e.0,
            );
            let p3 = (
                best_max_x * e.0 - best_max_y * e.1,
                best_max_x * e.1 + best_max_y * e.0,
            );
            let p4 = (
                best_min_x * e.0 - best_max_y * e.1,
                best_min_x * e.1 + best_max_y * e.0,
            );
            let rectangle = Polygon::new(
                LineString(vec![p1.into(), p2.into(), p3.into(), p4.into(), p1.into()]),
                vec![],
            );
            Some(rectangle)
        } else {
            None
        }
    }
}

#[cfg(test)]
mod test {
    use geo_types::{line_string, polygon, LineString, Polygon};

    use crate::MinimumRotatedRect;

    #[test]
    fn returns_polygon_mbr() {
        let poly: Polygon<f64> = polygon![(x: 3.3, y: 30.4), (x: 1.7, y: 24.6), (x: 13.4, y: 25.1), (x: 14.4, y: 31.0),(x:3.3,y:30.4)];
        let mbr = MinimumRotatedRect::minimum_rotated_rect(&poly).unwrap();
        assert_eq!(
            mbr.exterior(),
            &LineString::from(vec![
                (1.7000000000000004, 24.600000000000005),
                (14.649854163628412, 25.153412571095238),
                (14.400000000000002, 31.000000000000007),
                (1.4501458363715916, 30.446587428904774),
                (1.7000000000000004, 24.600000000000005),
            ])
        );
    }
    #[test]
    fn returns_linestring_mbr() {
        let poly: LineString<f64> = line_string![(x: 3.3, y: 30.4), (x: 1.7, y: 24.6), (x: 13.4, y: 25.1), (x: 14.4, y: 31.0)];
        let mbr = MinimumRotatedRect::minimum_rotated_rect(&poly).unwrap();
        assert_eq!(
            mbr.exterior(),
            &LineString::from(vec![
                (1.7000000000000004, 24.600000000000005),
                (14.649854163628412, 25.153412571095238),
                (14.400000000000002, 31.000000000000007),
                (1.4501458363715916, 30.446587428904774),
                (1.7000000000000004, 24.600000000000005),
            ])
        );
    }
}
