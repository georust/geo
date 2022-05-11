use std::iter::Sum;

use crate::{CoordFloat, Line, LineString, MultiLineString};

/// Calculation of the length

pub trait EuclideanLength<T, RHS = Self> {
    /// Calculation of the length of a Line
    ///
    /// # Examples
    ///
    /// ```
    /// use geo::EuclideanLength;
    /// use geo::line_string;
    ///
    /// let line_string = line_string![
    ///     (x: 40.02f64, y: 116.34),
    ///     (x: 42.02f64, y: 116.34),
    /// ];
    ///
    /// assert_eq!(
    ///     2.,
    ///     line_string.euclidean_length(),
    /// )
    /// ```
    fn euclidean_length(&self) -> T;
}

impl<T> EuclideanLength<T> for Line<T>
where
    T: CoordFloat,
{
    fn euclidean_length(&self) -> T {
        ::geo_types::private_utils::line_euclidean_length(*self)
    }
}

impl<T> EuclideanLength<T> for LineString<T>
where
    T: CoordFloat + Sum,
{
    fn euclidean_length(&self) -> T {
        self.lines().map(|line| line.euclidean_length()).sum()
    }
}

impl<T> EuclideanLength<T> for MultiLineString<T>
where
    T: CoordFloat + Sum,
{
    fn euclidean_length(&self) -> T {
        self.0
            .iter()
            .fold(T::zero(), |total, line| total + line.euclidean_length())
    }
}

#[cfg(test)]
mod test {
    use crate::line_string;
    use crate::EuclideanLength;
    use crate::{coord, Line, MultiLineString};

    #[test]
    fn empty_linestring_test() {
        let linestring = line_string![];
        assert_relative_eq!(0.0_f64, linestring.euclidean_length());
    }
    #[test]
    fn linestring_one_point_test() {
        let linestring = line_string![(x: 0., y: 0.)];
        assert_relative_eq!(0.0_f64, linestring.euclidean_length());
    }
    #[test]
    fn linestring_test() {
        let linestring = line_string![
            (x: 1., y: 1.),
            (x: 7., y: 1.),
            (x: 8., y: 1.),
            (x: 9., y: 1.),
            (x: 10., y: 1.),
            (x: 11., y: 1.)
        ];
        assert_relative_eq!(10.0_f64, linestring.euclidean_length());
    }
    #[test]
    fn multilinestring_test() {
        let mline = MultiLineString::new(vec![
            line_string![
                (x: 1., y: 0.),
                (x: 7., y: 0.),
                (x: 8., y: 0.),
                (x: 9., y: 0.),
                (x: 10., y: 0.),
                (x: 11., y: 0.)
            ],
            line_string![
                (x: 0., y: 0.),
                (x: 0., y: 5.)
            ],
        ]);
        assert_relative_eq!(15.0_f64, mline.euclidean_length());
    }
    #[test]
    fn line_test() {
        let line0 = Line::new(coord! { x: 0., y: 0. }, coord! { x: 0., y: 1. });
        let line1 = Line::new(coord! { x: 0., y: 0. }, coord! { x: 3., y: 4. });
        assert_relative_eq!(line0.euclidean_length(), 1.);
        assert_relative_eq!(line1.euclidean_length(), 5.);
    }
}
