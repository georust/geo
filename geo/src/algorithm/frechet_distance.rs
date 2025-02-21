use crate::line_measures::Euclidean;
use crate::{GeoFloat, LineString};
use num_traits::FromPrimitive;

#[deprecated(
    since = "0.30.0",
    note = "Please use the `Euclidean.frechet_distance` method from the `geo::line_measures::FrechetDistance` trait instead"
)]
/// Determine the similarity between two `LineStrings` using the [Frechet distance].
///
/// Based on [Computing Discrete Frechet Distance] by T. Eiter and H. Mannila.
///
/// [Frechet distance]: https://en.wikipedia.org/wiki/Fr%C3%A9chet_distance
/// [Computing Discrete Frechet Distance]: http://www.kr.tuwien.ac.at/staff/eiter/et-archive/cdtr9464.pdf
pub trait FrechetDistance<T, Rhs = Self> {
    /// Determine the similarity between two `LineStrings` using the [Frechet distance].
    ///
    /// # Examples
    ///
    /// ```
    /// use geo::FrechetDistance;
    /// use geo::line_string;
    ///
    /// let line_string_a = line_string![
    ///     (x: 1., y: 1.),
    ///     (x: 2., y: 1.),
    ///     (x: 2., y: 2.),
    ///     (x: 3., y: 3.)
    /// ];
    ///
    /// let line_string_b = line_string![
    ///     (x: 2., y: 2.),
    ///     (x: 0., y: 1.),
    ///     (x: 2., y: 4.),
    ///     (x: 3., y: 4.)
    /// ];
    ///
    /// let distance = line_string_a.frechet_distance(&line_string_b);
    ///
    /// assert_eq!(2., distance);
    /// ```
    ///
    /// [Frechet distance]: https://en.wikipedia.org/wiki/Fr%C3%A9chet_distance
    fn frechet_distance(&self, rhs: &Rhs) -> T;
}

#[allow(deprecated)]
impl<T> FrechetDistance<T, LineString<T>> for LineString<T>
where
    T: GeoFloat + FromPrimitive,
{
    fn frechet_distance(&self, ls: &LineString<T>) -> T {
        super::line_measures::FrechetDistance::frechet_distance(&Euclidean, self, ls)
    }
}
