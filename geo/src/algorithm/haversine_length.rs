
use num_traits::{Float, FromPrimitive};

use ::{Line, LineString, MultiLineString};
use algorithm::haversine_distance::HaversineDistance;

/// Calculation of the length

pub trait HaversineLength<T, RHS = Self> {
    /// Calculation of the length of a Line
    ///
    /// # Examples
    ///
    /// ```
    /// use geo::{Point, LineString, Coordinate};
    /// use geo::algorithm::haversine_length::HaversineLength;
    ///
    /// let mut vec = Vec::new();
    /// vec.push(Point::new(40.02f64, 116.34));
    /// vec.push(Point::new(42.02f64, 116.34));
    /// let linestring = LineString::from(vec);
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
        let (start, end) = self.points();
        start.haversine_distance(&end)
    }
}

impl<T> HaversineLength<T> for LineString<T>
    where T: Float + FromPrimitive
{
    fn haversine_length(&self) -> T {
        self.lines()
            .fold(T::zero(), |total_length, line| total_length + line.haversine_length())
    }
}

impl<T> HaversineLength<T> for MultiLineString<T>
    where T: Float + FromPrimitive
{
    fn haversine_length(&self) -> T {
        self.0.iter().fold(T::zero(), |total, line| total + line.haversine_length())
    }
}
