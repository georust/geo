use crate::{coord, polygon, Coord, CoordFloat, CoordNum, Line, Polygon};

#[cfg(any(feature = "approx", test))]
use approx::{AbsDiffEq, RelativeEq};

/// An _axis-aligned_ bounded 2D rectangle whose area is
/// defined by minimum and maximum `Coord`s.
///
/// The constructors and setters ensure the maximum
/// `Coord` is greater than or equal to the minimum.
/// Thus, a `Rect`s width, height, and area is guaranteed to
/// be greater than or equal to zero.
///
/// **Note.** While `Rect` implements `MapCoords` and
/// `RotatePoint` algorithmic traits, the usage is expected
/// to maintain the axis alignment. In particular, only
/// rotation by integer multiples of 90 degrees, will
/// preserve the original shape. In other cases, the min,
/// and max points are rotated or transformed, and a new
/// rectangle is created (with coordinate swaps to ensure
/// min < max).
///
/// # Examples
///
/// ```
/// use geo_types::{coord, Rect};
///
/// let rect = Rect::new(
///     coord! { x: 0., y: 4.},
///     coord! { x: 3., y: 10.},
/// );
///
/// assert_eq!(3., rect.width());
/// assert_eq!(6., rect.height());
/// assert_eq!(
///     coord! { x: 1.5, y: 7. },
///     rect.center()
/// );
/// ```
#[derive(Eq, PartialEq, Clone, Copy, Debug, Hash)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct Rect<T: CoordNum = f64> {
    min: Coord<T>,
    max: Coord<T>,
}

impl<T: CoordNum> Rect<T> {
    /// Creates a new rectangle from two corner coordinates.
    ///
    /// # Examples
    ///
    /// ```
    /// use geo_types::{coord, Rect};
    ///
    /// let rect = Rect::new(
    ///     coord! { x: 10., y: 20. },
    ///     coord! { x: 30., y: 10. }
    /// );
    /// assert_eq!(rect.min(), coord! { x: 10., y: 10. });
    /// assert_eq!(rect.max(), coord! { x: 30., y: 20. });
    /// ```
    pub fn new<C>(c1: C, c2: C) -> Self
    where
        C: Into<Coord<T>>,
    {
        let c1 = c1.into();
        let c2 = c2.into();
        let (min_x, max_x) = if c1.x < c2.x {
            (c1.x, c2.x)
        } else {
            (c2.x, c1.x)
        };
        let (min_y, max_y) = if c1.y < c2.y {
            (c1.y, c2.y)
        } else {
            (c2.y, c1.y)
        };
        Self {
            min: coord! { x: min_x, y: min_y },
            max: coord! { x: max_x, y: max_y },
        }
    }

    #[deprecated(
        since = "0.6.2",
        note = "Use `Rect::new` instead, since `Rect::try_new` will never Error"
    )]
    #[allow(deprecated)]
    pub fn try_new<C>(c1: C, c2: C) -> Result<Rect<T>, InvalidRectCoordinatesError>
    where
        C: Into<Coord<T>>,
    {
        Ok(Rect::new(c1, c2))
    }

    /// Returns the minimum `Coord` of the `Rect`.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use geo_types::{coord, Rect};
    ///
    /// let rect = Rect::new(
    ///     coord! { x: 5., y: 5. },
    ///     coord! { x: 15., y: 15. },
    /// );
    ///
    /// assert_eq!(rect.min(), coord! { x: 5., y: 5. });
    /// ```
    pub fn min(self) -> Coord<T> {
        self.min
    }

    /// Set the `Rect`’s minimum coordinate.
    ///
    /// # Panics
    ///
    /// Panics if `min`’s x/y is greater than the maximum coordinate’s x/y.
    pub fn set_min<C>(&mut self, min: C)
    where
        C: Into<Coord<T>>,
    {
        self.min = min.into();
        self.assert_valid_bounds();
    }

    /// Returns the maximum `Coord` of the `Rect`.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use geo_types::{coord, Rect};
    ///
    /// let rect = Rect::new(
    ///     coord! { x: 5., y: 5. },
    ///     coord! { x: 15., y: 15. },
    /// );
    ///
    /// assert_eq!(rect.max(), coord! { x: 15., y: 15. });
    /// ```
    pub fn max(self) -> Coord<T> {
        self.max
    }

    /// Set the `Rect`’s maximum coordinate.
    ///
    /// # Panics
    ///
    /// Panics if `max`’s x/y is less than the minimum coordinate’s x/y.
    pub fn set_max<C>(&mut self, max: C)
    where
        C: Into<Coord<T>>,
    {
        self.max = max.into();
        self.assert_valid_bounds();
    }

    /// Returns the width of the `Rect`.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use geo_types::{coord, Rect};
    ///
    /// let rect = Rect::new(
    ///     coord! { x: 5., y: 5. },
    ///     coord! { x: 15., y: 15. },
    /// );
    ///
    /// assert_eq!(rect.width(), 10.);
    /// ```
    pub fn width(self) -> T {
        self.max().x - self.min().x
    }

    /// Returns the height of the `Rect`.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use geo_types::{coord, Rect};
    ///
    /// let rect = Rect::new(
    ///     coord! { x: 5., y: 5. },
    ///     coord! { x: 15., y: 15. },
    /// );
    ///
    /// assert_eq!(rect.height(), 10.);
    /// ```
    pub fn height(self) -> T {
        self.max().y - self.min().y
    }

    /// Create a `Polygon` from the `Rect`.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use geo_types::{coord, Rect, polygon};
    ///
    /// let rect = Rect::new(
    ///     coord! { x: 0., y: 0. },
    ///     coord! { x: 1., y: 2. },
    /// );
    ///
    /// assert_eq!(
    ///     rect.to_polygon(),
    ///     polygon![
    ///         (x: 0., y: 0.),
    ///         (x: 0., y: 2.),
    ///         (x: 1., y: 2.),
    ///         (x: 1., y: 0.),
    ///         (x: 0., y: 0.),
    ///     ],
    /// );
    /// ```
    pub fn to_polygon(self) -> Polygon<T> {
        polygon![
            (x: self.min.x, y: self.min.y),
            (x: self.min.x, y: self.max.y),
            (x: self.max.x, y: self.max.y),
            (x: self.max.x, y: self.min.y),
            (x: self.min.x, y: self.min.y),
        ]
    }

    pub fn to_lines(&self) -> [Line<T>; 4] {
        [
            Line::new(
                coord! {
                    x: self.min.x,
                    y: self.min.y,
                },
                coord! {
                    x: self.min.x,
                    y: self.max.y,
                },
            ),
            Line::new(
                coord! {
                    x: self.min.x,
                    y: self.max.y,
                },
                coord! {
                    x: self.max.x,
                    y: self.max.y,
                },
            ),
            Line::new(
                coord! {
                    x: self.max.x,
                    y: self.max.y,
                },
                coord! {
                    x: self.max.x,
                    y: self.min.y,
                },
            ),
            Line::new(
                coord! {
                    x: self.max.x,
                    y: self.min.y,
                },
                coord! {
                    x: self.min.x,
                    y: self.min.y,
                },
            ),
        ]
    }

    /// Split a rectangle into two rectangles along the X-axis with equal widths.
    ///
    /// # Examples
    ///
    /// ```
    /// let rect = geo_types::Rect::new(
    ///     geo_types::coord! { x: 0., y: 0. },
    ///     geo_types::coord! { x: 4., y: 4. },
    /// );
    ///
    /// let [rect1, rect2] = rect.split_x();
    ///
    /// assert_eq!(
    ///     geo_types::Rect::new(
    ///         geo_types::coord! { x: 0., y: 0. },
    ///         geo_types::coord! { x: 2., y: 4. },
    ///     ),
    ///     rect1,
    /// );
    /// assert_eq!(
    ///     geo_types::Rect::new(
    ///         geo_types::coord! { x: 2., y: 0. },
    ///         geo_types::coord! { x: 4., y: 4. },
    ///     ),
    ///     rect2,
    /// );
    /// ```
    pub fn split_x(self) -> [Rect<T>; 2] {
        let two = T::one() + T::one();
        let mid_x = self.min().x + self.width() / two;
        [
            Rect::new(self.min(), coord! { x: mid_x, y: self.max().y }),
            Rect::new(coord! { x: mid_x, y: self.min().y }, self.max()),
        ]
    }

    /// Split a rectangle into two rectangles along the Y-axis with equal heights.
    ///
    /// # Examples
    ///
    /// ```
    /// let rect = geo_types::Rect::new(
    ///     geo_types::coord! { x: 0., y: 0. },
    ///     geo_types::coord! { x: 4., y: 4. },
    /// );
    ///
    /// let [rect1, rect2] = rect.split_y();
    ///
    /// assert_eq!(
    ///     geo_types::Rect::new(
    ///         geo_types::coord! { x: 0., y: 0. },
    ///         geo_types::coord! { x: 4., y: 2. },
    ///     ),
    ///     rect1,
    /// );
    /// assert_eq!(
    ///     geo_types::Rect::new(
    ///         geo_types::coord! { x: 0., y: 2. },
    ///         geo_types::coord! { x: 4., y: 4. },
    ///     ),
    ///     rect2,
    /// );
    /// ```
    pub fn split_y(self) -> [Rect<T>; 2] {
        let two = T::one() + T::one();
        let mid_y = self.min().y + self.height() / two;
        [
            Rect::new(self.min(), coord! { x: self.max().x, y: mid_y }),
            Rect::new(coord! { x: self.min().x, y: mid_y }, self.max()),
        ]
    }

    fn assert_valid_bounds(&self) {
        if !self.has_valid_bounds() {
            panic!("{}", RECT_INVALID_BOUNDS_ERROR);
        }
    }

    fn has_valid_bounds(&self) -> bool {
        self.min.x <= self.max.x && self.min.y <= self.max.y
    }
}

impl<T: CoordFloat> Rect<T> {
    /// Returns the center `Coord` of the `Rect`.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use geo_types::{coord, Rect};
    ///
    /// let rect = Rect::new(
    ///     coord! { x: 5., y: 5. },
    ///     coord! { x: 15., y: 15. },
    /// );
    ///
    /// assert_eq!(rect.center(), coord! { x: 10., y: 10. });
    /// ```
    pub fn center(self) -> Coord<T> {
        let two = T::one() + T::one();
        coord! {
            x: (self.max.x + self.min.x) / two,
            y: (self.max.y + self.min.y) / two,
        }
    }
}

static RECT_INVALID_BOUNDS_ERROR: &str = "Failed to create Rect: 'min' coordinate's x/y value must be smaller or equal to the 'max' x/y value";

#[cfg(any(feature = "approx", test))]
impl<T> RelativeEq for Rect<T>
where
    T: AbsDiffEq<Epsilon = T> + CoordNum + RelativeEq,
{
    #[inline]
    fn default_max_relative() -> Self::Epsilon {
        T::default_max_relative()
    }

    /// Equality assertion within a relative limit.
    ///
    /// # Examples
    ///
    /// ```
    /// use geo_types::Rect;
    ///
    /// let a = Rect::new((0.0, 0.0), (10.0, 10.0));
    /// let b = Rect::new((0.0, 0.0), (10.01, 10.0));
    ///
    /// approx::assert_relative_eq!(a, b, max_relative=0.1);
    /// approx::assert_relative_ne!(a, b, max_relative=0.0001);
    /// ```
    #[inline]
    fn relative_eq(
        &self,
        other: &Self,
        epsilon: Self::Epsilon,
        max_relative: Self::Epsilon,
    ) -> bool {
        if !self.min.relative_eq(&other.min, epsilon, max_relative) {
            return false;
        }

        if !self.max.relative_eq(&other.max, epsilon, max_relative) {
            return false;
        }

        true
    }
}

#[cfg(any(feature = "approx", test))]
impl<T> AbsDiffEq for Rect<T>
where
    T: AbsDiffEq<Epsilon = T> + CoordNum,
    T::Epsilon: Copy,
{
    type Epsilon = T;

    #[inline]
    fn default_epsilon() -> Self::Epsilon {
        T::default_epsilon()
    }

    /// Equality assertion with an absolute limit.
    ///
    /// # Examples
    ///
    /// ```
    /// use geo_types::{point, Rect};
    ///
    /// let a = Rect::new((0.0, 0.0), (10.0, 10.0));
    /// let b = Rect::new((0.0, 0.0), (10.01, 10.0));
    ///
    /// approx::abs_diff_eq!(a, b, epsilon=0.1);
    /// approx::abs_diff_ne!(a, b, epsilon=0.001);
    /// ```
    #[inline]
    fn abs_diff_eq(&self, other: &Self, epsilon: Self::Epsilon) -> bool {
        if !self.min.abs_diff_eq(&other.min, epsilon) {
            return false;
        }

        if !self.max.abs_diff_eq(&other.max, epsilon) {
            return false;
        }

        true
    }
}

#[deprecated(
    since = "0.6.2",
    note = "Use `Rect::new` instead, since `Rect::try_new` will never Error"
)]
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct InvalidRectCoordinatesError;

#[cfg(feature = "std")]
#[allow(deprecated)]
impl std::error::Error for InvalidRectCoordinatesError {}

#[allow(deprecated)]
impl core::fmt::Display for InvalidRectCoordinatesError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "{RECT_INVALID_BOUNDS_ERROR}")
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::coord;

    #[test]
    fn rect() {
        let rect = Rect::new((10, 10), (20, 20));
        assert_eq!(rect.min, coord! { x: 10, y: 10 });
        assert_eq!(rect.max, coord! { x: 20, y: 20 });

        let rect = Rect::new((20, 20), (10, 10));
        assert_eq!(rect.min, coord! { x: 10, y: 10 });
        assert_eq!(rect.max, coord! { x: 20, y: 20 });

        let rect = Rect::new((10, 20), (20, 10));
        assert_eq!(rect.min, coord! { x: 10, y: 10 });
        assert_eq!(rect.max, coord! { x: 20, y: 20 });
    }

    #[test]
    fn rect_width() {
        let rect = Rect::new((10, 10), (20, 20));
        assert_eq!(rect.width(), 10);
    }

    #[test]
    fn rect_height() {
        let rect = Rect::new((10., 10.), (20., 20.));
        assert_relative_eq!(rect.height(), 10.);
    }

    #[test]
    fn rect_center() {
        assert_relative_eq!(
            Rect::new((0., 10.), (10., 90.)).center(),
            Coord::from((5., 50.))
        );
        assert_relative_eq!(
            Rect::new((-42., -42.), (42., 42.)).center(),
            Coord::from((0., 0.))
        );
        assert_relative_eq!(
            Rect::new((0., 0.), (0., 0.)).center(),
            Coord::from((0., 0.))
        );
    }
}
