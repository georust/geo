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
