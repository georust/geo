use crate::coords_iter::CoordsIter;
use geo_types::{CoordFloat, LineString, Point};

use super::Distance;

/// Determine the similarity between two `LineStrings` using the [Frechet distance].
///
/// Based on [Computing Discrete Frechet Distance] by T. Eiter and H. Mannila.
///
/// [Frechet distance]: https://en.wikipedia.org/wiki/Fr%C3%A9chet_distance
/// [Computing Discrete Frechet Distance]: http://www.kr.tuwien.ac.at/staff/eiter/et-archive/cdtr9464.pdf
pub trait FrechetDistance<F: CoordFloat, Rhs = Self> {
    /// Determine the similarity between two `LineStrings` using the [Frechet distance].
    ///
    /// # Arguments
    ///
    /// * `rhs` - A reference to another `LineString` to compare with `self`.
    ///
    /// # Returns
    ///
    /// The Fréchet distance as a floating-point number of type `F`.
    ///
    /// # Examples
    ///
    /// ```
    /// use geo::algorithm::line_measures::{FrechetDistance, Euclidean};
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
    /// let distance = line_string_a.frechet_distance::<Euclidean>(&line_string_b);
    ///
    /// assert_eq!(2., distance);
    /// ```
    ///
    /// [Frechet distance]: https://en.wikipedia.org/wiki/Fr%C3%A9chet_distance
    fn frechet_distance<MetricSpace: Distance<F, Point<F>, Point<F>>>(&self, rhs: &Rhs) -> F;
}

impl<F: CoordFloat> FrechetDistance<F> for LineString<F> {
    fn frechet_distance<MetricSpace: Distance<F, Point<F>, Point<F>>>(&self, other: &Self) -> F {
        if self.coords_count() != 0 && other.coords_count() != 0 {
            Data {
                cache: vec![F::zero(); self.coords_count() * other.coords_count()],
                ls_a: self,
                ls_b: other,
            }
            .compute_linear::<MetricSpace>()
        } else {
            F::zero()
        }
    }
}

struct Data<'a, F: CoordFloat> {
    cache: Vec<F>,
    ls_a: &'a LineString<F>,
    ls_b: &'a LineString<F>,
}

impl<'a, F: CoordFloat> Data<'a, F> {
    /// [Reference implementation]: https://github.com/joaofig/discrete-frechet/tree/master
    fn compute_linear<MetricSpace: Distance<F, Point<F>, Point<F>>>(&mut self) -> F {
        let columns_count = self.ls_b.coords_count();

        for (i, a) in self.ls_a.points().enumerate() {
            for (j, b) in self.ls_b.points().enumerate() {
                let dist = MetricSpace.distance(a, b);

                self.cache[i * columns_count + j] = match (i, j) {
                    (0, 0) => dist,
                    (_, 0) => self.cache[(i - 1) * columns_count].max(dist),
                    (0, _) => self.cache[j - 1].max(dist),
                    (_, _) => self.cache[(i - 1) * columns_count + j]
                        .min(self.cache[(i - 1) * columns_count + j - 1])
                        .min(self.cache[i * columns_count + j - 1])
                        .max(dist),
                };
            }
        }

        self.cache[self.cache.len() - 1]
    }
}

#[cfg(test)]
mod test {
    use crate::Euclidean;

    use super::*;

    #[test]
    fn test_single_point_in_linestring_euclidean() {
        let ls_a = LineString::from(vec![(1., 1.)]);
        let ls_b = LineString::from(vec![(0., 2.)]);
        assert_relative_eq!(
            Euclidean.distance(Point::from(ls_a.0[0]), Point::from(ls_b.0[0])),
            ls_a.frechet_distance::<Euclidean>(&ls_b)
        );
    }

    #[test]
    fn test_identical_linestrings_euclidean() {
        let ls_a = LineString::from(vec![(1., 1.), (2., 1.), (2., 2.)]);
        let ls_b = LineString::from(vec![(1., 1.), (2., 1.), (2., 2.)]);
        assert_relative_eq!(0., ls_a.frechet_distance::<Euclidean>(&ls_b));
    }

    #[test]
    fn different_dimensions_linestrings_euclidean() {
        let ls_a = LineString::from(vec![(1., 1.)]);
        let ls_b = LineString::from(vec![(2., 2.), (0., 1.)]);
        assert_relative_eq!(2f64.sqrt(), ls_a.frechet_distance::<Euclidean>(&ls_b));
    }

    #[test]
    fn test_frechet_1_euclidean() {
        let ls_a = LineString::from(vec![(1., 1.), (2., 1.)]);
        let ls_b = LineString::from(vec![(2., 2.), (2., 3.)]);
        assert_relative_eq!(2., ls_a.frechet_distance::<Euclidean>(&ls_b));
    }

    #[test]
    fn test_frechet_2_euclidean() {
        let ls_a = LineString::from(vec![(1., 1.), (2., 1.), (2., 2.)]);
        let ls_b = LineString::from(vec![(2., 2.), (0., 1.), (2., 4.)]);
        assert_relative_eq!(2., ls_a.frechet_distance::<Euclidean>(&ls_b));
    }

    #[test] // comparing long linestrings should not panic or abort due to stack overflow
    fn test_frechet_long_linestrings_euclidean() {
        let ls: LineString = {
            let delta = 0.01;

            let mut ls = vec![(0.0, 0.0); 10_000];
            for i in 1..ls.len() {
                let (lat, lon) = ls[i - 1];
                ls[i] = (lat - delta, lon + delta);
            }

            ls.into()
        };

        assert_relative_eq!(ls.frechet_distance::<Euclidean>(&ls.clone()), 0.0);
    }
}
