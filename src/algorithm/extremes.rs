use num_traits::{Float, Signed};
use types::{Point, Polygon, MultiPoint, MultiPolygon};
use algorithm::convexhull::ConvexHull;
use types::{Extremes, ExtremePoint};

// Useful direction vectors, aligned with x and y axes:
// 1., 0. = largest x
// 0., 1. = largest y
// 0., -1. = smallest y
// -1, 0. = smallest x

// various tests for vector orientation relative to a direction vector u

// Not currently used, but maybe useful in the future
#[allow(dead_code)]
fn up<T>(u: &Point<T>, v: &Point<T>) -> bool
    where T: Float
{
    u.dot(v) > T::zero()
}

fn direction_sign<T>(u: &Point<T>, vi: &Point<T>, vj: &Point<T>) -> T
    where T: Float
{
    u.dot(&(*vi - *vj))
}

// true if Vi is above Vj
fn above<T>(u: &Point<T>, vi: &Point<T>, vj: &Point<T>) -> bool
    where T: Float
{
    direction_sign(u, vi, vj) > T::zero()
}

// true if Vi is below Vj
// Not currently used, but maybe useful in the future
#[allow(dead_code)]
fn below<T>(u: &Point<T>, vi: &Point<T>, vj: &Point<T>) -> bool
    where T: Float
{
    direction_sign(u, vi, vj) < T::zero()
}

// used to check the sign of a vec of floats
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
enum ListSign {
    Empty,
    Positive,
    Negative,
    Mixed,
}

// Wrap-around previous-vertex
impl<T> Polygon<T>
    where T: Float
{
    fn previous_vertex(&self, current_vertex: &usize) -> usize
        where T: Float
    {
        (current_vertex + (self.exterior.0.len() - 1) - 1) % (self.exterior.0.len() - 1)
    }
}

// positive implies a -> b -> c is counter-clockwise, negative implies clockwise
fn cross_prod<T>(p_a: &Point<T>, p_b: &Point<T>, p_c: &Point<T>) -> T
    where T: Float
{
    (p_b.x() - p_a.x()) * (p_c.y() - p_a.y()) - (p_b.y() - p_a.y()) * (p_c.x() - p_a.x())
}

// wrapper for extreme-finding function
fn find_extreme_indices<T, F>(func: F, polygon: &Polygon<T>) -> Result<Extremes, ()>
    where T: Float + Signed,
          F: Fn(&Point<T>, &Polygon<T>) -> Result<usize, ()>
{
    // For each consecutive pair of edges of the polygon (each triplet of points),
    // compute the z-component of the cross product of the vectors defined by the
    // edges pointing towards the points in increasing order.
    // Take the cross product of these vectors
    // The polygon is convex if the z-components of the cross products are either
    // all positive or all negative. Otherwise, the polygon is non-convex.
    // see: http://stackoverflow.com/a/1881201/416626
    let convex = polygon
        .exterior
        .0
        .iter()
        .enumerate()
        .map(|(idx, _)| {
            let prev_1 = polygon.previous_vertex(&idx);
            let prev_2 = polygon.previous_vertex(&prev_1);
            cross_prod(&polygon.exterior.0[prev_2],
                       &polygon.exterior.0[prev_1],
                       &polygon.exterior.0[idx])
        })
        // accumulate and check cross-product result signs in a single pass
        // positive implies ccw convexity, negative implies cw convexity
        // anything else implies non-convexity
        .fold(ListSign::Empty, |acc, n| {
            match (acc, n.is_positive()) {
                (ListSign::Empty, true) | (ListSign::Positive, true) => ListSign::Positive,
                (ListSign::Empty, false) | (ListSign::Negative, false) => ListSign::Negative,
                _ => ListSign::Mixed
            }
        });
    if convex == ListSign::Mixed {
        return Err(());
    }
    let directions = vec![Point::new(T::zero(), -T::one()),
                          Point::new(T::one(), T::zero()),
                          Point::new(T::zero(), T::one()),
                          Point::new(-T::one(), T::zero())];
    Ok(directions
           .iter()
           .map(|p| func(&p, &polygon).unwrap())
           .collect::<Vec<usize>>()
           .into())
}

// find a convex, counter-clockwise oriented polygon's maximum vertex in a specified direction
// u: a direction vector. We're using a point to represent this, which is a hack but works fine
fn polymax_naive_indices<T>(u: &Point<T>, poly: &Polygon<T>) -> Result<usize, ()>
    where T: Float
{
    let vertices = &poly.exterior.0;
    let mut max: usize = 0;
    for (i, _) in vertices.iter().enumerate() {
        // if vertices[i] is above prior vertices[max]
        if above(u, &vertices[i], &vertices[max]) {
            max = i;
        }
    }
    return Ok(max);
}

pub trait ExtremeIndices<T: Float + Signed> {
    /// Find the extreme `x` and `y` indices of a convex Polygon
    ///
    /// The polygon **must be convex and properly (ccw) oriented**.
    ///
    /// ```
    /// use geo::{Point, LineString, Polygon};
    /// use geo::extremes::ExtremeIndices;
    /// // a diamond shape
    /// let points_raw = vec![(1.0, 0.0), (2.0, 1.0), (1.0, 2.0), (0.0, 1.0), (1.0, 0.0)];
    /// let points = points_raw.iter().map(|e| Point::new(e.0, e.1)).collect::<Vec<_>>();
    /// let poly = Polygon::new(LineString(points), vec![]);
    /// // Polygon is both convex and oriented counter-clockwise
    /// let extremes = poly.extreme_indices().unwrap();
    /// assert_eq!(extremes.ymin, 0);
    /// assert_eq!(extremes.xmax, 1);
    /// assert_eq!(extremes.ymax, 2);
    /// assert_eq!(extremes.xmin, 3);
    /// ```
    fn extreme_indices(&self) -> Result<Extremes, ()>;
}

impl<T> ExtremeIndices<T> for Polygon<T>
    where T: Float + Signed
{
    fn extreme_indices(&self) -> Result<Extremes, ()> {
        find_extreme_indices(polymax_naive_indices, self)
    }
}

impl<T> ExtremeIndices<T> for MultiPolygon<T>
    where T: Float + Signed
{
    fn extreme_indices(&self) -> Result<Extremes, ()> {
        find_extreme_indices(polymax_naive_indices, &self.convex_hull())
    }
}

impl<T> ExtremeIndices<T> for MultiPoint<T>
    where T: Float + Signed
{
    fn extreme_indices(&self) -> Result<Extremes, ()> {
        find_extreme_indices(polymax_naive_indices, &self.convex_hull())
    }
}

pub trait ExtremePoints<T: Float> {
    /// Find the extreme `x` and `y` points of a Geometry
    ///
    /// This trait is available to any struct implementing both `ConvexHull` amd `ExtremeIndices`
    ///
    /// ```
    /// use geo::{Point, LineString, Polygon};
    /// use geo::extremes::ExtremePoints;
    /// let points_raw = vec![(1.0, 0.0), (2.0, 1.0), (1.0, 2.0), (0.0, 1.0), (1.0, 0.0)];
    /// let points = points_raw
    ///     .iter()
    ///     .map(|e| Point::new(e.0, e.1))
    ///     .collect::<Vec<_>>();
    /// let poly1 = Polygon::new(LineString(points), vec![]);
    /// let extremes = poly1.extreme_points();
    /// let correct = Point::new(0.0, 1.0);
    /// assert_eq!(extremes.xmin, correct);
    /// ```
    fn extreme_points(&self) -> ExtremePoint<T>;
}

impl<T, G> ExtremePoints<T> for G
    where T: Float + Signed,
          G: ConvexHull<T> + ExtremeIndices<T>
{
    // Any Geometry implementing `ConvexHull` and `ExtremeIndices` gets this automatically
    fn extreme_points(&self) -> ExtremePoint<T> {
        let ch = self.convex_hull();
        // safe to unwrap, since we're guaranteeing the polygon's convexity
        let indices = ch.extreme_indices().unwrap();
        ExtremePoint {
            ymin: ch.exterior.0[indices.ymin],
            xmax: ch.exterior.0[indices.xmax],
            ymax: ch.exterior.0[indices.ymax],
            xmin: ch.exterior.0[indices.xmin],
        }
    }
}

#[cfg(test)]
mod test {

    use types::{Point, LineString};
    use super::*;
    #[test]
    fn test_polygon_extreme_x() {
        // a diamond shape
        let points_raw = vec![(1.0, 0.0), (2.0, 1.0), (1.0, 2.0), (0.0, 1.0), (1.0, 0.0)];
        let points = points_raw
            .iter()
            .map(|e| Point::new(e.0, e.1))
            .collect::<Vec<_>>();
        let poly1 = Polygon::new(LineString(points), vec![]);
        let min_x = polymax_naive_indices(&Point::new(-1., 0.), &poly1).unwrap();
        let correct = 3_usize;
        assert_eq!(min_x, correct);
    }
    #[test]
    #[should_panic]
    fn test_extreme_indices_bad_polygon() {
        // non-convex, with a bump on the top-right edge
        let points_raw = vec![(1.0, 0.0),
                              (1.3, 1.),
                              (2.0, 1.0),
                              (1.75, 1.75),
                              (1.0, 2.0),
                              (0.0, 1.0),
                              (1.0, 0.0)];
        let points = points_raw
            .iter()
            .map(|e| Point::new(e.0, e.1))
            .collect::<Vec<_>>();
        let poly1 = Polygon::new(LineString(points), vec![]);
        let extremes = find_extreme_indices(polymax_naive_indices, &poly1).unwrap();
        let correct = Extremes {
            ymin: 0,
            xmax: 1,
            ymax: 3,
            xmin: 4,
        };
        assert_eq!(extremes, correct);
    }
    #[test]
    fn test_extreme_indices_good_polygon() {
        // non-convex, with a bump on the top-right edge
        let points_raw = vec![(1.0, 0.0),
                              (1.3, 1.),
                              (2.0, 1.0),
                              (1.75, 1.75),
                              (1.0, 2.0),
                              (0.0, 1.0),
                              (1.0, 0.0)];
        let points = points_raw
            .iter()
            .map(|e| Point::new(e.0, e.1))
            .collect::<Vec<_>>();
        let poly1 = Polygon::new(LineString(points), vec![]);
        let extremes = find_extreme_indices(polymax_naive_indices, &poly1.convex_hull()).unwrap();
        let correct = Extremes {
            ymin: 0,
            xmax: 1,
            ymax: 3,
            xmin: 4,
        };
        assert_eq!(extremes, correct);
    }
    #[test]
    fn test_polygon_extreme_wrapper_convex() {
        // convex, with a bump on the top-right edge
        let points_raw =
            vec![(1.0, 0.0), (2.0, 1.0), (1.75, 1.75), (1.0, 2.0), (0.0, 1.0), (1.0, 0.0)];
        let points = points_raw
            .iter()
            .map(|e| Point::new(e.0, e.1))
            .collect::<Vec<_>>();
        let poly1 = Polygon::new(LineString(points), vec![]);
        let extremes = find_extreme_indices(polymax_naive_indices, &poly1.convex_hull()).unwrap();
        let correct = Extremes {
            ymin: 0,
            xmax: 1,
            ymax: 3,
            xmin: 4,
        };
        assert_eq!(extremes, correct);
    }
    #[test]
    fn test_polygon_extreme_point_x() {
        // a diamond shape
        let points_raw = vec![(1.0, 0.0), (2.0, 1.0), (1.0, 2.0), (0.0, 1.0), (1.0, 0.0)];
        let points = points_raw
            .iter()
            .map(|e| Point::new(e.0, e.1))
            .collect::<Vec<_>>();
        let poly1 = Polygon::new(LineString(points), vec![]);
        let extremes = poly1.extreme_points();
        let correct = Point::new(0.0, 1.0);
        assert_eq!(extremes.xmin, correct);
    }
}
