use crate::euclidean_distance::EuclideanDistance;
use crate::{LineString, Point};
use num_traits::{Float, FromPrimitive};

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
    /// use geo::algorithm::frechet_distance::FrechetDistance;
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

impl<T> FrechetDistance<T, LineString<T>> for LineString<T>
where
    T: Float + FromPrimitive,
{
    fn frechet_distance(&self, ls: &LineString<T>) -> T {
        if self.num_coords() != 0 && ls.num_coords() != 0 {
            let mut data = Data {
                cache: vec![vec![T::nan(); ls.num_coords()]; self.num_coords()],
                ls_a: self,
                ls_b: ls,
            };
            data.compute(self.num_coords() - 1, ls.num_coords() - 1)
        } else {
            T::zero()
        }
    }
}

struct Data<'a, T>
where
    T: Float + FromPrimitive,
{
    cache: Vec<Vec<T>>,
    ls_a: &'a LineString<T>,
    ls_b: &'a LineString<T>,
}

impl<'a, T> Data<'a, T>
where
    T: Float + FromPrimitive,
{
    fn compute(&mut self, i: usize, j: usize) -> T {
        if self.cache[i][j].is_nan() {
            let eucl = Point::from(self.ls_a[i]).euclidean_distance(&Point::from(self.ls_b[j]));
            self.cache[i][j] = match (i, j) {
                (0, 0) => eucl,
                (_, 0) => self.compute(i - 1, 0).max(eucl),
                (0, _) => self.compute(0, j - 1).max(eucl),
                (_, _) => ((self.compute(i - 1, j).min(self.compute(i - 1, j - 1)))
                    .min(self.compute(i, j - 1)))
                .max(eucl),
            };
        }
        self.cache[i][j]
    }
}

#[cfg(test)]
mod test {
    use crate::algorithm::frechet_distance::FrechetDistance;
    use crate::euclidean_distance::EuclideanDistance;
    use crate::LineString;

    #[test]
    fn test_single_point_in_linestring() {
        let ls_a = LineString::from(vec![(1., 1.)]);
        let ls_b = LineString::from(vec![(0., 2.)]);
        assert_relative_eq!(
            (ls_a.clone().into_points())[0].euclidean_distance(&(&ls_b.clone().into_points())[0]),
            ls_a.frechet_distance(&ls_b)
        );
    }

    #[test]
    fn test_identical_linestrings() {
        let ls_a = LineString::from(vec![(1., 1.), (2., 1.), (2., 2.)]);
        let ls_b = LineString::from(vec![(1., 1.), (2., 1.), (2., 2.)]);
        assert_relative_eq!(0., ls_a.frechet_distance(&ls_b));
    }

    #[test]
    fn different_dimensions_linestrings() {
        let ls_a = LineString::from(vec![(1., 1.)]);
        let ls_b = LineString::from(vec![(2., 2.), (0., 1.)]);
        assert_relative_eq!(2f64.sqrt(), ls_a.frechet_distance(&ls_b));
    }

    #[test]
    fn test_frechet_1() {
        let ls_a = LineString::from(vec![(1., 1.), (2., 1.)]);
        let ls_b = LineString::from(vec![(2., 2.), (2., 3.)]);
        assert_relative_eq!(2., ls_a.frechet_distance(&ls_b));
    }

    #[test]
    fn test_frechet_2() {
        let ls_a = LineString::from(vec![(1., 1.), (2., 1.), (2., 2.)]);
        let ls_b = LineString::from(vec![(2., 2.), (0., 1.), (2., 4.)]);
        assert_relative_eq!(2., ls_a.frechet_distance(&ls_b));
    }
}
