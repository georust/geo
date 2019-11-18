use crate::{Coordinate, CoordinateType};

/// A bounded 2D quadrilateral whose area is defined by minimum and maximum `Coordinates`.
#[derive(PartialEq, Clone, Copy, Debug, Hash)]
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
    ///     Coordinate { x: 10., y: 20. },
    /// );
    ///
    /// assert_eq!(rect.min(), Coordinate { x: 0., y: 0. });
    /// assert_eq!(rect.max(), Coordinate { x: 10., y: 20. });
    /// ```
    pub fn new<C>(min: C, max: C) -> Rect<T>
    where
        C: Into<Coordinate<T>>,
    {
        let (min, max) = (min.into(), max.into());

        Self::assert_valid_bounds(min, max);

        Rect { min, max }
    }

    pub fn min(self) -> Coordinate<T> {
        self.min
    }

    pub fn set_min<C>(&mut self, min: C)
    where
        C: Into<Coordinate<T>>,
    {
        self.min = min.into();
        Self::assert_valid_bounds(self.min, self.max);
    }

    pub fn max(self) -> Coordinate<T> {
        self.max
    }

    pub fn set_max<C>(&mut self, max: C)
    where
        C: Into<Coordinate<T>>,
    {
        self.max = max.into();
        Self::assert_valid_bounds(self.min, self.max);
    }

    pub fn width(self) -> T {
        self.max().x - self.min().x
    }

    pub fn height(self) -> T {
        self.max().y - self.min().y
    }

    fn assert_valid_bounds(min: Coordinate<T>, max: Coordinate<T>) {
        assert!(
            min.x <= max.x && min.y <= max.y,
            "Failed to create the Rect type: 'min' coordinate's x/y value must be smaller or equal to the 'max' x/y value"
        );
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
