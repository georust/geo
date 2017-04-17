use num_traits::{Float, ToPrimitive};
use types::{Line, LineString, MultiLineString, MultiPoint, MultiPolygon, Point, Polygon};
use algorithm::contains::Contains;

/// Returns the distance between two geometries.

pub trait Distance<T, Rhs = Self> {
    /// Returns the distance between two geometries
    ///
    /// If a `Point` is contained by a `Polygon`, the distance is `0.0`
    ///
    /// If a `Point` lies on a `Polygon`'s exterior or interior rings, the distance is `0.0`
    ///
    /// If a `Point` lies on a `LineString`, the distance is `0.0`
    ///
    /// The distance between a `Point` and an empty `LineString` is `0.0`
    ///
    /// ```
    /// use geo::{COORD_PRECISION, Point, LineString, Polygon};
    /// use geo::algorithm::distance::Distance;
    ///
    /// // Point to Point example
    /// let p = Point::new(-72.1235, 42.3521);
    /// let dist = p.distance(&Point::new(-72.1260, 42.45));
    /// assert!(dist < COORD_PRECISION);
    ///
    /// // Point to Polygon example
    /// let points = vec![
    ///     (5., 1.),
    ///     (4., 2.),
    ///     (4., 3.),
    ///     (5., 4.),
    ///     (6., 4.),
    ///     (7., 3.),
    ///     (7., 2.),
    ///     (6., 1.),
    ///     (5., 1.)
    /// ];
    /// let ls = LineString(points.iter().map(|e| Point::new(e.0, e.1)).collect());
    /// let poly = Polygon::new(ls, vec![]);
    /// // A Random point outside the polygon
    /// let p = Point::new(2.5, 0.5);
    /// let dist = p.distance(&poly);
    /// assert_eq!(dist, 2.1213203435596424);
    ///
    /// // Point to LineString example
    /// let points = vec![
    ///     (5., 1.),
    ///     (4., 2.),
    ///     (4., 3.),
    ///     (5., 4.),
    ///     (6., 4.),
    ///     (7., 3.),
    ///     (7., 2.),
    ///     (6., 1.),
    /// ];
    /// let ls = LineString(points.iter().map(|e| Point::new(e.0, e.1)).collect());
    /// // A Random point outside the LineString
    /// let p = Point::new(5.5, 2.1);
    /// let dist = p.distance(&ls);
    /// assert_eq!(dist, 1.1313708498984762);
    /// ```
    fn distance(&self, rhs: &Rhs) -> T;
}

// Return minimum distance between a Point and a Line segment
// This is a helper for Point-to-LineString and Point-to-Polygon distance
// adapted from https://github.com/OSGeo/geos/blob/master/src/algorithm/CGAlgorithms.cpp#L191
fn line_segment_distance<T>(point: &Point<T>, start: &Point<T>, end: &Point<T>) -> T
where
    T: Float + ToPrimitive,
{
    if start == end {
        return point.distance(start);
    }
    let dx = end.x() - start.x();
    let dy = end.y() - start.y();
    let r = ((point.x() - start.x()) * dx + (point.y() - start.y()) * dy) / (dx.powi(2) + dy.powi(2));
    if r <= T::zero() {
        return point.distance(start);
    }
    if r >= T::one() {
        return point.distance(end);
    }
    let s = ((start.y() - point.y()) * dx - (start.x() - point.x()) * dy) / (dx * dx + dy * dy);
    s.abs() * (dx * dx + dy * dy).sqrt()
}

impl<T> Distance<T, Point<T>> for Point<T>
where
    T: Float,
{
    /// Minimum distance between two Points
    fn distance(&self, p: &Point<T>) -> T {
        let (dx, dy) = (self.x() - p.x(), self.y() - p.y());
        dx.hypot(dy)
    }
}

impl<T> Distance<T, MultiPoint<T>> for Point<T>
where
    T: Float,
{
    /// Minimum distance from a Point to a MultiPoint
    fn distance(&self, points: &MultiPoint<T>) -> T {
        points
            .0
            .iter()
            .map(|p| {
                let (dx, dy) = (self.x() - p.x(), self.y() - p.y());
                dx.hypot(dy)
            })
            .fold(T::max_value(), |accum, val| accum.min(val))
    }
}

impl<T> Distance<T, Point<T>> for MultiPoint<T>
where
    T: Float,
{
    /// Minimum distance from a MultiPoint to a Point
    fn distance(&self, point: &Point<T>) -> T {
        point.distance(self)
    }
}

impl<T> Distance<T, Polygon<T>> for Point<T>
where
    T: Float,
{
    /// Minimum distance from a Point to a Polygon
    fn distance(&self, polygon: &Polygon<T>) -> T {
        // No need to continue if the polygon contains the point, or is zero-length
        if polygon.contains(self) || polygon.exterior.0.is_empty() {
            return T::zero();
        }
        // fold the minimum interior ring distance if any, followed by the exterior
        // shell distance, returning the minimum of the two distances
        polygon
            .interiors
            .iter()
            .map(|ring| self.distance(ring))
            .fold(T::max_value(), |accum, val| accum.min(val))
            .min(
                polygon
                    .exterior
                    .lines()
                    .map(|line| line_segment_distance(self, &line.start, &line.end))
                    .fold(T::max_value(), |accum, val| accum.min(val)),
            )
    }
}

impl<T> Distance<T, Point<T>> for Polygon<T>
where
    T: Float,
{
    /// Minimum distance from a Polygon to a Point
    fn distance(&self, point: &Point<T>) -> T {
        point.distance(self)
    }
}

impl<T> Distance<T, MultiPolygon<T>> for Point<T>
where
    T: Float,
{
    /// Minimum distance from a Point to a MultiPolygon
    fn distance(&self, mpolygon: &MultiPolygon<T>) -> T {
        mpolygon
            .0
            .iter()
            .map(|p| self.distance(p))
            .fold(T::max_value(), |accum, val| accum.min(val))
    }
}

impl<T> Distance<T, Point<T>> for MultiPolygon<T>
where
    T: Float,
{
    /// Minimum distance from a MultiPolygon to a Point
    fn distance(&self, point: &Point<T>) -> T {
        point.distance(self)
    }
}

impl<T> Distance<T, MultiLineString<T>> for Point<T>
where
    T: Float,
{
    /// Minimum distance from a Point to a MultiLineString
    fn distance(&self, mls: &MultiLineString<T>) -> T {
        mls.0
            .iter()
            .map(|ls| self.distance(ls))
            .fold(T::max_value(), |accum, val| accum.min(val))
    }
}

impl<T> Distance<T, Point<T>> for MultiLineString<T>
where
    T: Float,
{
    /// Minimum distance from a MultiLineString to a Point
    fn distance(&self, point: &Point<T>) -> T {
        point.distance(self)
    }
}

impl<T> Distance<T, LineString<T>> for Point<T>
where
    T: Float,
{
    /// Minimum distance from a Point to a LineString
    fn distance(&self, linestring: &LineString<T>) -> T {
        // No need to continue if the point is on the LineString, or it's empty
        if linestring.contains(self) || linestring.0.is_empty() {
            return T::zero();
        }
        linestring
            .lines()
            .map(|line| line_segment_distance(self, &line.start, &line.end))
            .fold(T::max_value(), |accum, val| accum.min(val))
    }
}

impl<T> Distance<T, Point<T>> for LineString<T>
where
    T: Float,
{
    /// Minimum distance from a LineString to a Point
    fn distance(&self, point: &Point<T>) -> T {
        point.distance(self)
    }
}

impl<T> Distance<T, Point<T>> for Line<T>
where
    T: Float,
{
    /// Minimum distance from a Line to a Point
    fn distance(&self, point: &Point<T>) -> T {
        line_segment_distance(point, &self.start, &self.end)
    }
}
impl<T> Distance<T, Line<T>> for Point<T>
where
    T: Float,
{
    /// Minimum distance from a Line to a Point
    fn distance(&self, line: &Line<T>) -> T {
        line.distance(self)
    }
}

#[cfg(test)]
mod test {
    use types::{Line, LineString, MultiLineString, MultiPoint, MultiPolygon, Point, Polygon};
    use algorithm::distance::{line_segment_distance, Distance};

    #[test]
    fn line_segment_distance_test() {
        let o1 = Point::new(8.0, 0.0);
        let o2 = Point::new(5.5, 0.0);
        let o3 = Point::new(5.0, 0.0);
        let o4 = Point::new(4.5, 1.5);

        let p1 = Point::new(7.2, 2.0);
        let p2 = Point::new(6.0, 1.0);

        let dist = line_segment_distance(&o1, &p1, &p2);
        let dist2 = line_segment_distance(&o2, &p1, &p2);
        let dist3 = line_segment_distance(&o3, &p1, &p2);
        let dist4 = line_segment_distance(&o4, &p1, &p2);
        // Results agree with Shapely
        assert_relative_eq!(dist, 2.0485900789263356);
        assert_relative_eq!(dist2, 1.118033988749895);
        assert_relative_eq!(dist3, 1.4142135623730951);
        assert_relative_eq!(dist4, 1.5811388300841898);
        // Point is on the line
        let zero_dist = line_segment_distance(&p1, &p1, &p2);
        assert_relative_eq!(zero_dist, 0.0);
    }
    #[test]
    // Point to Polygon, outside point
    fn point_polygon_distance_outside_test() {
        // an octagon
        let points = vec![
            (5., 1.),
            (4., 2.),
            (4., 3.),
            (5., 4.),
            (6., 4.),
            (7., 3.),
            (7., 2.),
            (6., 1.),
            (5., 1.),
        ];
        let ls = LineString(points.iter().map(|e| Point::new(e.0, e.1)).collect());
        let poly = Polygon::new(ls, vec![]);
        // A Random point outside the octagon
        let p = Point::new(2.5, 0.5);
        let dist = p.distance(&poly);
        assert_relative_eq!(dist, 2.1213203435596424);
    }
    #[test]
    // Point to Polygon, inside point
    fn point_polygon_distance_inside_test() {
        // an octagon
        let points = vec![
            (5., 1.),
            (4., 2.),
            (4., 3.),
            (5., 4.),
            (6., 4.),
            (7., 3.),
            (7., 2.),
            (6., 1.),
            (5., 1.),
        ];
        let ls = LineString(points.iter().map(|e| Point::new(e.0, e.1)).collect());
        let poly = Polygon::new(ls, vec![]);
        // A Random point inside the octagon
        let p = Point::new(5.5, 2.1);
        let dist = p.distance(&poly);
        assert_relative_eq!(dist, 0.0);
    }
    #[test]
    // Point to Polygon, on boundary
    fn point_polygon_distance_boundary_test() {
        // an octagon
        let points = vec![
            (5., 1.),
            (4., 2.),
            (4., 3.),
            (5., 4.),
            (6., 4.),
            (7., 3.),
            (7., 2.),
            (6., 1.),
            (5., 1.),
        ];
        let ls = LineString(points.iter().map(|e| Point::new(e.0, e.1)).collect());
        let poly = Polygon::new(ls, vec![]);
        // A point on the octagon
        let p = Point::new(5.0, 1.0);
        let dist = p.distance(&poly);
        assert_relative_eq!(dist, 0.0);
    }
    #[test]
    // Point to Polygon, on boundary
    fn flibble() {
        let exterior = LineString(vec![
            Point::new(0., 0.),
            Point::new(0., 0.0004),
            Point::new(0.0004, 0.0004),
            Point::new(0.0004, 0.),
            Point::new(0., 0.),
        ]);

        let poly = Polygon::new(exterior.clone(), vec![]);
        let bugged_point = Point::new(0.0001, 0.);
        assert_eq!(poly.distance(&bugged_point), 0.);
    }
    #[test]
    // Point to Polygon, empty Polygon
    fn point_polygon_empty_test() {
        // an empty Polygon
        let points = vec![];
        let ls = LineString(points);
        let poly = Polygon::new(ls, vec![]);
        // A point on the octagon
        let p = Point::new(2.5, 0.5);
        let dist = p.distance(&poly);
        assert_relative_eq!(dist, 0.0);
    }
    #[test]
    // Point to Polygon with an interior ring
    fn point_polygon_interior_cutout_test() {
        // an octagon
        let ext_points = vec![
            (4., 1.),
            (5., 2.),
            (5., 3.),
            (4., 4.),
            (3., 4.),
            (2., 3.),
            (2., 2.),
            (3., 1.),
            (4., 1.),
        ];
        // cut out a triangle inside octagon
        let int_points = vec![(3.5, 3.5), (4.4, 1.5), (2.6, 1.5), (3.5, 3.5)];
        let ls_ext = LineString(ext_points.iter().map(|e| Point::new(e.0, e.1)).collect());
        let ls_int = LineString(int_points.iter().map(|e| Point::new(e.0, e.1)).collect());
        let poly = Polygon::new(ls_ext, vec![ls_int]);
        // A point inside the cutout triangle
        let p = Point::new(3.5, 2.5);
        let dist = p.distance(&poly);
        // 0.41036467732879783 <-- Shapely
        assert_relative_eq!(dist, 0.41036467732879767);
    }
    #[test]
    fn point_distance_multipolygon_test() {
        let ls1 = LineString(vec![
            Point::new(0.0, 0.0),
            Point::new(1.0, 10.0),
            Point::new(2.0, 0.0),
            Point::new(0.0, 0.0),
        ]);
        let ls2 = LineString(vec![
            Point::new(3.0, 0.0),
            Point::new(4.0, 10.0),
            Point::new(5.0, 0.0),
            Point::new(3.0, 0.0),
        ]);
        let p1 = Polygon::new(ls1, vec![]);
        let p2 = Polygon::new(ls2, vec![]);
        let mp = MultiPolygon(vec![p1, p2]);
        let p = Point::new(50.0, 50.0);
        assert_relative_eq!(p.distance(&mp), 60.959002616512684);
    }
    #[test]
    // Point to LineString
    fn point_linestring_distance_test() {
        // like an octagon, but missing the lowest horizontal segment
        let points = vec![
            (5., 1.),
            (4., 2.),
            (4., 3.),
            (5., 4.),
            (6., 4.),
            (7., 3.),
            (7., 2.),
            (6., 1.),
        ];
        let ls = LineString(points.iter().map(|e| Point::new(e.0, e.1)).collect());
        // A Random point "inside" the LineString
        let p = Point::new(5.5, 2.1);
        let dist = p.distance(&ls);
        assert_relative_eq!(dist, 1.1313708498984762);
    }
    #[test]
    // Point to LineString, point lies on the LineString
    fn point_linestring_contains_test() {
        // like an octagon, but missing the lowest horizontal segment
        let points = vec![
            (5., 1.),
            (4., 2.),
            (4., 3.),
            (5., 4.),
            (6., 4.),
            (7., 3.),
            (7., 2.),
            (6., 1.),
        ];
        let ls = LineString(points.iter().map(|e| Point::new(e.0, e.1)).collect());
        // A point which lies on the LineString
        let p = Point::new(5.0, 4.0);
        let dist = p.distance(&ls);
        assert_relative_eq!(dist, 0.0);
    }
    #[test]
    // Point to LineString, closed triangle
    fn point_linestring_triangle_test() {
        let points = vec![(3.5, 3.5), (4.4, 2.0), (2.6, 2.0), (3.5, 3.5)];
        let ls = LineString(points.iter().map(|e| Point::new(e.0, e.1)).collect());
        let p = Point::new(3.5, 2.5);
        let dist = p.distance(&ls);
        assert_relative_eq!(dist, 0.5);
    }
    #[test]
    // Point to LineString, empty LineString
    fn point_linestring_empty_test() {
        let points = vec![];
        let ls = LineString(points);
        let p = Point::new(5.0, 4.0);
        let dist = p.distance(&ls);
        assert_relative_eq!(dist, 0.0);
    }
    #[test]
    fn distance_multilinestring_test() {
        let v1 = LineString(vec![Point::new(0.0, 0.0), Point::new(1.0, 10.0)]);
        let v2 = LineString(vec![
            Point::new(1.0, 10.0),
            Point::new(2.0, 0.0),
            Point::new(3.0, 1.0),
        ]);
        let mls = MultiLineString(vec![v1, v2]);
        let p = Point::new(50.0, 50.0);
        assert_relative_eq!(p.distance(&mls), 63.25345840347388);
    }
    #[test]
    fn distance1_test() {
        assert_eq!(
            Point::<f64>::new(0., 0.).distance(&Point::<f64>::new(1., 0.)),
            1.
        );
    }
    #[test]
    fn distance2_test() {
        let dist = Point::new(-72.1235, 42.3521).distance(&Point::new(72.1260, 70.612));
        assert_relative_eq!(dist, 146.99163308930207);
    }
    #[test]
    fn distance_multipoint_test() {
        let v = vec![
            Point::new(0.0, 10.0),
            Point::new(1.0, 1.0),
            Point::new(10.0, 0.0),
            Point::new(1.0, -1.0),
            Point::new(0.0, -10.0),
            Point::new(-1.0, -1.0),
            Point::new(-10.0, 0.0),
            Point::new(-1.0, 1.0),
            Point::new(0.0, 10.0),
        ];
        let mp = MultiPoint(v);
        let p = Point::new(50.0, 50.0);
        assert_eq!(p.distance(&mp), 64.03124237432849)
    }
    #[test]
    // test edge-vertex minimum distance
    fn test_minimum_polygon_distance() {
        let points_raw = vec![(126., 232.),
                              (126., 212.),
                              (112., 202.),
                              (97., 204.),
                              (87., 215.),
                              (87., 232.),
                              (100., 246.),
                              (118., 247.)];
        let points = points_raw
            .iter()
            .map(|e| Point::new(e.0, e.1))
            .collect::<Vec<_>>();
        let poly1 = Polygon::new(LineString(points), vec![]);

        let points_raw_2 = vec![(188., 231.),
                                (189., 207.),
                                (174., 196.),
                                (164., 196.),
                                (147., 220.),
                                (158., 242.),
                                (177., 242.)];
        let points2 = points_raw_2
            .iter()
            .map(|e| Point::new(e.0, e.1))
            .collect::<Vec<_>>();
        let poly2 = Polygon::new(LineString(points2), vec![]);
        let dist = min_poly_dist(&poly1.convex_hull(), &poly2.convex_hull());
        assert_eq!(dist, 21.0);
    }
    #[test]
    // test vertex-vertex minimum distance
    fn test_minimum_polygon_distance_2() {
        let points_raw = vec![(118., 200.), (153., 179.), (106., 155.), (88., 190.), (118., 200.)];
        let points = points_raw
            .iter()
            .map(|e| Point::new(e.0, e.1))
            .collect::<Vec<_>>();
        let poly1 = Polygon::new(LineString(points), vec![]);

        let points_raw_2 =
            vec![(242., 186.), (260., 146.), (182., 175.), (216., 193.), (242., 186.)];
        let points2 = points_raw_2
            .iter()
            .map(|e| Point::new(e.0, e.1))
            .collect::<Vec<_>>();
        let poly2 = Polygon::new(LineString(points2), vec![]);
        let dist = min_poly_dist(&poly1.convex_hull(), &poly2.convex_hull());
        assert_eq!(dist, 29.274562336608895);
    }
    #[test]
    // test edge-edge minimum distance
    fn test_minimum_polygon_distance_3() {
        let points_raw = vec![(182., 182.), (182., 168.), (138., 160.), (136., 193.), (182., 182.)];
        let points = points_raw
            .iter()
            .map(|e| Point::new(e.0, e.1))
            .collect::<Vec<_>>();
        let poly1 = Polygon::new(LineString(points), vec![]);

        let points_raw_2 =
            vec![(232., 196.), (234., 150.), (194., 165.), (194., 191.), (232., 196.)];
        let points2 = points_raw_2
            .iter()
            .map(|e| Point::new(e.0, e.1))
            .collect::<Vec<_>>();
        let poly2 = Polygon::new(LineString(points2), vec![]);
        let dist = min_poly_dist(&poly1.convex_hull(), &poly2.convex_hull());
        assert_eq!(dist, 12.0);
    }
    #[test]
    fn test_vertex_line_distance() {
        let p = Point::new(0., 0.);
        let q = Point::new(3.8, 5.7);
        let r = Point::new(22.5, 10.);
        let dist = p.vertex_line_distance(&q, &r);
        assert_eq!(dist, 6.850547423381579);
    }
}
