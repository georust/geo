use crate::{polygon, Coordinate, CoordinateType, Polygon};

/// A bounded 2D quadrilateral whose area is defined by minimum and maximum `Coordinate`s.
///
/// The constructors and setters ensure the maximum `Coordinate` is greater than or equal to the
/// minimum. Thus, a `Rect`s width, height, and area is guaranteed to be greater than or equal to
/// zero.
///
/// # Examples
///
/// ```
/// use geo_types::{Coordinate, Rect};
///
/// let rect = Rect::new(
///     Coordinate { x: 0., y: 4.},
///     Coordinate { x: 3., y: 10.},
/// );
///
/// assert_eq!(3., rect.width());
/// assert_eq!(6., rect.height());
/// ```
#[derive(Eq, PartialEq, Clone, Copy, Debug, Hash)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct Rect<T>
where
    T: CoordinateType,
{
    min: Coordinate<T>,
    max: Coordinate<T>,
}

impl<T: CoordinateType> Rect<T> {
    /// Constructor to creates a new rectangle from coordinates, where `min` denotes to the
    /// coordinates of the bottom-left corner, and `max` denotes to the coordinates of the
    /// top-right corner
    ///
    /// # Panics
    ///
    /// Panics if `min`'s x/y coordinate is larger than that of the `max`'s.
    ///
    /// # Examples
    ///
    /// ```
    /// use geo_types::{Coordinate, Rect};
    ///
    /// let rect = Rect::new(
    ///     Coordinate { x: 0., y: 0. },
    ///     Coordinate { x: 10., y: 20. }
    /// );
    /// ```
    pub fn new<C>(min: C, max: C) -> Rect<T>
    where
        C: Into<Coordinate<T>>,
    {
        Rect::try_new(min, max).unwrap()
    }

    pub fn try_new<C>(min: C, max: C) -> Result<Rect<T>, InvalidRectCoordinatesError>
    where
        C: Into<Coordinate<T>>,
    {
        let rect = Rect {
            min: min.into(),
            max: max.into(),
        };

        if rect.has_valid_bounds() {
            Ok(rect)
        } else {
            Err(InvalidRectCoordinatesError)
        }
    }

    pub fn min(self) -> Coordinate<T> {
        self.min
    }

    /// Set the `Rect`’s minimum coordinate.
    ///
    /// # Panics
    ///
    /// Panics if `min`’s x/y is greater than the maximum coordinate’s x/y.
    pub fn set_min<C>(&mut self, min: C)
    where
        C: Into<Coordinate<T>>,
    {
        self.min = min.into();
        self.assert_valid_bounds();
    }

    pub fn max(self) -> Coordinate<T> {
        self.max
    }

    /// Set the `Rect`’s maximum coordinate.
    ///
    /// # Panics
    ///
    /// Panics if `max`’s x/y is less than the minimum coordinate’s x/y.
    pub fn set_max<C>(&mut self, max: C)
    where
        C: Into<Coordinate<T>>,
    {
        self.max = max.into();
        self.assert_valid_bounds();
    }

    pub fn width(self) -> T {
        self.max().x - self.min().x
    }

    pub fn height(self) -> T {
        self.max().y - self.min().y
    }

    /// Create a `Polygon` from the `Rect`.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use geo_types::{Coordinate, Rect, polygon};
    ///
    /// let rect = Rect::new(
    ///     Coordinate { x: 0., y: 0. },
    ///     Coordinate { x: 10., y: 20. },
    /// );
    ///
    /// assert_eq!(
    ///     rect.to_polygon(),
    ///     polygon![
    ///         (x: 0., y: 0.),
    ///         (x: 0., y: 20.),
    ///         (x: 10., y: 20.),
    ///         (x: 10., y: 0.),
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

    fn assert_valid_bounds(&self) {
        if !self.has_valid_bounds() {
            panic!(RECT_INVALID_BOUNDS_ERROR);
        }
    }

    fn has_valid_bounds(&self) -> bool {
        self.min.x <= self.max.x && self.min.y <= self.max.y
    }
}

static RECT_INVALID_BOUNDS_ERROR: &str = "Failed to create Rect: 'min' coordinate's x/y value must be smaller or equal to the 'max' x/y value";

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct InvalidRectCoordinatesError;

impl std::error::Error for InvalidRectCoordinatesError {}

impl std::fmt::Display for InvalidRectCoordinatesError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", RECT_INVALID_BOUNDS_ERROR)
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::Coordinate;

    #[test]
    fn rect() {
        let rect = Rect::new((10, 10), (20, 20));
        assert_eq!(rect.min, Coordinate { x: 10, y: 10 });
        assert_eq!(rect.max, Coordinate { x: 20, y: 20 });
    }

    #[test]
    #[should_panic]
    fn rect_panic() {
        let _ = Rect::new((10, 20), (20, 10));
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
}
