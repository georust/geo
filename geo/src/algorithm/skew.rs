use crate::{AffineOps, AffineTransform, BoundingRect, Coord, CoordFloat, CoordNum, Rect};

/// An affine transformation which skews a geometry, sheared by angles along x and y dimensions.
///
/// ## Performance
///
/// If you will be performing multiple transformations, like [`Scale`](crate::Scale),
/// [`Skew`], [`Translate`](crate::Translate), or [`Rotate`](crate::Rotate), it is more
/// efficient to compose the transformations and apply them as a single operation using the
/// [`AffineOps`] trait.
///
pub trait Skew<T: CoordNum> {
    /// An affine transformation which skews a geometry, sheared by a uniform angle along the x and
    /// y dimensions.
    ///
    /// # Examples
    ///
    /// ```
    /// use geo::Skew;
    /// use geo::{Polygon, polygon};
    ///
    /// let square: Polygon = polygon![
    ///     (x: 0., y: 0.),
    ///     (x: 10., y: 0.),
    ///     (x: 10., y: 10.),
    ///     (x: 0., y: 10.)
    /// ];
    ///
    /// let skewed = square.skew(30.);
    ///
    /// let expected_output: Polygon = polygon![
    ///     (x: -2.89, y: -2.89),
    ///     (x: 7.11, y: 2.89),
    ///     (x: 12.89, y: 12.89),
    ///     (x: 2.89, y: 7.11)
    /// ];
    /// approx::assert_relative_eq!(skewed, expected_output, epsilon = 1e-2);
    /// ```
    #[must_use]
    fn skew(&self, degrees: T) -> Self;

    /// Mutable version of [`skew`](Self::skew).
    fn skew_mut(&mut self, degrees: T);

    /// An affine transformation which skews a geometry, sheared by an angle along the x and y dimensions.
    ///
    /// # Examples
    ///
    /// ```
    /// use geo::Skew;
    /// use geo::{Polygon, polygon};
    ///
    /// let square: Polygon = polygon![
    ///     (x: 0., y: 0.),
    ///     (x: 10., y: 0.),
    ///     (x: 10., y: 10.),
    ///     (x: 0., y: 10.)
    /// ];
    ///
    /// let skewed = square.skew_xy(30., 12.);
    ///
    /// let expected_output: Polygon = polygon![
    ///     (x: -2.89, y: -1.06),
    ///     (x: 7.11, y: 1.06),
    ///     (x: 12.89, y: 11.06),
    ///     (x: 2.89, y: 8.94)
    /// ];
    /// approx::assert_relative_eq!(skewed, expected_output, epsilon = 1e-2);
    /// ```
    #[must_use]
    fn skew_xy(&self, degrees_x: T, degrees_y: T) -> Self;

    /// Mutable version of [`skew_xy`](Self::skew_xy).
    fn skew_xy_mut(&mut self, degrees_x: T, degrees_y: T);

    /// An affine transformation which skews a geometry around a point of `origin`, sheared by an
    /// angle along the x and y dimensions.
    ///
    /// The point of origin is *usually* given as the 2D bounding box centre of the geometry, in
    /// which case you can just use [`skew`](Self::skew) or [`skew_xy`](Self::skew_xy), but this method allows you
    /// to specify any point.
    ///
    /// # Examples
    ///
    /// ```
    /// use geo::Skew;
    /// use geo::{Polygon, polygon, point};
    ///
    /// let square: Polygon = polygon![
    ///     (x: 0., y: 0.),
    ///     (x: 10., y: 0.),
    ///     (x: 10., y: 10.),
    ///     (x: 0., y: 10.)
    /// ];
    ///
    /// let origin = point! { x: 2., y: 2. };
    /// let skewed = square.skew_around_point(45.0, 10.0, origin);
    ///
    /// let expected_output: Polygon = polygon![
    ///     (x: -2., y: -0.353),
    ///     (x: 8., y: 1.410),
    ///     (x: 18., y: 11.41),
    ///     (x: 8., y: 9.647)
    /// ];
    /// approx::assert_relative_eq!(skewed, expected_output, epsilon = 1e-2);
    /// ```
    #[must_use]
    fn skew_around_point(&self, degrees_x: T, degrees_y: T, origin: impl Into<Coord<T>>) -> Self;

    /// Mutable version of [`skew_around_point`](Self::skew_around_point).
    fn skew_around_point_mut(&mut self, degrees_x: T, degrees_y: T, origin: impl Into<Coord<T>>);
}

impl<T, IR, G> Skew<T> for G
where
    T: CoordFloat,
    IR: Into<Option<Rect<T>>>,
    G: Clone + AffineOps<T> + BoundingRect<T, Output = IR>,
{
    fn skew(&self, degrees: T) -> Self {
        self.skew_xy(degrees, degrees)
    }

    fn skew_mut(&mut self, degrees: T) {
        self.skew_xy_mut(degrees, degrees);
    }

    fn skew_xy(&self, degrees_x: T, degrees_y: T) -> Self {
        let origin = match self.bounding_rect().into() {
            Some(rect) => rect.center(),
            // Empty geometries have no bounding rect, but in that case
            // transforming is a no-op anyway.
            None => return self.clone(),
        };
        self.skew_around_point(degrees_x, degrees_y, origin)
    }

    fn skew_xy_mut(&mut self, degrees_x: T, degrees_y: T) {
        let origin = match self.bounding_rect().into() {
            Some(rect) => rect.center(),
            // Empty geometries have no bounding rect, but in that case
            // transforming is a no-op anyway.
            None => return,
        };
        self.skew_around_point_mut(degrees_x, degrees_y, origin);
    }

    fn skew_around_point(&self, xs: T, ys: T, origin: impl Into<Coord<T>>) -> Self {
        let transform = AffineTransform::skew(xs, ys, origin);
        self.affine_transform(&transform)
    }

    fn skew_around_point_mut(&mut self, xs: T, ys: T, origin: impl Into<Coord<T>>) {
        let transform = AffineTransform::skew(xs, ys, origin);
        self.affine_transform_mut(&transform);
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
        let sheared = ls.skew_around_point(45.0, 45.0, origin);
        assert_eq!(
            sheared,
            line_string![
                (x: -1.9999999999999991, y: 0.0),
                (x: 7.999999999999999, y: 10.0)
            ]
        );
    }
}
