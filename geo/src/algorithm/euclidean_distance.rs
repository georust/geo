use crate::algorithm::contains::Contains;
use crate::algorithm::euclidean_length::EuclideanLength;
use crate::algorithm::intersects::Intersects;
use crate::algorithm::polygon_distance_fast_path::*;
use crate::kernels::*;
use crate::utils::{coord_pos_relative_to_ring, CoordPos};
use crate::{
    Line, LineString, MultiLineString, MultiPoint, MultiPolygon, Point, Polygon, Triangle,
};
use num_traits::float::FloatConst;
use num_traits::{Bounded, Float, Signed};

use rstar::RTree;
use rstar::RTreeNum;

/// Returns the distance between two geometries.

pub trait EuclideanDistance<T, Rhs = Self> {
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
    /// # Examples
    ///
    /// `Point` to `Point`:
    ///
    /// ```
    /// use approx::assert_relative_eq;
    /// use geo::algorithm::euclidean_distance::EuclideanDistance;
    /// use geo::point;
    ///
    /// let p1 = point!(x: -72.1235, y: 42.3521);
    /// let p2 = point!(x: -72.1260, y: 42.45);
    ///
    /// let distance = p1.euclidean_distance(&p2);
    ///
    /// assert_relative_eq!(distance, 0.09793191512474639);
    /// ```
    ///
    /// `Point` to `Polygon`:
    ///
    /// ```
    /// use approx::assert_relative_eq;
    /// use geo::algorithm::euclidean_distance::EuclideanDistance;
    /// use geo::{point, polygon};
    ///
    /// let polygon = polygon![
    ///     (x: 5., y: 1.),
    ///     (x: 4., y: 2.),
    ///     (x: 4., y: 3.),
    ///     (x: 5., y: 4.),
    ///     (x: 6., y: 4.),
    ///     (x: 7., y: 3.),
    ///     (x: 7., y: 2.),
    ///     (x: 6., y: 1.),
    ///     (x: 5., y: 1.),
    /// ];
    ///
    /// let point = point!(x: 2.5, y: 0.5);
    ///
    /// let distance = point.euclidean_distance(&polygon);
    ///
    /// assert_relative_eq!(distance, 2.1213203435596424);
    /// ```
    ///
    /// `Point` to `LineString`:
    ///
    /// ```
    /// use approx::assert_relative_eq;
    /// use geo::algorithm::euclidean_distance::EuclideanDistance;
    /// use geo::{point, line_string};
    ///
    /// let line_string = line_string![
    ///     (x: 5., y: 1.),
    ///     (x: 4., y: 2.),
    ///     (x: 4., y: 3.),
    ///     (x: 5., y: 4.),
    ///     (x: 6., y: 4.),
    ///     (x: 7., y: 3.),
    ///     (x: 7., y: 2.),
    ///     (x: 6., y: 1.),
    /// ];
    ///
    /// let point = point!(x: 5.5, y: 2.1);
    ///
    /// let distance = point.euclidean_distance(&line_string);
    ///
    /// assert_relative_eq!(distance, 1.1313708498984762);
    /// ```
    fn euclidean_distance(&self, rhs: &Rhs) -> T;
}

impl<T> EuclideanDistance<T, Point<T>> for Point<T>
where
    T: Float,
{
    /// Minimum distance between two Points
    fn euclidean_distance(&self, p: &Point<T>) -> T {
        Line::new(self.0, p.0).euclidean_length()
    }
}

impl<T> EuclideanDistance<T, MultiPoint<T>> for Point<T>
where
    T: Float,
{
    /// Minimum distance from a Point to a MultiPoint
    fn euclidean_distance(&self, points: &MultiPoint<T>) -> T {
        points
            .0
            .iter()
            .map(|p| self.euclidean_distance(p))
            .fold(T::max_value(), |accum, val| accum.min(val))
    }
}

impl<T> EuclideanDistance<T, Point<T>> for MultiPoint<T>
where
    T: Float,
{
    /// Minimum distance from a MultiPoint to a Point
    fn euclidean_distance(&self, point: &Point<T>) -> T {
        point.euclidean_distance(self)
    }
}

impl<T> EuclideanDistance<T, Polygon<T>> for Point<T>
where
    T: Float + HasKernel,
{
    /// Minimum distance from a Point to a Polygon
    fn euclidean_distance(&self, polygon: &Polygon<T>) -> T {
        // No need to continue if the polygon contains the point, or is zero-length
        if polygon.contains(self) || polygon.exterior().0.is_empty() {
            return T::zero();
        }
        // fold the minimum interior ring distance if any, followed by the exterior
        // shell distance, returning the minimum of the two distances
        polygon
            .interiors()
            .iter()
            .map(|ring| self.euclidean_distance(ring))
            .fold(T::max_value(), |accum, val| accum.min(val))
            .min(
                polygon
                    .exterior()
                    .lines()
                    .map(|line| {
                        ::geo_types::private_utils::line_segment_distance(
                            *self,
                            line.start_point(),
                            line.end_point(),
                        )
                    })
                    .fold(T::max_value(), |accum, val| accum.min(val)),
            )
    }
}

impl<T> EuclideanDistance<T, Point<T>> for Polygon<T>
where
    T: Float + HasKernel,
{
    /// Minimum distance from a Polygon to a Point
    fn euclidean_distance(&self, point: &Point<T>) -> T {
        point.euclidean_distance(self)
    }
}

impl<T> EuclideanDistance<T, MultiPolygon<T>> for Point<T>
where
    T: Float + HasKernel,
{
    /// Minimum distance from a Point to a MultiPolygon
    fn euclidean_distance(&self, mpolygon: &MultiPolygon<T>) -> T {
        mpolygon
            .0
            .iter()
            .map(|p| self.euclidean_distance(p))
            .fold(T::max_value(), |accum, val| accum.min(val))
    }
}

impl<T> EuclideanDistance<T, Point<T>> for MultiPolygon<T>
where
    T: Float + HasKernel,
{
    /// Minimum distance from a MultiPolygon to a Point
    fn euclidean_distance(&self, point: &Point<T>) -> T {
        point.euclidean_distance(self)
    }
}

impl<T> EuclideanDistance<T, MultiLineString<T>> for Point<T>
where
    T: Float,
{
    /// Minimum distance from a Point to a MultiLineString
    fn euclidean_distance(&self, mls: &MultiLineString<T>) -> T {
        mls.0
            .iter()
            .map(|ls| self.euclidean_distance(ls))
            .fold(T::max_value(), |accum, val| accum.min(val))
    }
}

impl<T> EuclideanDistance<T, Point<T>> for MultiLineString<T>
where
    T: Float,
{
    /// Minimum distance from a MultiLineString to a Point
    fn euclidean_distance(&self, point: &Point<T>) -> T {
        point.euclidean_distance(self)
    }
}

impl<T> EuclideanDistance<T, LineString<T>> for Point<T>
where
    T: Float,
{
    /// Minimum distance from a Point to a LineString
    fn euclidean_distance(&self, linestring: &LineString<T>) -> T {
        ::geo_types::private_utils::point_line_string_euclidean_distance(*self, linestring)
    }
}

impl<T> EuclideanDistance<T, Point<T>> for LineString<T>
where
    T: Float,
{
    /// Minimum distance from a LineString to a Point
    fn euclidean_distance(&self, point: &Point<T>) -> T {
        point.euclidean_distance(self)
    }
}

impl<T> EuclideanDistance<T, Point<T>> for Line<T>
where
    T: Float,
{
    /// Minimum distance from a Line to a Point
    fn euclidean_distance(&self, point: &Point<T>) -> T {
        ::geo_types::private_utils::point_line_euclidean_distance(*point, *self)
    }
}

impl<T> EuclideanDistance<T, Line<T>> for Point<T>
where
    T: Float,
{
    /// Minimum distance from a Line to a Point
    fn euclidean_distance(&self, line: &Line<T>) -> T {
        line.euclidean_distance(self)
    }
}

/// LineString-LineString distance
impl<T> EuclideanDistance<T, LineString<T>> for LineString<T>
where
    T: Float + HasKernel + Signed + RTreeNum,
{
    fn euclidean_distance(&self, other: &LineString<T>) -> T {
        if self.intersects(other) {
            T::zero()
        } else {
            nearest_neighbour_distance(self, other)
        }
    }
}

/// This method handles a corner case in which a candidate polygon
/// is disjoint because it's contained in the inner ring
/// we work around this by checking that Polygons with inner rings don't
/// contain a point from the candidate Polygon's outer shell in their simple representations
fn ring_contains_point<T>(poly: &Polygon<T>, p: Point<T>) -> bool
where
    T: HasKernel,
{
    match coord_pos_relative_to_ring(p.0, &poly.exterior()) {
        CoordPos::Inside => true,
        CoordPos::OnBoundary | CoordPos::Outside => false,
    }
}

/// LineString to Line
impl<T> EuclideanDistance<T, Line<T>> for LineString<T>
where
    T: Float + FloatConst + Signed + RTreeNum + HasKernel,
{
    fn euclidean_distance(&self, other: &Line<T>) -> T {
        self.lines().fold(Bounded::max_value(), |acc, line| {
            acc.min(line.euclidean_distance(other))
        })
    }
}

/// Line to LineString
impl<T> EuclideanDistance<T, LineString<T>> for Line<T>
where
    T: Float + FloatConst + Signed + RTreeNum + HasKernel,
{
    fn euclidean_distance(&self, other: &LineString<T>) -> T {
        other.euclidean_distance(self)
    }
}

/// LineString to Polygon
impl<T> EuclideanDistance<T, Polygon<T>> for LineString<T>
where
    T: Float + FloatConst + Signed + RTreeNum + HasKernel,
{
    fn euclidean_distance(&self, other: &Polygon<T>) -> T {
        if self.intersects(other) || other.contains(self) {
            T::zero()
        } else if !other.interiors().is_empty() && ring_contains_point(other, Point(self.0[0])) {
            // check each ring distance, returning the minimum
            let mut mindist: T = Float::max_value();
            for ring in other.interiors() {
                mindist = mindist.min(nearest_neighbour_distance(self, ring))
            }
            mindist
        } else {
            nearest_neighbour_distance(self, &other.exterior())
        }
    }
}

/// Polygon to LineString distance
impl<T> EuclideanDistance<T, LineString<T>> for Polygon<T>
where
    T: Float + FloatConst + Signed + RTreeNum + HasKernel,
{
    fn euclidean_distance(&self, other: &LineString<T>) -> T {
        other.euclidean_distance(self)
    }
}

/// Line to MultiPolygon distance
impl<T> EuclideanDistance<T, MultiPolygon<T>> for Line<T>
where
    T: Float + FloatConst + Signed + RTreeNum + HasKernel,
{
    fn euclidean_distance(&self, mpolygon: &MultiPolygon<T>) -> T {
        mpolygon
            .0
            .iter()
            .map(|p| self.euclidean_distance(p))
            .fold(Bounded::max_value(), |accum, val| accum.min(val))
    }
}

/// MultiPolygon to Line distance
impl<T> EuclideanDistance<T, Line<T>> for MultiPolygon<T>
where
    T: Float + FloatConst + Signed + RTreeNum + HasKernel,
{
    fn euclidean_distance(&self, other: &Line<T>) -> T {
        other.euclidean_distance(self)
    }
}

/// Line to Line distance
impl<T> EuclideanDistance<T, Line<T>> for Line<T>
where
    T: Float + FloatConst + Signed + RTreeNum + HasKernel,
{
    fn euclidean_distance(&self, other: &Line<T>) -> T {
        if self.intersects(other) || self.contains(other) {
            return T::zero();
        }
        // minimum of all Point-Line distances
        self.start_point()
            .euclidean_distance(other)
            .min(self.end_point().euclidean_distance(other))
            .min(other.start_point().euclidean_distance(self))
            .min(other.end_point().euclidean_distance(self))
    }
}

// Line to Polygon distance
impl<T> EuclideanDistance<T, Polygon<T>> for Line<T>
where
    T: Float + Signed + RTreeNum + FloatConst + HasKernel,
{
    fn euclidean_distance(&self, other: &Polygon<T>) -> T {
        if other.contains(self) || self.intersects(other) {
            return T::zero();
        }
        // line-line distance between each exterior polygon segment and the line
        let exterior_min = other
            .exterior()
            .lines()
            .fold(<T as Bounded>::max_value(), |acc, point| {
                acc.min(self.euclidean_distance(&point))
            });
        // line-line distance between each interior ring segment and the line
        // if there are no rings this just evaluates to max_float
        let interior_min = other
            .interiors()
            .iter()
            .map(|ring| {
                ring.lines().fold(<T as Bounded>::max_value(), |acc, line| {
                    acc.min(self.euclidean_distance(&line))
                })
            })
            .fold(<T as Bounded>::max_value(), |acc, ring_min| {
                acc.min(ring_min)
            });
        // return smaller of the two values
        exterior_min.min(interior_min)
    }
}

// Polygon to Line distance
impl<T> EuclideanDistance<T, Line<T>> for Polygon<T>
where
    T: Float + FloatConst + Signed + RTreeNum + HasKernel,
{
    fn euclidean_distance(&self, other: &Line<T>) -> T {
        other.euclidean_distance(self)
    }
}

// Polygon to Polygon distance
impl<T> EuclideanDistance<T, Polygon<T>> for Polygon<T>
where
    T: Float + FloatConst + RTreeNum + HasKernel,
{
    /// This implementation has a "fast path" in cases where both input polygons are convex:
    /// it switches to an implementation of the "rotating calipers" method described in [Pirzadeh (1999), pp24—30](http://digitool.library.mcgill.ca/R/?func=dbin-jump-full&object_id=21623&local_base=GEN01-MCG02),
    ///  which is approximately an order of magnitude faster than the standard method.
    fn euclidean_distance(&self, poly2: &Polygon<T>) -> T {
        if self.intersects(poly2) {
            return T::zero();
        }
        // Containment check
        if !self.interiors().is_empty() && ring_contains_point(self, Point(poly2.exterior().0[0])) {
            // check each ring distance, returning the minimum
            let mut mindist: T = Float::max_value();
            for ring in self.interiors() {
                mindist = mindist.min(nearest_neighbour_distance(&poly2.exterior(), ring))
            }
            return mindist;
        } else if !poly2.interiors().is_empty()
            && ring_contains_point(poly2, Point(self.exterior().0[0]))
        {
            let mut mindist: T = Float::max_value();
            for ring in poly2.interiors() {
                mindist = mindist.min(nearest_neighbour_distance(&self.exterior(), ring))
            }
            return mindist;
        }
        use super::is_convex::IsConvex;
        if !poly2.exterior().is_convex() || !self.exterior().is_convex() {
            // fall back to R* nearest neighbour method
            nearest_neighbour_distance(&self.exterior(), &poly2.exterior())
        } else {
            min_poly_dist(self, poly2)
        }
    }
}

impl<T> EuclideanDistance<T, Point<T>> for Triangle<T>
where
    T: Float + HasKernel,
{
    fn euclidean_distance(&self, point: &Point<T>) -> T {
        if self.contains(point) {
            return T::zero();
        }

        [(self.0, self.1), (self.1, self.2), (self.2, self.0)]
            .iter()
            .map(|edge| {
                ::geo_types::private_utils::line_segment_distance(
                    *point,
                    edge.0.into(),
                    edge.1.into(),
                )
            })
            .fold(T::max_value(), |accum, val| accum.min(val))
    }
}
/// Uses an R* tree and nearest-neighbour lookups to calculate minimum distances
// This is somewhat slow and memory-inefficient, but certainly better than quadratic time
pub fn nearest_neighbour_distance<T>(geom1: &LineString<T>, geom2: &LineString<T>) -> T
where
    T: Float + RTreeNum,
{
    let tree_a: RTree<Line<_>> = RTree::bulk_load(geom1.lines().collect::<Vec<_>>());
    let tree_b: RTree<Line<_>> = RTree::bulk_load(geom2.lines().collect::<Vec<_>>());
    // Return minimum distance between all geom a points and all geom b points
    geom2
        .points_iter()
        .fold(<T as Bounded>::max_value(), |acc, point| {
            let nearest = tree_a.nearest_neighbor(&point).unwrap();
            acc.min(nearest.euclidean_distance(&point))
        })
        .min(
            geom1
                .points_iter()
                .fold(Bounded::max_value(), |acc, point| {
                    let nearest = tree_b.nearest_neighbor(&point).unwrap();
                    acc.min(nearest.euclidean_distance(&point))
                }),
        )
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::algorithm::convex_hull::ConvexHull;
    use crate::algorithm::euclidean_distance::EuclideanDistance;
    use crate::{Line, LineString, MultiLineString, MultiPoint, MultiPolygon, Point, Polygon};
    use geo_types::{polygon, private_utils::line_segment_distance, Coordinate};

    #[test]
    fn line_segment_distance_test() {
        let o1 = Point::new(8.0, 0.0);
        let o2 = Point::new(5.5, 0.0);
        let o3 = Point::new(5.0, 0.0);
        let o4 = Point::new(4.5, 1.5);

        let p1 = Point::new(7.2, 2.0);
        let p2 = Point::new(6.0, 1.0);

        let dist = line_segment_distance(o1, p1, p2);
        let dist2 = line_segment_distance(o2, p1, p2);
        let dist3 = line_segment_distance(o3, p1, p2);
        let dist4 = line_segment_distance(o4, p1, p2);
        // Results agree with Shapely
        assert_relative_eq!(dist, 2.0485900789263356);
        assert_relative_eq!(dist2, 1.118033988749895);
        assert_relative_eq!(dist3, std::f64::consts::SQRT_2); // workaround clippy::correctness error approx_constant (1.4142135623730951)
        assert_relative_eq!(dist4, 1.5811388300841898);
        // Point is on the line
        let zero_dist = line_segment_distance(p1, p1, p2);
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
        let ls = LineString::from(points);
        let poly = Polygon::new(ls, vec![]);
        // A Random point outside the octagon
        let p = Point::new(2.5, 0.5);
        let dist = p.euclidean_distance(&poly);
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
        let ls = LineString::from(points);
        let poly = Polygon::new(ls, vec![]);
        // A Random point inside the octagon
        let p = Point::new(5.5, 2.1);
        let dist = p.euclidean_distance(&poly);
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
        let ls = LineString::from(points);
        let poly = Polygon::new(ls, vec![]);
        // A point on the octagon
        let p = Point::new(5.0, 1.0);
        let dist = p.euclidean_distance(&poly);
        assert_relative_eq!(dist, 0.0);
    }
    #[test]
    // Point to Polygon, on boundary
    fn point_polygon_boundary_test2() {
        let exterior = LineString::from(vec![
            (0., 0.),
            (0., 0.0004),
            (0.0004, 0.0004),
            (0.0004, 0.),
            (0., 0.),
        ]);

        let poly = Polygon::new(exterior.clone(), vec![]);
        let bugged_point = Point::new(0.0001, 0.);
        assert_relative_eq!(poly.euclidean_distance(&bugged_point), 0.);
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
        let dist = p.euclidean_distance(&poly);
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
        let ls_ext = LineString::from(ext_points);
        let ls_int = LineString::from(int_points);
        let poly = Polygon::new(ls_ext, vec![ls_int]);
        // A point inside the cutout triangle
        let p = Point::new(3.5, 2.5);
        let dist = p.euclidean_distance(&poly);

        // 0.41036467732879783 <-- Shapely
        assert_relative_eq!(dist, 0.41036467732879767);
    }

    #[test]
    fn line_distance_multipolygon_do_not_intersect_test() {
        // checks that the distance from the multipolygon
        // is equal to the distance from the closest polygon
        // taken in isolation, whatever that distance is
        let ls1 = LineString::from(vec![
            (0.0, 0.0),
            (10.0, 0.0),
            (10.0, 10.0),
            (5.0, 15.0),
            (0.0, 10.0),
            (0.0, 0.0),
        ]);
        let ls2 = LineString::from(vec![
            (0.0, 30.0),
            (0.0, 25.0),
            (10.0, 25.0),
            (10.0, 30.0),
            (0.0, 30.0),
        ]);
        let ls3 = LineString::from(vec![
            (15.0, 30.0),
            (15.0, 25.0),
            (20.0, 25.0),
            (20.0, 30.0),
            (15.0, 30.0),
        ]);
        let pol1 = Polygon::new(ls1, vec![]);
        let pol2 = Polygon::new(ls2, vec![]);
        let pol3 = Polygon::new(ls3, vec![]);
        let mp = MultiPolygon(vec![pol1.clone(), pol2.clone(), pol3.clone()]);
        let pnt1 = Point::new(0.0, 15.0);
        let pnt2 = Point::new(10.0, 20.0);
        let ln = Line::new(pnt1.0, pnt2.0);
        let dist_mp_ln = ln.euclidean_distance(&mp);
        let dist_pol1_ln = ln.euclidean_distance(&pol1);
        assert_relative_eq!(dist_mp_ln, dist_pol1_ln);
    }

    #[test]
    fn point_distance_multipolygon_test() {
        let ls1 = LineString::from(vec![(0.0, 0.0), (1.0, 10.0), (2.0, 0.0), (0.0, 0.0)]);
        let ls2 = LineString::from(vec![(3.0, 0.0), (4.0, 10.0), (5.0, 0.0), (3.0, 0.0)]);
        let p1 = Polygon::new(ls1, vec![]);
        let p2 = Polygon::new(ls2, vec![]);
        let mp = MultiPolygon(vec![p1, p2]);
        let p = Point::new(50.0, 50.0);
        assert_relative_eq!(p.euclidean_distance(&mp), 60.959002616512684);
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
        let ls = LineString::from(points);
        // A Random point "inside" the LineString
        let p = Point::new(5.5, 2.1);
        let dist = p.euclidean_distance(&ls);
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
        let ls = LineString::from(points);
        // A point which lies on the LineString
        let p = Point::new(5.0, 4.0);
        let dist = p.euclidean_distance(&ls);
        assert_relative_eq!(dist, 0.0);
    }
    #[test]
    // Point to LineString, closed triangle
    fn point_linestring_triangle_test() {
        let points = vec![(3.5, 3.5), (4.4, 2.0), (2.6, 2.0), (3.5, 3.5)];
        let ls = LineString::from(points);
        let p = Point::new(3.5, 2.5);
        let dist = p.euclidean_distance(&ls);
        assert_relative_eq!(dist, 0.5);
    }
    #[test]
    // Point to LineString, empty LineString
    fn point_linestring_empty_test() {
        let points = vec![];
        let ls = LineString(points);
        let p = Point::new(5.0, 4.0);
        let dist = p.euclidean_distance(&ls);
        assert_relative_eq!(dist, 0.0);
    }
    #[test]
    fn distance_multilinestring_test() {
        let v1 = LineString::from(vec![(0.0, 0.0), (1.0, 10.0)]);
        let v2 = LineString::from(vec![(1.0, 10.0), (2.0, 0.0), (3.0, 1.0)]);
        let mls = MultiLineString(vec![v1, v2]);
        let p = Point::new(50.0, 50.0);
        assert_relative_eq!(p.euclidean_distance(&mls), 63.25345840347388);
    }
    #[test]
    fn distance1_test() {
        assert_relative_eq!(
            Point::<f64>::new(0., 0.).euclidean_distance(&Point::<f64>::new(1., 0.)),
            1.
        );
    }
    #[test]
    fn distance2_test() {
        let dist = Point::new(-72.1235, 42.3521).euclidean_distance(&Point::new(72.1260, 70.612));
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
        assert_relative_eq!(p.euclidean_distance(&mp), 64.03124237432849)
    }
    #[test]
    fn distance_line_test() {
        let line0 = Line::from([(0., 0.), (5., 0.)]);
        let p0 = Point::new(2., 3.);
        let p1 = Point::new(3., 0.);
        let p2 = Point::new(6., 0.);
        assert_relative_eq!(line0.euclidean_distance(&p0), 3.);
        assert_relative_eq!(p0.euclidean_distance(&line0), 3.);

        assert_relative_eq!(line0.euclidean_distance(&p1), 0.);
        assert_relative_eq!(p1.euclidean_distance(&line0), 0.);

        assert_relative_eq!(line0.euclidean_distance(&p2), 1.);
        assert_relative_eq!(p2.euclidean_distance(&line0), 1.);
    }
    #[test]
    fn distance_line_line_test() {
        let line0 = Line::from([(0., 0.), (5., 0.)]);
        let line1 = Line::from([(2., 1.), (7., 2.)]);
        assert_relative_eq!(line0.euclidean_distance(&line1), 1.);
        assert_relative_eq!(line1.euclidean_distance(&line0), 1.);
    }
    #[test]
    // See https://github.com/georust/geo/issues/476
    fn distance_line_polygon_test() {
        let line = Line::new(
            Coordinate {
                x: -0.17084137691985102,
                y: 0.8748085493016657,
            },
            Coordinate {
                x: -0.17084137691985102,
                y: 0.09858870312437906,
            },
        );
        let poly: Polygon<f64> = polygon![
            Coordinate {
                x: -0.10781391405721802,
                y: -0.15433610862574643,
            },
            Coordinate {
                x: -0.7855276236615211,
                y: 0.23694208404779793,
            },
            Coordinate {
                x: -0.7855276236615214,
                y: -0.5456143012992907,
            },
            Coordinate {
                x: -0.10781391405721802,
                y: -0.15433610862574643,
            },
        ];
        assert_eq!(line.euclidean_distance(&poly), 0.18752558079168907);
    }
    #[test]
    // test edge-vertex minimum distance
    fn test_minimum_polygon_distance() {
        let points_raw = vec![
            (126., 232.),
            (126., 212.),
            (112., 202.),
            (97., 204.),
            (87., 215.),
            (87., 232.),
            (100., 246.),
            (118., 247.),
        ];
        let points = points_raw
            .iter()
            .map(|e| Point::new(e.0, e.1))
            .collect::<Vec<_>>();
        let poly1 = Polygon::new(LineString::from(points), vec![]);

        let points_raw_2 = vec![
            (188., 231.),
            (189., 207.),
            (174., 196.),
            (164., 196.),
            (147., 220.),
            (158., 242.),
            (177., 242.),
        ];
        let points2 = points_raw_2
            .iter()
            .map(|e| Point::new(e.0, e.1))
            .collect::<Vec<_>>();
        let poly2 = Polygon::new(LineString::from(points2), vec![]);
        let dist = min_poly_dist(&poly1.convex_hull(), &poly2.convex_hull());
        let dist2 = nearest_neighbour_distance(&poly1.exterior(), &poly2.exterior());
        assert_relative_eq!(dist, 21.0);
        assert_relative_eq!(dist2, 21.0);
    }
    #[test]
    // test vertex-vertex minimum distance
    fn test_minimum_polygon_distance_2() {
        let points_raw = vec![
            (118., 200.),
            (153., 179.),
            (106., 155.),
            (88., 190.),
            (118., 200.),
        ];
        let points = points_raw
            .iter()
            .map(|e| Point::new(e.0, e.1))
            .collect::<Vec<_>>();
        let poly1 = Polygon::new(LineString::from(points), vec![]);

        let points_raw_2 = vec![
            (242., 186.),
            (260., 146.),
            (182., 175.),
            (216., 193.),
            (242., 186.),
        ];
        let points2 = points_raw_2
            .iter()
            .map(|e| Point::new(e.0, e.1))
            .collect::<Vec<_>>();
        let poly2 = Polygon::new(LineString::from(points2), vec![]);
        let dist = min_poly_dist(&poly1.convex_hull(), &poly2.convex_hull());
        let dist2 = nearest_neighbour_distance(&poly1.exterior(), &poly2.exterior());
        assert_relative_eq!(dist, 29.274562336608895);
        assert_relative_eq!(dist2, 29.274562336608895);
    }
    #[test]
    // test edge-edge minimum distance
    fn test_minimum_polygon_distance_3() {
        let points_raw = vec![
            (182., 182.),
            (182., 168.),
            (138., 160.),
            (136., 193.),
            (182., 182.),
        ];
        let points = points_raw
            .iter()
            .map(|e| Point::new(e.0, e.1))
            .collect::<Vec<_>>();
        let poly1 = Polygon::new(LineString::from(points), vec![]);

        let points_raw_2 = vec![
            (232., 196.),
            (234., 150.),
            (194., 165.),
            (194., 191.),
            (232., 196.),
        ];
        let points2 = points_raw_2
            .iter()
            .map(|e| Point::new(e.0, e.1))
            .collect::<Vec<_>>();
        let poly2 = Polygon::new(LineString::from(points2), vec![]);
        let dist = min_poly_dist(&poly1.convex_hull(), &poly2.convex_hull());
        let dist2 = nearest_neighbour_distance(&poly1.exterior(), &poly2.exterior());
        assert_relative_eq!(dist, 12.0);
        assert_relative_eq!(dist2, 12.0);
    }
    #[test]
    fn test_large_polygon_distance() {
        let points = include!("test_fixtures/norway_main.rs");
        let points_ls: Vec<_> = points.iter().map(|e| Point::new(e[0], e[1])).collect();
        let ls = LineString::from(points_ls);
        let poly1 = Polygon::new(ls, vec![]);
        let vec2 = vec![
            (4.921875, 66.33750501996518),
            (3.69140625, 65.21989393613207),
            (6.15234375, 65.07213008560697),
            (4.921875, 66.33750501996518),
        ];
        let poly2 = Polygon::new(vec2.into(), vec![]);
        let distance = poly1.euclidean_distance(&poly2);
        // GEOS says 2.2864896295566055
        assert_relative_eq!(distance, 2.2864896295566055);
    }
    #[test]
    // A polygon inside another polygon's ring; they're disjoint in the DE-9IM sense:
    // FF2FF1212
    fn test_poly_in_ring() {
        let shell = include!("test_fixtures/shell.rs");
        let shell_ls: LineString<f64> = shell.into();
        let ring = include!("test_fixtures/ring.rs");
        let ring_ls: LineString<f64> = ring.into();
        let poly_in_ring = include!("test_fixtures/poly_in_ring.rs");
        let poly_in_ring_ls: LineString<f64> = poly_in_ring.into();
        // inside is "inside" outside's ring, but they are disjoint
        let outside = Polygon::new(shell_ls, vec![ring_ls]);
        let inside = Polygon::new(poly_in_ring_ls, vec![]);
        assert_relative_eq!(outside.euclidean_distance(&inside), 5.992772737231033);
    }
    #[test]
    // two ring LineStrings; one encloses the other but they neither touch nor intersect
    fn test_linestring_distance() {
        let ring = include!("test_fixtures/ring.rs");
        let ring_ls: LineString<f64> = ring.into();
        let in_ring = include!("test_fixtures/poly_in_ring.rs");
        let in_ring_ls: LineString<f64> = in_ring.into();
        assert_relative_eq!(ring_ls.euclidean_distance(&in_ring_ls), 5.992772737231033);
    }
    #[test]
    // Line-Polygon test: closest point on Polygon is NOT nearest to a Line end-point
    fn test_line_polygon_simple() {
        let line = Line::from([(0.0, 0.0), (0.0, 3.0)]);
        let v = vec![(5.0, 1.0), (5.0, 2.0), (0.25, 1.5), (5.0, 1.0)];
        let poly = Polygon::new(v.into(), vec![]);
        assert_relative_eq!(line.euclidean_distance(&poly), 0.25);
    }
    #[test]
    // Line-Polygon test: Line intersects Polygon
    fn test_line_polygon_intersects() {
        let line = Line::from([(0.5, 0.0), (0.0, 3.0)]);
        let v = vec![(5.0, 1.0), (5.0, 2.0), (0.25, 1.5), (5.0, 1.0)];
        let poly = Polygon::new(v.into(), vec![]);
        assert_relative_eq!(line.euclidean_distance(&poly), 0.0);
    }
    #[test]
    // Line-Polygon test: Line contained by interior ring
    fn test_line_polygon_inside_ring() {
        let line = Line::from([(4.4, 1.5), (4.45, 1.5)]);
        let v = vec![(5.0, 1.0), (5.0, 2.0), (0.25, 1.0), (5.0, 1.0)];
        let v2 = vec![(4.5, 1.2), (4.5, 1.8), (3.5, 1.2), (4.5, 1.2)];
        let poly = Polygon::new(v.into(), vec![v2.into()]);
        assert_relative_eq!(line.euclidean_distance(&poly), 0.04999999999999982);
    }
    #[test]
    // LineString-Line test
    fn test_linestring_line_distance() {
        let line = Line::from([(0.0, 0.0), (0.0, 2.0)]);
        let ls: LineString<_> = vec![(3.0, 0.0), (1.0, 1.0), (3.0, 2.0)].into();
        assert_relative_eq!(ls.euclidean_distance(&line), 1.0);
    }

    #[test]
    // Triangle-Point test: point on vertex
    fn test_triangle_point_on_vertex_distance() {
        let triangle = Triangle::from([(0.0, 0.0), (2.0, 0.0), (2.0, 2.0)]);
        let point = Point::new(0.0, 0.0);
        assert_relative_eq!(triangle.euclidean_distance(&point), 0.0);
    }

    #[test]
    // Triangle-Point test: point on edge
    fn test_triangle_point_on_edge_distance() {
        let triangle = Triangle::from([(0.0, 0.0), (2.0, 0.0), (2.0, 2.0)]);
        let point = Point::new(1.5, 0.0);
        assert_relative_eq!(triangle.euclidean_distance(&point), 0.0);
    }

    #[test]
    // Triangle-Point test
    fn test_triangle_point_distance() {
        let triangle = Triangle::from([(0.0, 0.0), (2.0, 0.0), (2.0, 2.0)]);
        let point = Point::new(2.0, 3.0);
        assert_relative_eq!(triangle.euclidean_distance(&point), 1.0);
    }

    #[test]
    // Triangle-Point test: point within triangle
    fn test_triangle_point_inside_distance() {
        let triangle = Triangle::from([(0.0, 0.0), (2.0, 0.0), (2.0, 2.0)]);
        let point = Point::new(1.0, 0.5);
        assert_relative_eq!(triangle.euclidean_distance(&point), 0.0);
    }
}
