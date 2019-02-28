use crate::euclidean_distance::EuclideanDistance;
use crate::{LineString, Point};
use num_traits::{Float, FromPrimitive};

/// Determine the similarity between two `LineStrings` using the [frechet distance].
///
/// [frechet distance]: https://en.wikipedia.org/wiki/Fr%C3%A9chet_distance
pub trait FrechetDistance<T, Rhs = Self> {
    /// Determine the similarity between two `LineStrings` using the [frechet distance].
    ///
    /// # Units
    ///
    /// - return value: float
    ///
    /// # Examples
    ///
    /// ```
    /// use geo::LineString;
    /// use geo::algorithm::frechet_distance::FrechetDistance;
    ///
    /// let ls_a = LineString::from(vec![(1., 1.), (2., 1.), (2., 2.), (3., 3.)]);
    /// let ls_b = LineString::from(vec![(2., 2.), (0., 1.), (2., 4.), (3., 4.)]);
    ///
    /// let distance = ls_a.frechet_distance(&ls_b);
    ///
    /// assert_eq!(2., distance);
    /// ```
    ///
    /// [frechet distance]: https://en.wikipedia.org/wiki/Fr%C3%A9chet_distance
    fn frechet_distance(&self, rhs: &Rhs) -> T;
}

impl<T> FrechetDistance<T, LineString<T>> for LineString<T>
where
    T: Float + FromPrimitive,
{
    fn frechet_distance(&self, ls: &LineString<T>) -> T {
        let points_a: Vec<Point<T>> = self.clone().into_points();
        let points_b: Vec<Point<T>> = ls.clone().into_points();
        if points_a.len() != 0 && points_b.len() != 0 {
            let mut data = Data {
                cache: vec![vec![T::nan(); points_b.len()]; points_a.len()],
                points_a,
                points_b,
            };
            data.c(data.points_a.len() - 1, data.points_b.len() - 1)
        } else {
            T::zero()
        }
    }
}

struct Data<T>
where
    T: Float + FromPrimitive,
{
    cache: Vec<Vec<T>>,
    points_a: Vec<Point<T>>,
    points_b: Vec<Point<T>>,
}

impl<T> Data<T>
where
    T: Float + FromPrimitive,
{
    fn c(&mut self, i: usize, j: usize) -> T {
        if self.cache[i][j].is_nan() {
            let eucl = self.points_a[i].euclidean_distance(&self.points_b[j]);
            self.cache[i][j] = match (i, j) {
                (0, 0) => eucl,
                (_, 0) => self.c(i - 1, 0).max(eucl),
                (0, _) => self.c(0, j - 1).max(eucl),
                (_, _) => {
                    ((self.c(i - 1, j).min(self.c(i - 1, j - 1))).min(self.c(i, j - 1))).max(eucl)
                }
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
        assert_eq!(
            (ls_a.clone().into_points())[0].euclidean_distance(&(&ls_b.clone().into_points())[0]),
            ls_a.frechet_distance(&ls_b)
        );
    }

    #[test]
    fn test_identical_linestrings() {
        let ls_a = LineString::from(vec![(1., 1.), (2., 1.), (2., 2.)]);
        let ls_b = LineString::from(vec![(1., 1.), (2., 1.), (2., 2.)]);
        assert_eq!(0., ls_a.frechet_distance(&ls_b));
    }

    #[test]
    fn different_dimensions_linestrings() {
        let ls_a = LineString::from(vec![(1., 1.)]);
        let ls_b = LineString::from(vec![(2., 2.), (0., 1.)]);
        assert_eq!(2f64.sqrt(), ls_a.frechet_distance(&ls_b));
    }

    #[test]
    fn test_frechet_1() {
        let ls_a = LineString::from(vec![(1., 1.), (2., 1.)]);
        let ls_b = LineString::from(vec![(2., 2.), (2., 3.)]);
        assert_eq!(2., ls_a.frechet_distance(&ls_b));
    }

    #[test]
    fn test_frechet_2() {
        let ls_a = LineString::from(vec![(1., 1.), (2., 1.), (2., 2.)]);
        let ls_b = LineString::from(vec![(2., 2.), (0., 1.), (2., 4.)]);
        assert_eq!(2., ls_a.frechet_distance(&ls_b));
    }
}
