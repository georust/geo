use num_traits::Float;

use crate::{
    algorithm::{centroid::Centroid, rotate::Rotate, BoundingRect, CoordsIter},
    Area, ConvexHull, CoordFloat, GeoFloat, GeoNum, LinesIter, Polygon,
};
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
/// let poly: Polygon<f64> = polygon![(x: 3.3, y: 30.4), (x: 1.7, y: 24.6), (x: 13.4, y: 25.1), (x: 14.4, y: 31.0),(x:3.3,y:30.4)];
/// let mbr = MinimumRotatedRect::minimum_rotated_rect(&poly).unwrap();
/// assert_eq!(
///     mbr.exterior(),
///     &LineString::from(vec![
///         (14.649854163628408, 25.153412571095235),
///         (14.4, 31.0),
///         (1.4501458363715918, 30.446587428904767),
///         (1.7000000000000006, 24.6),
///         (14.649854163628408, 25.153412571095235),
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
        let convex_poly = ConvexHull::convex_hull(self);
        let mut min_area: T = Float::max_value();
        let mut min_angle: T = T::zero();
        let mut rect_poly: Option<Polygon<T>> = None;
        let rotate_point = convex_poly.centroid();
        for line in convex_poly.exterior().lines_iter() {
            let (ci, cii) = line.points();
            let angle = (cii.y() - ci.y()).atan2(cii.x() - ci.x()).to_degrees();
            let rotated_poly = Rotate::rotate_around_point(&convex_poly, -angle, rotate_point?);
            let tmp_poly = rotated_poly.bounding_rect()?.to_polygon();
            let area = tmp_poly.unsigned_area();
            if area < min_area {
                min_area = area;
                min_angle = angle;
                rect_poly = Some(tmp_poly);
            }
        }
        Some(rect_poly?.rotate_around_point(min_angle, rotate_point?))
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
                (14.649854163628408, 25.153412571095235),
                (14.4, 31.0),
                (1.4501458363715918, 30.446587428904767),
                (1.7000000000000006, 24.6),
                (14.649854163628408, 25.153412571095235),
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
                (14.649854163628408, 25.153412571095235),
                (14.4, 31.0),
                (1.4501458363715918, 30.446587428904767),
                (1.7000000000000006, 24.6),
                (14.649854163628408, 25.153412571095235),
            ])
        );
    }
}
