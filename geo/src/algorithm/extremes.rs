use crate::algorithm::convexhull::ConvexHull;
use crate::{ExtremePoint, Extremes};
use crate::{MultiPoint, MultiPolygon, Point, Polygon};
use num_traits::{Float, Signed};

// Useful direction vectors, aligned with x and y axes:
// 1., 0. = largest x
// 0., 1. = largest y
// 0., -1. = smallest y
// -1, 0. = smallest x

// various tests for vector orientation relative to a direction vector u

// Not currently used, but maybe useful in the future
#[allow(dead_code)]
fn up<T>(u: Point<T>, v: Point<T>) -> bool
where
    T: Float,
{
    u.dot(v) > T::zero()
}

fn direction_sign<T>(u: Point<T>, vi: Point<T>, vj: Point<T>) -> T
where
    T: Float,
{
    u.dot(vi - vj)
}

// true if Vi is above Vj
fn above<T>(u: Point<T>, vi: Point<T>, vj: Point<T>) -> bool
where
    T: Float,
{
    direction_sign(u, vi, vj) > T::zero()
}

// true if Vi is below Vj
// Not currently used, but maybe useful in the future
#[allow(dead_code)]
fn below<T>(u: Point<T>, vi: Point<T>, vj: Point<T>) -> bool
where
    T: Float,
{
    direction_sign(u, vi, vj) < T::zero()
}

// wrapper for extreme-finding function
fn find_extreme_indices<T, F>(func: F, polygon: &Polygon<T>) -> Result<Extremes, ()>
where
    T: Float + Signed,
    F: Fn(Point<T>, &Polygon<T>) -> Result<usize, ()>,
{
    if !polygon.is_convex() {
        return Err(());
    }
    let directions = vec![
        Point::new(T::zero(), -T::one()),
        Point::new(T::one(), T::zero()),
        Point::new(T::zero(), T::one()),
        Point::new(-T::one(), T::zero()),
    ];
    Ok(directions
        .iter()
        .map(|p| func(*p, polygon).unwrap())
        .collect::<Vec<usize>>()
        .into())
}

// find a convex, counter-clockwise oriented polygon's maximum vertex in a specified direction
// u: a direction vector. We're using a point to represent this, which is a hack but works fine
fn polymax_naive_indices<T>(u: Point<T>, poly: &Polygon<T>) -> Result<usize, ()>
where
    T: Float,
{
    let vertices = &poly.exterior().0;
    let mut max: usize = 0;
    for (i, _) in vertices.iter().enumerate() {
        // if vertices[i] is above prior vertices[max]
        if above(u, Point(vertices[i]), Point(vertices[max])) {
            max = i;
        }
    }
    Ok(max)
}

pub trait ExtremeIndices<T: Float + Signed> {
    /// Find the extreme `x` and `y` _indices_ of a convex Polygon
    ///
    /// The polygon **must be convex and properly (ccw) oriented**.
    ///
    /// # Examples
    ///
    /// ```
    /// use geo::{Point, LineString, Polygon};
    /// use geo::extremes::ExtremeIndices;
    /// // a diamond shape
    /// let points_raw = vec![(1.0, 0.0), (2.0, 1.0), (1.0, 2.0), (0.0, 1.0), (1.0, 0.0)];
    /// let points = points_raw.iter().map(|e| Point::new(e.0, e.1)).collect::<Vec<_>>();
    /// let poly = Polygon::new(LineString::from(points), vec![]);
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
where
    T: Float + Signed,
{
    fn extreme_indices(&self) -> Result<Extremes, ()> {
        find_extreme_indices(polymax_naive_indices, self)
    }
}

impl<T> ExtremeIndices<T> for MultiPolygon<T>
where
    T: Float + Signed,
{
    fn extreme_indices(&self) -> Result<Extremes, ()> {
        find_extreme_indices(polymax_naive_indices, &self.convex_hull())
    }
}

impl<T> ExtremeIndices<T> for MultiPoint<T>
where
    T: Float + Signed,
{
    fn extreme_indices(&self) -> Result<Extremes, ()> {
        find_extreme_indices(polymax_naive_indices, &self.convex_hull())
    }
}

pub trait ExtremePoints<T: Float> {
    /// Find the extreme `x` and `y` `Point`s of a Geometry
    ///
    /// This trait is available to any struct implementing both `ConvexHull` amd `ExtremeIndices`
    ///
    /// # Examples
    ///
    /// ```
    /// use geo::{Point, LineString, Polygon};
    /// use geo::extremes::ExtremePoints;
    /// let points_raw = vec![(1.0, 0.0), (2.0, 1.0), (1.0, 2.0), (0.0, 1.0), (1.0, 0.0)];
    /// let points = points_raw
    ///     .iter()
    ///     .map(|e| Point::new(e.0, e.1))
    ///     .collect::<Vec<_>>();
    /// let poly1 = Polygon::new(LineString::from(points), vec![]);
    /// let extremes = poly1.extreme_points();
    /// let correct = Point::new(0.0, 1.0);
    /// assert_eq!(extremes.xmin, correct);
    /// ```
    fn extreme_points(&self) -> ExtremePoint<T>;
}

impl<T, G> ExtremePoints<T> for G
where
    T: Float + Signed,
    G: ConvexHull<T> + ExtremeIndices<T>,
{
    // Any Geometry implementing `ConvexHull` and `ExtremeIndices` gets this automatically
    fn extreme_points(&self) -> ExtremePoint<T> {
        let ch = self.convex_hull();
        // safe to unwrap, since we're guaranteeing the polygon's convexity
        let indices = ch.extreme_indices().unwrap();
        ExtremePoint {
            ymin: Point(ch.exterior().0[indices.ymin]),
            xmax: Point(ch.exterior().0[indices.xmax]),
            ymax: Point(ch.exterior().0[indices.ymax]),
            xmin: Point(ch.exterior().0[indices.xmin]),
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::{point, polygon};

    #[test]
    fn test_polygon_extreme_x() {
        // a diamond shape
        let poly1 = polygon![
            (x: 1.0, y: 0.0),
            (x: 2.0, y: 1.0),
            (x: 1.0, y: 2.0),
            (x: 0.0, y: 1.0),
            (x: 1.0, y: 0.0)
        ];
        let min_x = polymax_naive_indices(point!(x: -1., y: 0.), &poly1).unwrap();
        let correct = 3_usize;
        assert_eq!(min_x, correct);
    }
    #[test]
    #[should_panic]
    fn test_extreme_indices_bad_polygon() {
        // non-convex, with a bump on the top-right edge
        let poly1 = polygon![
            (x: 1.0, y: 0.0),
            (x: 1.3, y: 1.),
            (x: 2.0, y: 1.0),
            (x: 1.75, y: 1.75),
            (x: 1.0, y: 2.0),
            (x: 0.0, y: 1.0),
            (x: 1.0, y: 0.0)
        ];
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
        let poly1 = polygon![
            (x: 1.0, y: 0.0),
            (x: 1.3, y: 1.),
            (x: 2.0, y: 1.0),
            (x: 1.75, y: 1.75),
            (x: 1.0, y: 2.0),
            (x: 0.0, y: 1.0),
            (x: 1.0, y: 0.0)
        ];
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
        let poly1 = polygon![
            (x: 1.0, y: 0.0),
            (x: 2.0, y: 1.0),
            (x: 1.75, y: 1.75),
            (x: 1.0, y: 2.0),
            (x: 0.0, y: 1.0),
            (x: 1.0, y: 0.0)
        ];
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
        let poly1 = polygon![
            (x: 1.0, y: 0.0),
            (x: 2.0, y: 1.0),
            (x: 1.0, y: 2.0),
            (x: 0.0, y: 1.0),
            (x: 1.0, y: 0.0)
        ];
        let extremes = poly1.extreme_points();
        let correct = point!(x: 0.0, y: 1.0);
        assert_eq!(extremes.xmin, correct);
    }
}
