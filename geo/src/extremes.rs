use super::kernels::*;
use crate::prelude::*;
use crate::*;
use num_traits::Signed;

// Useful direction vectors, aligned with x and y axes:
// 1., 0. = largest x
// 0., 1. = largest y
// 0., -1. = smallest y
// -1, 0. = smallest x

/// Predicate that returns `true` if `vi` is (strictly) above `vj`
/// along the direction of `u`
fn above<T>(u: Coordinate<T>, vi: Coordinate<T>, vj: Coordinate<T>) -> bool
where
    T: CoordinateType + Signed + HasKernel,
{
    T::Ker::dot_product_sign(u, vi - vj) == Orientation::CounterClockwise
}

/// Predicate that returns `true` if `vi` is (strictly) below `vj`
/// along the direction of `u`
#[allow(dead_code)]
fn below<T>(u: Coordinate<T>, vi: Coordinate<T>, vj: Coordinate<T>) -> bool
where
    T: CoordinateType + Signed + HasKernel,
{
    T::Ker::dot_product_sign(u, vi - vj) == Orientation::Clockwise
}

// wrapper for extreme-finding function
fn find_extreme_indices<T, F>(func: F, polygon: &Polygon<T>) -> Result<Extremes, ()>
where
    T: HasKernel + Signed,
    F: Fn(Coordinate<T>, &Polygon<T>) -> Result<usize, ()>,
{
    if !polygon.exterior().is_convex() {
        return Err(());
    }
    let directions: Vec<Coordinate<_>> = vec![
        (T::zero(), -T::one()).into(),
        (T::one(), T::zero()).into(),
        (T::zero(), T::one()).into(),
        (-T::one(), T::zero()).into(),
    ];
    Ok(directions
        .into_iter()
        .map(|p| func(p, polygon).unwrap())
        .collect::<Vec<usize>>()
        .into())
}

// find a convex, counter-clockwise oriented polygon's maximum vertex in a specified direction
// u: a direction vector. We're using a point to represent this, which is a hack but works fine
fn polymax_naive_indices<T>(u: Coordinate<T>, poly: &Polygon<T>) -> Result<usize, ()>
where
    T: HasKernel + Signed,
{
    let vertices = &poly.exterior().0;
    let mut max: usize = 0;
    for (i, _) in vertices.iter().enumerate() {
        // if vertices[i] is above prior vertices[max]
        if above(u, vertices[i], vertices[max]) {
            max = i;
        }
    }
    Ok(max)
}

pub trait ExtremeIndices {
    /// Find the extreme `x` and `y` _indices_ of a convex Polygon
    ///
    /// The polygon **must be convex and properly (ccw) oriented**.
    ///
    /// # Examples
    ///
    /// ```
    /// use geo::extremes::ExtremeIndices;
    /// use geo::polygon;
    ///
    /// // a diamond shape
    /// let polygon = polygon![
    ///     (x: 1.0, y: 0.0),
    ///     (x: 2.0, y: 1.0),
    ///     (x: 1.0, y: 2.0),
    ///     (x: 0.0, y: 1.0),
    ///     (x: 1.0, y: 0.0),
    /// ];
    ///
    /// // Polygon is both convex and oriented counter-clockwise
    /// let extremes = polygon.extreme_indices().unwrap();
    ///
    /// assert_eq!(extremes.ymin, 0);
    /// assert_eq!(extremes.xmax, 1);
    /// assert_eq!(extremes.ymax, 2);
    /// assert_eq!(extremes.xmin, 3);
    /// ```
    fn extreme_indices(&self) -> Result<Extremes, ()>;
}

impl<T> ExtremeIndices for Polygon<T>
where
    T: Signed + HasKernel,
{
    fn extreme_indices(&self) -> Result<Extremes, ()> {
        find_extreme_indices(polymax_naive_indices, self)
    }
}

impl<T> ExtremeIndices for MultiPolygon<T>
where
    T: Signed + HasKernel,
{
    fn extreme_indices(&self) -> Result<Extremes, ()> {
        find_extreme_indices(polymax_naive_indices, &self.convex_hull())
    }
}

impl<T> ExtremeIndices for MultiPoint<T>
where
    T: Signed + HasKernel,
{
    fn extreme_indices(&self) -> Result<Extremes, ()> {
        find_extreme_indices(polymax_naive_indices, &self.convex_hull())
    }
}

pub trait ExtremePoints {
    type Scalar: CoordinateType;
    /// Find the extreme `x` and `y` `Point`s of a Geometry
    ///
    /// This trait is available to any struct implementing both `ConvexHull` amd `ExtremeIndices`
    ///
    /// # Examples
    ///
    /// ```
    /// use geo::extremes::ExtremePoints;
    /// use geo::{point, polygon};
    ///
    /// // a diamond shape
    /// let polygon = polygon![
    ///     (x: 1.0, y: 0.0),
    ///     (x: 2.0, y: 1.0),
    ///     (x: 1.0, y: 2.0),
    ///     (x: 0.0, y: 1.0),
    ///     (x: 1.0, y: 0.0),
    /// ];
    ///
    /// let extremes = polygon.extreme_points();
    ///
    /// assert_eq!(extremes.xmin, point!(x: 0., y: 1.));
    /// ```
    fn extreme_points(&self) -> ExtremePoint<Self::Scalar>;
}

impl<T, G> ExtremePoints for G
where
    T: Signed + HasKernel,
    G: ConvexHull<Scalar = T> + ExtremeIndices,
{
    type Scalar = T;

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
        let min_x = polymax_naive_indices(Coordinate { x: -1., y: 0. }, &poly1).unwrap();
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
