use geo_types::{CoordFloat, LineString, Point};

use crate::CoordsIter;

use super::Distance;

/// Determine the similarity between two `LineStrings` using the [Frechet distance].
///
/// Based on [Computing Discrete Frechet Distance] by T. Eiter and H. Mannila.
///
/// [Frechet distance]: https://en.wikipedia.org/wiki/Fr%C3%A9chet_distance
/// [Computing Discrete Frechet Distance]: http://www.kr.tuwien.ac.at/staff/eiter/et-archive/cdtr9464.pdf
pub trait FrechetDistance<F: CoordFloat>: Distance<F, Point<F>, Point<F>> {
    /// Returns the Fréchet distance between two LineStrings.
    ///
    /// See [specific implementations](#implementors) for details.
    ///
    /// # Examples
    ///
    /// ```
    /// use geo::line_measures::FrechetDistance;
    /// use geo::{Haversine, Euclidean, LineString, HaversineMeasure};
    /// use geo::line_string;
    ///
    /// let line_1 = line_string![
    ///     (x: 0., y: 0.),
    ///     (x: 1., y: 1.)
    /// ];
    /// let line_2 = line_string![
    ///     (x: 0., y: 1.),
    ///     (x: 1., y: 2.)
    /// ];
    ///
    /// // Using Euclidean distance
    /// let euclidean_distance = Euclidean.frechet_distance(&line_1, &line_2);
    ///
    /// // Using Haversine distance for geographic coordinates
    /// let haversine_distance = Haversine.frechet_distance(&line_1, &line_2);
    ///
    /// // Using parameterized Haversine for different planetary bodies
    /// let mars_measure = HaversineMeasure::new(3389.5); // Mars radius in km
    /// let mars_distance = mars_measure.frechet_distance(&line_1, &line_2);
    /// ```
    ///
    /// [Frechet distance]: https://en.wikipedia.org/wiki/Fr%C3%A9chet_distance
    fn frechet_distance(&self, ls_1: &LineString<F>, ls_2: &LineString<F>) -> F;
}

impl<F, MetricSpace> FrechetDistance<F> for MetricSpace
where
    F: CoordFloat,
    MetricSpace: Distance<F, Point<F>, Point<F>>,
{
    fn frechet_distance(&self, ls_1: &LineString<F>, ls_2: &LineString<F>) -> F {
        let (n1, n2) = (ls_1.coords_count(), ls_2.coords_count());
        if n1 == 0 || n2 == 0 {
            return F::zero();
        }

        let (ls_short, ls_long) = if n1 <= n2 { (ls_1, ls_2) } else { (ls_2, ls_1) };

        DiscreteFrechetCalculator::new(ls_short, ls_long).calculate(self)
    }
}

/// Helper struct for the dynamic programming calculation of discrete Fréchet distance.
/// It uses a buffer representing two rows of the DP table to achieve O(min(m,n)) space.
struct DiscreteFrechetCalculator<'a, F: CoordFloat> {
    cache: Vec<F>,
    ls_short: &'a LineString<F>,
    ls_long: &'a LineString<F>,
}

impl<'a, F: CoordFloat> DiscreteFrechetCalculator<'a, F> {
    fn new(ls_short: &'a LineString<F>, ls_long: &'a LineString<F>) -> Self {
        Self {
            cache: vec![F::zero(); ls_short.coords_count() * 2],
            ls_short,
            ls_long,
        }
    }

    fn calculate(&mut self, metric_space: &impl Distance<F, Point<F>, Point<F>>) -> F {
        let row_length = self.ls_short.coords_count();
        let (mut prev_row, mut cur_row) = self.cache.split_at_mut(row_length);

        let p_long_0: Point<F> = self.ls_long[0].into();
        let p_short_0: Point<F> = self.ls_short[0].into();

        prev_row[0] = metric_space.distance(p_long_0, p_short_0);
        for (j, p_short) in self.ls_short.points().enumerate().skip(1) {
            let d = metric_space.distance(p_long_0, p_short);
            prev_row[j] = prev_row[j - 1].max(d);
        }

        for p_long in self.ls_long.points().skip(1) {
            let d = metric_space.distance(p_long, p_short_0);
            cur_row[0] = prev_row[0].max(d);

            for (j, p_short) in self.ls_short.points().enumerate().skip(1) {
                let d = metric_space.distance(p_long, p_short);
                cur_row[j] = {
                    let best_prev = prev_row[j].min(prev_row[j - 1]).min(cur_row[j - 1]);
                    d.max(best_prev)
                };
            }
            std::mem::swap(&mut prev_row, &mut cur_row);
        }

        prev_row[row_length - 1]
    }
}

#[cfg(test)]
mod test {
    use crate::{Euclidean, HaversineMeasure};

    use super::*;

    #[test]
    fn test_single_point_in_linestring_euclidean() {
        let ls_a = LineString::from(vec![(1., 1.)]);
        let ls_b = LineString::from(vec![(0., 2.)]);
        assert_relative_eq!(
            Euclidean.distance(Point::from(ls_a.0[0]), Point::from(ls_b.0[0])),
            Euclidean.frechet_distance(&ls_a, &ls_b)
        );
    }

    #[test]
    fn test_identical_linestrings_euclidean() {
        let ls_a = LineString::from(vec![(1., 1.), (2., 1.), (2., 2.)]);
        let ls_b = LineString::from(vec![(1., 1.), (2., 1.), (2., 2.)]);
        assert_relative_eq!(0., Euclidean.frechet_distance(&ls_a, &ls_b));
    }

    #[test]
    fn different_dimensions_linestrings_euclidean() {
        let ls_a = LineString::from(vec![(1., 1.)]);
        let ls_b = LineString::from(vec![(2., 2.), (0., 1.)]);
        assert_relative_eq!(2f64.sqrt(), Euclidean.frechet_distance(&ls_a, &ls_b));
    }

    #[test]
    fn test_frechet_1_euclidean() {
        let ls_a = LineString::from(vec![(1., 1.), (2., 1.)]);
        let ls_b = LineString::from(vec![(2., 2.), (2., 3.)]);
        assert_relative_eq!(2., Euclidean.frechet_distance(&ls_a, &ls_b));
    }

    #[test]
    fn test_frechet_2_euclidean() {
        let ls_a = LineString::from(vec![(1., 1.), (2., 1.), (2., 2.)]);
        let ls_b = LineString::from(vec![(2., 2.), (0., 1.), (2., 4.)]);
        assert_relative_eq!(2., Euclidean.frechet_distance(&ls_a, &ls_b));
    }

    #[test] // comparing long linestrings should not panic or abort due to stack overflow
    fn test_frechet_long_linestrings_euclidean() {
        let ls: LineString = {
            let delta = 0.01;

            let mut ls = vec![(0.0, 0.0); 1_000];
            for i in 1..ls.len() {
                let (lat, lon) = ls[i - 1];
                ls[i] = (lat - delta, lon + delta);
            }

            ls.into()
        };

        assert_relative_eq!(Euclidean.frechet_distance(&ls, &ls), 0.0);
    }

    #[test]
    fn test_single_point_in_linestring_haversine_custom() {
        let mars_measure = HaversineMeasure::new(3389.5); // Mars radius in km
        let ls_a = LineString::from(vec![(1., 1.)]);
        let ls_b = LineString::from(vec![(0., 2.)]);
        assert_relative_eq!(
            mars_measure.distance(Point::from(ls_a.0[0]), Point::from(ls_b.0[0])),
            mars_measure.frechet_distance(&ls_a, &ls_b)
        );
    }

    #[test]
    fn test_identical_linestrings_haversine_custom() {
        let mars_measure = HaversineMeasure::new(3389.5);
        let ls_a = LineString::from(vec![(1., 1.), (2., 1.), (2., 2.)]);
        let ls_b = LineString::from(vec![(1., 1.), (2., 1.), (2., 2.)]);
        assert_relative_eq!(0., mars_measure.frechet_distance(&ls_a, &ls_b));
    }

    #[test]
    fn different_dimensions_linestrings_haversine_custom() {
        let mars_measure = HaversineMeasure::new(3389.5);
        let ls_a = LineString::from(vec![(1., 1.)]);
        let ls_b = LineString::from(vec![(2., 2.), (0., 1.)]);
        let expected_distance = mars_measure.distance(Point::new(1., 1.), Point::new(2., 2.));
        assert_relative_eq!(
            expected_distance,
            mars_measure.frechet_distance(&ls_a, &ls_b)
        );
    }

    #[test]
    fn test_frechet_long_linestrings_haversine_custom() {
        let mars_measure = HaversineMeasure::new(3389.5);
        let ls: LineString = {
            let delta = 0.01;

            let mut ls = vec![(0.0, 0.0); 1_000];
            for i in 1..ls.len() {
                let (lat, lon) = ls[i - 1];
                ls[i] = (lat - delta, lon + delta);
            }

            ls.into()
        };

        assert_relative_eq!(mars_measure.frechet_distance(&ls, &ls), 0.0);
    }
}
