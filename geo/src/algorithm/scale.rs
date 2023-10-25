use crate::{AffineOps, AffineTransform, BoundingRect, Coord, CoordFloat, CoordNum, Rect};

/// An affine transformation which scales a geometry up or down by a factor.
///
/// ## Performance
///
/// If you will be performing multiple transformations, like [`Scale`],
/// [`Skew`](crate::Skew), [`Translate`](crate::Translate), or [`Rotate`](crate::Rotate), it is more
/// efficient to compose the transformations and apply them as a single operation using the
/// [`AffineOps`] trait.
pub trait Scale<T: CoordNum> {
    /// Scale a geometry from it's bounding box center.
    ///
    /// # Examples
    ///
    /// ```
    /// use geo::Scale;
    /// use geo::{LineString, line_string};
    ///
    /// let ls: LineString = line_string![(x: 0., y: 0.), (x: 10., y: 10.)];
    ///
    /// let scaled = ls.scale(2.);
    ///
    /// assert_eq!(scaled, line_string![
    ///     (x: -5., y: -5.),
    ///     (x: 15., y: 15.)
    /// ]);
    /// ```
    #[must_use]
    fn scale(&self, scale_factor: T) -> Self;

    /// Mutable version of [`scale`](Self::scale)
    fn scale_mut(&mut self, scale_factor: T);

    /// Scale a geometry from it's bounding box center, using different values for `x_factor` and
    /// `y_factor` to distort the geometry's [aspect ratio](https://en.wikipedia.org/wiki/Aspect_ratio).
    ///
    /// # Examples
    ///
    /// ```
    /// use geo::Scale;
    /// use geo::{LineString, line_string};
    ///
    /// let ls: LineString = line_string![(x: 0., y: 0.), (x: 10., y: 10.)];
    ///
    /// let scaled = ls.scale_xy(2., 4.);
    ///
    /// assert_eq!(scaled, line_string![
    ///     (x: -5., y: -15.),
    ///     (x: 15., y: 25.)
    /// ]);
    /// ```
    #[must_use]
    fn scale_xy(&self, x_factor: T, y_factor: T) -> Self;

    /// Mutable version of [`scale_xy`](Self::scale_xy).
    fn scale_xy_mut(&mut self, x_factor: T, y_factor: T);

    /// Scale a geometry around a point of `origin`.
    ///
    /// The point of origin is *usually* given as the 2D bounding box centre of the geometry, in
    /// which case you can just use [`scale`](Self::scale) or [`scale_xy`](Self::scale_xy), but
    /// this method allows you to specify any point.
    ///
    /// # Examples
    ///
    /// ```
    /// use geo::Scale;
    /// use geo::{LineString, line_string, Coord};
    ///
    /// let ls: LineString = line_string![(x: 0., y: 0.), (x: 10., y: 10.)];
    ///
    /// let scaled = ls.scale_around_point(2., 4., Coord { x: 100., y: 100. });
    ///
    /// assert_eq!(scaled, line_string![
    ///     (x: -100., y: -300.),
    ///     (x: -80., y: -260.)
    /// ]);
    /// ```
    #[must_use]
    fn scale_around_point(&self, x_factor: T, y_factor: T, origin: impl Into<Coord<T>>) -> Self;

    /// Mutable version of [`scale_around_point`](Self::scale_around_point).
    fn scale_around_point_mut(&mut self, x_factor: T, y_factor: T, origin: impl Into<Coord<T>>);
}

impl<T, IR, G> Scale<T> for G
where
    T: CoordFloat,
    IR: Into<Option<Rect<T>>>,
    G: Clone + AffineOps<T> + BoundingRect<T, Output = IR>,
{
    fn scale(&self, scale_factor: T) -> Self {
        self.scale_xy(scale_factor, scale_factor)
    }

    fn scale_mut(&mut self, scale_factor: T) {
        self.scale_xy_mut(scale_factor, scale_factor);
    }

    fn scale_xy(&self, x_factor: T, y_factor: T) -> Self {
        let origin = match self.bounding_rect().into() {
            Some(rect) => rect.center(),
            // Empty geometries have no bounding rect, but in that case
            // transforming is a no-op anyway.
            None => return self.clone(),
        };
        self.scale_around_point(x_factor, y_factor, origin)
    }

    fn scale_xy_mut(&mut self, x_factor: T, y_factor: T) {
        let origin = match self.bounding_rect().into() {
            Some(rect) => rect.center(),
            // Empty geometries have no bounding rect, but in that case
            // transforming is a no-op anyway.
            None => return,
        };
        self.scale_around_point_mut(x_factor, y_factor, origin);
    }

    fn scale_around_point(&self, x_factor: T, y_factor: T, origin: impl Into<Coord<T>>) -> Self {
        let affineop = AffineTransform::scale(x_factor, y_factor, origin);
        self.affine_transform(&affineop)
    }

    fn scale_around_point_mut(&mut self, x_factor: T, y_factor: T, origin: impl Into<Coord<T>>) {
        let affineop = AffineTransform::scale(x_factor, y_factor, origin);
        self.affine_transform_mut(&affineop)
    }
}
