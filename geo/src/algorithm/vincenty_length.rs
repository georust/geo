use num_traits::{Float, FromPrimitive};

use crate::algorithm::vincenty_distance::{FailedToConvergeError, VincentyDistance};
use crate::{Line, LineString, MultiLineString};

/// Determine the length of a geometry using [Vincenty’s formulae].
///
/// [Vincenty’s formulae]: https://en.wikipedia.org/wiki/Vincenty%27s_formulae
pub trait VincentyLength<T, RHS = Self> {
    /// Determine the length of a geometry using [Vincenty’s formulae].
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
    ///     // Osaka
    ///     (135.5244559, 34.687455)
    /// ]);
    ///
    /// let length = linestring.vincenty_length().unwrap();
    ///
    /// assert_eq!(
    ///     15_109_158., // meters
    ///     length.round()
    /// );
    /// ```
    ///
    /// [Vincenty’s formulae]: https://en.wikipedia.org/wiki/Vincenty%27s_formulae
    fn vincenty_length(&self) -> Result<T, FailedToConvergeError>;
}

impl<T> VincentyLength<T> for Line<T>
where
    T: Float + FromPrimitive,
{
    /// The units of the returned value is meters.
    fn vincenty_length(&self) -> Result<T, FailedToConvergeError> {
        let (start, end) = self.points();
        start.vincenty_distance(&end)
    }
}

impl<T> VincentyLength<T> for LineString<T>
where
    T: Float + FromPrimitive,
{
    fn vincenty_length(&self) -> Result<T, FailedToConvergeError> {
        let mut length = T::zero();
        for line in self.lines() {
            length = length + line.vincenty_length()?;
        }
        Ok(length)
    }
}

impl<T> VincentyLength<T> for MultiLineString<T>
where
    T: Float + FromPrimitive,
{
    fn vincenty_length(&self) -> Result<T, FailedToConvergeError> {
        let mut length = T::zero();
        for line_string in &self.0 {
            length = length + line_string.vincenty_length()?;
        }
        Ok(length)
    }
}
