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

#[cfg(test)]
mod hegel_props {
    use super::Scale;
    use crate::utils::pbt_gens::{coords, star_polygons};
    use crate::{AffineOps, AffineTransform, Area, BoundingRect, Point};
    use hegel::generators;

    fn factor(tc: &hegel::TestCase) -> f64 {
        tc.draw(generators::floats::<f64>().min_value(0.25).max_value(4.0))
    }

    // `scale` is documented as scaling "from it's bounding box center", so
    // scaling by 1 must be a no-op.
    #[hegel::test]
    fn scaling_by_one_changes_nothing(tc: hegel::TestCase) {
        let polygon = tc.draw(star_polygons());
        assert_eq!(polygon.scale(1.0), polygon);
    }

    // Scaling by a factor and then by its reciprocal about the same fixed point
    // returns the original geometry.
    #[hegel::test]
    fn scaling_about_a_point_is_undone_by_the_reciprocal_factor(tc: hegel::TestCase) {
        let polygon = tc.draw(star_polygons());
        let origin = Point::from(tc.draw(coords(1e3)));
        let f = factor(&tc);
        let round_tripped =
            polygon
                .scale_around_point(f, f, origin)
                .scale_around_point(1.0 / f, 1.0 / f, origin);
        for (before, after) in polygon.exterior().0.iter().zip(&round_tripped.exterior().0) {
            let tolerance = 1e-9
                * before
                    .x
                    .abs()
                    .max(before.y.abs())
                    .max(origin.x().abs())
                    .max(1.0);
            assert!(
                (after.x - before.x).abs() <= tolerance && (after.y - before.y).abs() <= tolerance,
                "{before:?} came back as {after:?}"
            );
        }
    }

    // "Scale a geometry from it's bounding box center", so the bounding box
    // centre is the fixed point of `scale_xy`.
    #[hegel::test]
    fn scale_xy_fixes_the_bounding_box_centre(tc: hegel::TestCase) {
        let polygon = tc.draw(star_polygons());
        let centre = polygon
            .bounding_rect()
            .expect("polygons have coords")
            .center();
        let scaled = polygon.scale_xy(factor(&tc), factor(&tc));
        let scaled_centre = scaled
            .bounding_rect()
            .expect("polygons have coords")
            .center();
        let tolerance = 1e-9 * centre.x.abs().max(centre.y.abs()).max(1.0);
        assert!(
            (scaled_centre.x - centre.x).abs() <= tolerance
                && (scaled_centre.y - centre.y).abs() <= tolerance,
            "bounding box centre moved from {centre:?} to {scaled_centre:?}"
        );
    }

    // Uniform scaling multiplies area by the square of the factor.
    #[hegel::test]
    fn uniform_scaling_squares_the_factor_into_the_area(tc: hegel::TestCase) {
        let polygon = tc.draw(star_polygons());
        let f = factor(&tc);
        assert_relative_eq!(
            polygon.scale(f).unsigned_area(),
            polygon.unsigned_area() * f * f,
            max_relative = 1e-9
        );
    }

    // `scale_around_point` delegates to `AffineTransform::scale`; the trait's
    // performance note directs callers to compose transforms instead of
    // chaining, which relies on the two agreeing.
    #[hegel::test]
    fn scale_around_point_matches_the_affine_scaling(tc: hegel::TestCase) {
        let polygon = tc.draw(star_polygons());
        let origin = Point::from(tc.draw(coords(1e3)));
        let (x, y) = (factor(&tc), factor(&tc));
        assert_eq!(
            polygon.scale_around_point(x, y, origin),
            polygon.affine_transform(&AffineTransform::scale(x, y, origin))
        );
    }
}
