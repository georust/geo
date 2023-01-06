use crate::{
    algorithm::{centroid::Centroid, rotate::Rotate, BoundingRect, CoordsIter},
    coords_iter, Area, ConvexHull, CoordFloat, GeoFloat, GeoNum, Point, Polygon,
};
/// Returns the minimun bounding rect of polygon.
///

pub trait MinimunRotatedRect<'a, T> {
    /// Return the minimun bounding rectangle of polygon
    ///
    /// # Examples
    ///
    /// ```
    /// # #[macro_use] extern crate approx;
    /// #
    /// use geo::MinimunRotatedRect;
    /// use geo::line_string;
    ///
    /// let poly: Polygon<f64> = polygon![(x: 3.3, y: 30.4), (x: 1.7, y: 24.6), (x: 13.4, y: 25.1), (x: 14.4, y: 31.0),(x:3.3,y:30.4)];
    /// let mbr = MinimunRotatedRect::minimun_rotated_rect(&poly).unwrap();
    /// assert_eq!(
    ///     mbr.exterior(),
    ///     &LineString::from(vec![
    ///         (1.7000000000000006, 24.6),
    ///         (1.4501458363715918, 30.446587428904767),
    ///         (14.4, 31.0),
    ///         (14.649854163628408, 25.153412571095235),
    ///         (1.7000000000000006, 24.6),
    ///     ])
    /// );
    /// ```
    type Scalar: GeoNum;
    type Output: Into<Option<Polygon<Self::Scalar>>>;
    fn minimun_rotated_rect(&'a self) -> Self::Output;
}

impl<'a, T, G> MinimunRotatedRect<'a, T> for G
where
    T: CoordFloat + GeoFloat + GeoNum,
    G: CoordsIter<'a, Scalar = T>,
    f64: From<T>,
{
    type Scalar = T;
    type Output = Option<Polygon<T>>;
    fn minimun_rotated_rect(&'a self) -> Self::Output {
        let convex_poly = ConvexHull::convex_hull(self);
        let mut min_area = f64::MAX;
        let mut min_angle: T = T::zero();
        let points = convex_poly.exterior().clone().into_points();
        let mut rect_poly: Option<Polygon<T>> = None;
        let mut rotate_point: Option<Point<T>> = None;
        for i in 0..points.len() - 1 {
            let ci = points[i];
            let cii = points[i + 1];
            let angle = (cii.y() - ci.y()).atan2(cii.x() - ci.x()).to_degrees();
            let rotated_poly = Rotate::rotate_around_centroid(&convex_poly, -angle);
            let tmp_poly = rotated_poly.bounding_rect()?.to_polygon();
            let area = f64::from(tmp_poly.unsigned_area());
            if area < min_area {
                min_area = area;
                min_angle = angle;
                rect_poly = Some(tmp_poly);
                rotate_point = convex_poly.centroid();
            }
        }
        Some(rect_poly?.rotate_around_point(min_angle, rotate_point?))
    }
}

#[cfg(test)]
mod test {
    use geo_types::{line_string, polygon, LineString, Polygon};

    use crate::MinimunRotatedRect;

    #[test]
    fn returns_polygon_mbr() {
        let poly: Polygon<f64> = polygon![(x: 3.3, y: 30.4), (x: 1.7, y: 24.6), (x: 13.4, y: 25.1), (x: 14.4, y: 31.0),(x:3.3,y:30.4)];
        let mbr = MinimunRotatedRect::minimun_rotated_rect(&poly).unwrap();
        assert_eq!(
            mbr.exterior(),
            &LineString::from(vec![
                (1.7000000000000006, 24.6),
                (1.4501458363715918, 30.446587428904767),
                (14.4, 31.0),
                (14.649854163628408, 25.153412571095235),
                (1.7000000000000006, 24.6),
            ])
        );
    }
    #[test]
    fn returns_linestring_mbr() {
        let poly: LineString<f64> = line_string![(x: 3.3, y: 30.4), (x: 1.7, y: 24.6), (x: 13.4, y: 25.1), (x: 14.4, y: 31.0)];
        let mbr = MinimunRotatedRect::minimun_rotated_rect(&poly).unwrap();
        assert_eq!(
            mbr.exterior(),
            &LineString::from(vec![
                (1.7000000000000006, 24.6),
                (1.4501458363715918, 30.446587428904767),
                (14.4, 31.0),
                (14.649854163628408, 25.153412571095235),
                (1.7000000000000006, 24.6),
            ])
        );
    }
}
