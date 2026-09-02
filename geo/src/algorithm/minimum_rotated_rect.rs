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

#[cfg(test)]
mod hegel_props {
    use super::MinimumRotatedRect;
    use crate::utils::pbt_gens::grid_coords;
    use crate::{Area, BoundingRect, ConvexHull, Coord, Distance, Euclidean, MultiPoint, Point};
    use hegel::generators::{self, PrintableGenerator};

    fn point_sets() -> impl PrintableGenerator<Vec<Coord<f64>>> {
        generators::vecs(grid_coords()).max_size(32)
    }

    fn multi_point(coords: &[Coord<f64>]) -> MultiPoint<f64> {
        MultiPoint::new(coords.iter().copied().map(Point::from).collect())
    }

    // "minimum rotated rect is the rectangle that can enclose all points given
    // and have smallest area of all enclosing rectangles".
    #[hegel::test]
    fn the_minimum_rotated_rect_encloses_every_input_point(tc: hegel::TestCase) {
        let coords = tc.draw(point_sets());
        let points = multi_point(&coords);
        let Some(rect) = points.minimum_rotated_rect() else {
            return;
        };
        // The implementation translates a hull vertex to the origin and back
        // "to improve precision", so a point on the boundary can land a few
        // ulps outside. The slack is relative to the extent of the input.
        let scale = coords
            .iter()
            .map(|c| c.x.abs().max(c.y.abs()))
            .fold(1.0_f64, f64::max);
        for coord in &coords {
            let distance = Euclidean.distance(&Point::from(*coord), &rect);
            assert!(
                distance <= 1e-9 * scale,
                "{coord:?} lies {distance} outside {rect:?}"
            );
        }
    }

    // The axis-aligned bounding rect is one of the enclosing rectangles, so the
    // minimum-area one cannot be larger; and every enclosing rectangle covers
    // the convex hull, so it cannot be smaller than the hull.
    #[hegel::test]
    fn its_area_sits_between_the_hull_and_the_bounding_rect(tc: hegel::TestCase) {
        let coords = tc.draw(point_sets());
        let points = multi_point(&coords);
        let Some(rect) = points.minimum_rotated_rect() else {
            return;
        };
        let area = rect.unsigned_area();
        let hull_area = points.convex_hull().unsigned_area();
        let bounding_area = points
            .bounding_rect()
            .expect("a non-empty point set")
            .unsigned_area();
        assert!(
            area <= bounding_area * (1.0 + 1e-9) + 1e-9,
            "minimum rotated rect area {area} exceeds the bounding rect area {bounding_area}"
        );
        assert!(
            area >= hull_area * (1.0 - 1e-9) - 1e-9,
            "minimum rotated rect area {area} is below the hull area {hull_area}"
        );
    }

    // Only the convex hull's vertices can be extreme, and the implementation
    // runs rotating calipers over the hull, so hulling first leaves the same
    // rectangle. The two runs pick different hull vertices as their precision
    // origin, so the corners agree only to within rounding and the comparison
    // is on area.
    #[hegel::test]
    fn it_only_depends_on_the_convex_hull(tc: hegel::TestCase) {
        let coords = tc.draw(point_sets());
        let points = multi_point(&coords);
        let (Some(direct), Some(via_hull)) = (
            points.minimum_rotated_rect(),
            points.convex_hull().minimum_rotated_rect(),
        ) else {
            assert_eq!(
                points.minimum_rotated_rect(),
                points.convex_hull().minimum_rotated_rect()
            );
            return;
        };
        assert_relative_eq!(
            direct.unsigned_area(),
            via_hull.unsigned_area(),
            max_relative = 1e-9,
            epsilon = 1e-9
        );
    }

    // Rotating calipers returns a closed four-corner ring with no holes.
    #[hegel::test]
    fn the_result_is_a_closed_quadrilateral(tc: hegel::TestCase) {
        let coords = tc.draw(point_sets());
        let Some(rect) = multi_point(&coords).minimum_rotated_rect() else {
            return;
        };
        assert_eq!(rect.exterior().0.len(), 5);
        assert!(rect.exterior().is_closed());
        assert!(rect.interiors().is_empty());
    }
}
