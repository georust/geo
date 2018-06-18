use num_traits::{Float, ToPrimitive};
use {CoordinateType, Point};

/// A primitive type which holds `x` and `y` position information
#[derive(PartialEq, Clone, Copy, Debug)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct Coordinate<T>
where
    T: CoordinateType,
{
    pub x: T,
    pub y: T,
}

impl<T: CoordinateType> From<(T, T)> for Coordinate<T> {
    fn from(coords: (T, T)) -> Self {
        Coordinate {
            x: coords.0,
            y: coords.1,
        }
    }
}

impl<T: CoordinateType> From<[T; 2]> for Coordinate<T> {
    fn from(coords: [T; 2]) -> Self {
        Coordinate {
            x: coords[0],
            y: coords[1],
        }
    }
}

impl<T: CoordinateType> From<Point<T>> for Coordinate<T> {
    fn from(point: Point<T>) -> Self {
        point.0
    }
}

impl<T>Coordinate<T> 
where
    T: CoordinateType + ToPrimitive,
{

    /// Returns a tuple that contains the x/horizontal & y/vertical component of the coordinate.
    ///
    /// # Examples
    ///
    /// ```
    /// use geo_types::Coordinate;
    ///
    /// 
    ///  let c = Coordinate {
    ///        x: 40.02f64,
    ///        y: 116.34,
    ///    };
    /// let (x, y) = c.x_y();
    ///
    /// assert_eq!(y, 116.34);
    /// assert_eq!(x, 40.02f64);
    /// ```
    pub fn x_y(&self) -> (T, T) {
        (self.x, self.y)
    }
}
