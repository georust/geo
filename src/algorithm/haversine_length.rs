
use num_traits::{Float, FromPrimitive};

use types::{Line, LineString, MultiLineString};
use algorithm::haversine_distance::HaversineDistance;

/// Calculation of the length

pub trait HaversineLength<T, RHS = Self> {
    /// Calculation of the length of a Line
    ///
    /// ```
    /// use geo::{Point, LineString, Coordinate};
    /// use geo::algorithm::haversine_length::HaversineLength;
    ///
    /// let mut vec = Vec::new();
    /// vec.push(Point::new(40.02f64, 116.34));
    /// vec.push(Point::new(42.02f64, 116.34));
    /// let linestring = LineString(vec);
    ///
    /// println!("HaversineLength {}", linestring.haversine_length());
    /// ```
    ///
    fn haversine_length(&self) -> T;
}

impl<T> HaversineLength<T> for Line<T>
    where T: Float + FromPrimitive
{
    fn haversine_length(&self) -> T {
        self.start.haversine_distance(&self.end)
    }
}

impl<T> HaversineLength<T> for LineString<T>
    where T: Float + FromPrimitive
{
    fn haversine_length(&self) -> T {
        self.0.windows(2)
              .fold(T::zero(), |total_length, p| total_length + p[0].haversine_distance(&p[1]))
    }
}

impl<T> HaversineLength<T> for MultiLineString<T>
    where T: Float + FromPrimitive
{
    fn haversine_length(&self) -> T {
        self.iter().fold(T::zero(), |total, line| total + line.haversine_length())
    }
}
