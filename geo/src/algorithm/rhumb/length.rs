use num_traits::FromPrimitive;

use crate::{CoordFloat, Length, Line, LineString, MultiLineString, Rhumb};

#[deprecated(
    since = "0.29.0",
    note = "Please use the `Rhumb.length(&line)` via the `Length` trait instead."
)]
/// Determine the length of a geometry assuming each segment is a [rhumb line].
///
/// [rhumb line]: https://en.wikipedia.org/wiki/Rhumb_line
///
/// *Note*: this implementation uses a mean earth radius of 6371.088 km, based on the [recommendation of
/// the IUGG](ftp://athena.fsv.cvut.cz/ZFG/grs80-Moritz.pdf)
pub trait RhumbLength<T, RHS = Self> {
    /// Determine the length of a geometry assuming each segment is a [rhumb line].
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
    /// let length = linestring.rhumb_length();
    ///
    /// assert_eq!(
    ///     5_794_129., // meters
    ///     length.round()
    /// );
    /// ```
    ///
    /// [rhumb line]: https://en.wikipedia.org/wiki/Rhumb_line
    fn rhumb_length(&self) -> T;
}

#[allow(deprecated)]
impl<T> RhumbLength<T> for Line<T>
where
    T: CoordFloat + FromPrimitive,
{
    fn rhumb_length(&self) -> T {
        Rhumb.length(self)
    }
}

#[allow(deprecated)]
impl<T> RhumbLength<T> for LineString<T>
where
    T: CoordFloat + FromPrimitive,
{
    fn rhumb_length(&self) -> T {
        Rhumb.length(self)
    }
}

#[allow(deprecated)]
impl<T> RhumbLength<T> for MultiLineString<T>
where
    T: CoordFloat + FromPrimitive,
{
    fn rhumb_length(&self) -> T {
        Rhumb.length(self)
    }
}
