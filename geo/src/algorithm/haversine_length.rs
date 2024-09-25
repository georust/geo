use num_traits::FromPrimitive;

use crate::{CoordFloat, Line, LineString, MultiLineString};
use crate::{Distance, Haversine};

#[deprecated(
    since = "0.29.0",
    note = "Please use the `line.length::<Haversine>()` via the `Length` trait instead."
)]
/// Determine the length of a geometry using the [haversine formula].
///
/// [haversine formula]: https://en.wikipedia.org/wiki/Haversine_formula
///
/// *Note*: this implementation uses a mean earth radius of 6371.0088 km, based on the [recommendation of
/// the IUGG](ftp://athena.fsv.cvut.cz/ZFG/grs80-Moritz.pdf)
pub trait HaversineLength<T, RHS = Self> {
    /// Determine the length of a geometry using the [haversine formula].
    ///
    /// # Units
    ///
    /// - return value: meters
    ///
    /// # Examples
    ///
    /// ```
    /// use geo::prelude::*;
    /// use geo::LineString;
    ///
    /// let linestring = LineString::<f64>::from(vec![
    ///     // New York City
    ///     (-74.006, 40.7128),
    ///     // London
    ///     (-0.1278, 51.5074),
    /// ]);
    ///
    /// let length = linestring.haversine_length();
    ///
    /// assert_eq!(
    ///     5_570_230., // meters
    ///     length.round()
    /// );
    /// ```
    ///
    /// [haversine formula]: https://en.wikipedia.org/wiki/Haversine_formula
    fn haversine_length(&self) -> T;
}

impl<T> HaversineLength<T> for Line<T>
where
    T: CoordFloat + FromPrimitive,
{
    fn haversine_length(&self) -> T {
        let (start, end) = self.points();
        Haversine::distance(start, end)
    }
}

impl<T> HaversineLength<T> for LineString<T>
where
    T: CoordFloat + FromPrimitive,
{
    fn haversine_length(&self) -> T {
        self.lines().fold(T::zero(), |total_length, line| {
            total_length + line.haversine_length()
        })
    }
}

impl<T> HaversineLength<T> for MultiLineString<T>
where
    T: CoordFloat + FromPrimitive,
{
    fn haversine_length(&self) -> T {
        self.0
            .iter()
            .fold(T::zero(), |total, line| total + line.haversine_length())
    }
}
