use crate::{AffineOps, AffineTransform, CoordFloat, CoordNum, Point};

/// An affine transformation which skews a geometry, sheared by angles along x and y dimensions.
///
/// The point of origin is *usually* given as the 2D bounding box centre of the geometry, but
/// any coordinate may be specified. Angles are given in **degrees**.
///
/// ## Performance
///
/// If you will be performing multiple transformations, like [`Scale`](crate::Scale),
/// [`Skew`](crate::Skew), [`Translate`](crate::Translate), or [`Rotate`](crate::Rotate), it is more
/// efficient to compose the transformations and apply them as a single operation using the
/// [`AffineOps`](crate::AffineOps) trait.
///
/// # Examples
///
/// ```
/// use geo::Skew;
/// use geo::{line_string, BoundingRect, point, LineString, Centroid};
/// use approx::assert_relative_eq;
///
/// let ls: LineString<f64> = line_string![
///     (x: 0.0f64, y: 0.0f64),
///     (x: 0.0f64, y: 10.0f64),
/// ];
/// // let origin = ls.bounding_rect().unwrap().centroid();
/// let origin = point!{ x: 0.0f64, y: 5.0f64 };
/// let skewed = ls.skew(45.0, 45.0, origin);
///
/// assert_relative_eq!(skewed, line_string![
///     (x: -4.99f64, y: 0.0f64),
///     (x: 4.99f64, y: 10.0f64)
/// ], max_relative = 1.0);
/// ```
pub trait Skew<T: CoordNum> {
    #[must_use]
    fn skew(&self, xs: T, ys: T, origin: Point<T>) -> Self;
    fn skew_mut(&mut self, xs: T, ys: T, origin: Point<T>);
}

impl<T, G> Skew<T> for G
where
    T: CoordFloat,
    G: AffineOps<T>,
{
    fn skew(&self, xs: T, ys: T, origin: Point<T>) -> Self {
        let transform = AffineTransform::skew(xs, ys, origin);
        self.affine_transform(&transform)
    }

    fn skew_mut(&mut self, xs: T, ys: T, origin: Point<T>) {
        let transform = AffineTransform::skew(xs, ys, origin);
        self.affine_transform_mut(&transform)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{line_string, BoundingRect, Centroid, LineString};

    #[test]
    fn skew_linestring() {
        let ls: LineString<f64> = line_string![
            (x: 3.0, y: 0.0),
            (x: 3.0, y: 10.0),
        ];
        let origin = ls.bounding_rect().unwrap().centroid();
        let sheared = ls.skew(45.0, 45.0, origin);
        assert_eq!(
            sheared,
            line_string![
                (x: -1.9999999999999991, y: 0.0),
                (x: 7.999999999999999, y: 10.0)
            ]
        );
    }
}
