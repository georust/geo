use geo_types::{Coord, LineString};
use num_traits::Bounded;

use crate::{ConvexHull, CoordFloat, GeoFloat, GeoNum, Polygon, algorithm::CoordsIter};
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
///         (1.6999999999999975, 24.6),
///         (1.450145836371588, 30.44658742890477),
///         (14.4, 31.0),
///         (14.64985416362841, 25.15341257109523),
///         (1.6999999999999975, 24.6),
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

        // Pick a hull vertex and translate it to the origin to improve precision
        let ref_p = hull.exterior().points().next()?;

        for edge in hull.exterior().lines() {
            let (dx, dy) = edge.delta().x_y();
            let norm = dx.hypot(dy);
            if norm.is_zero() {
                continue;
            }
            let edge = (dx / norm, dy / norm);

            let (mut min_x, mut max_x) = (T::zero(), T::zero());
            let (mut min_y, mut max_y) = (T::zero(), T::zero());

            for point in hull
                .exterior()
                .points()
                .skip(1)
                .take(hull.exterior().0.len() - 2)
            {
                let tx = point.x() - ref_p.x();
                let ty = point.y() - ref_p.y();

                let x = tx * edge.0 + ty * edge.1;
                let y = -tx * edge.1 + ty * edge.0;

                min_x = min_x.min(x);
                max_x = max_x.max(x);
                min_y = min_y.min(y);
                max_y = max_y.max(y);
            }

            let area = (max_x - min_x) * (max_y - min_y);
            if area < min_area {
                min_area = area;
                best_edge = Some(edge);
                best_min_x = min_x;
                best_max_x = max_x;
                best_min_y = min_y;
                best_max_y = max_y;
            }
        }

        if let Some((dx, dy)) = best_edge {
            let to_world = |x: T, y: T| Coord {
                x: x * dx + y * -dy + ref_p.x(),
                y: x * dy + y * dx + ref_p.y(),
            };

            let p1 = to_world(best_min_x, best_min_y);
            let p2 = to_world(best_min_x, best_max_y);
            let p3 = to_world(best_max_x, best_max_y);
            let p4 = to_world(best_max_x, best_min_y);
            let rectangle = Polygon::new(LineString(vec![p1, p2, p3, p4, p1]), Vec::new());
            Some(rectangle)
        } else {
            None
        }
    }
}

#[cfg(test)]
mod test {
    use geo_types::{LineString, Polygon, line_string, polygon};

    use crate::MinimumRotatedRect;

    #[test]
    fn returns_polygon_mbr() {
        let poly: Polygon<f64> = polygon![(x: 3.3, y: 30.4), (x: 1.7, y: 24.6), (x: 13.4, y: 25.1), (x: 14.4, y: 31.0),(x:3.3,y:30.4)];
        let mbr = MinimumRotatedRect::minimum_rotated_rect(&poly).unwrap();
        assert_eq!(
            mbr.exterior(),
            &LineString::from(vec![
                (1.6999999999999975, 24.6),
                (1.450145836371588, 30.44658742890477),
                (14.4, 31.0),
                (14.64985416362841, 25.15341257109523),
                (1.6999999999999975, 24.6),
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
                (1.6999999999999975, 24.6),
                (1.450145836371588, 30.44658742890477),
                (14.4, 31.0),
                (14.64985416362841, 25.15341257109523),
                (1.6999999999999975, 24.6),
            ])
        );
    }
}
