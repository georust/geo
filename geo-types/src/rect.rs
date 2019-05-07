use crate::{Coordinate, CoordinateType};

/// A bounded 2D quadrilateral whose area is defined by minimum and maximum `Coordinates`.
#[derive(PartialEq, Clone, Copy, Debug)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct Rect<T>
where
    T: CoordinateType,
{
    pub min: Coordinate<T>,
    pub max: Coordinate<T>,
}

impl<T: CoordinateType> Rect<T> {
    /// Creates a new rectangle.
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
    /// assert_eq!(rect.min, Coordinate { x: 0., y: 0. });
    /// assert_eq!(rect.max, Coordinate { x: 10., y: 20. });
    /// ```
    pub fn new<C>(min: C, max: C) -> Rect<T>
    where
        C: Into<Coordinate<T>>,
    {
        let (min, max) = (min.into(), max.into());

        assert!(
            min.x < max.x && min.y < max.y,
            "Failed to create the Rectangle: the minimum x/y values must be smaller than the maximum x/y values"
        );

        Rect { min, max }
    }

    pub fn width(self) -> T {
        self.max.x - self.min.x
    }

    pub fn height(self) -> T {
        self.max.y - self.min.y
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::{Coordinate};

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
    #[should_panic]
    fn line_panic() {
        let _ = Rect::new((10, 20), (10, 20));
    }

    #[test]
    fn rect_width() {
        let rect = Rect::new((10, 10), (20, 20));
        assert_eq!(rect.width(), 10);
    }

    #[test]
    fn rect_height() {
        let rect = Rect::new((10., 10.), (20., 20.));
        assert_eq!(rect.height(), 10.);
    }
}