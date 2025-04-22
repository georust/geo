use std::iter::Sum;

use crate::{CoordFloat, Euclidean, Length, Line, LineString, MultiLineString};

/// Calculation of the length
#[deprecated(
    since = "0.29.0",
    note = "Please use the `Euclidean.length(&line)` via the `Length` trait instead."
)]
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

#[allow(deprecated)]
impl<T> EuclideanLength<T> for Line<T>
where
    T: CoordFloat,
{
    fn euclidean_length(&self) -> T {
        Euclidean.length(self)
    }
}

#[allow(deprecated)]
impl<T> EuclideanLength<T> for LineString<T>
where
    T: CoordFloat + Sum,
{
    fn euclidean_length(&self) -> T {
        Euclidean.length(self)
    }
}

#[allow(deprecated)]
impl<T> EuclideanLength<T> for MultiLineString<T>
where
    T: CoordFloat + Sum,
{
    fn euclidean_length(&self) -> T {
        Euclidean.length(self)
    }
}

#[cfg(test)]
mod test {
    use crate::line_string;
    #[allow(deprecated)]
    use crate::EuclideanLength;
    use crate::{coord, Line, MultiLineString};

    #[allow(deprecated)]
    #[test]
    fn empty_linestring_test() {
        let linestring = line_string![];
        assert_relative_eq!(0.0_f64, linestring.euclidean_length());
    }
    #[allow(deprecated)]
    #[test]
    fn linestring_one_point_test() {
        let linestring = line_string![(x: 0., y: 0.)];
        assert_relative_eq!(0.0_f64, linestring.euclidean_length());
    }
    #[allow(deprecated)]
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
    #[allow(deprecated)]
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
    #[allow(deprecated)]
    #[test]
    fn line_test() {
        let line0 = Line::new(coord! { x: 0., y: 0. }, coord! { x: 0., y: 1. });
        let line1 = Line::new(coord! { x: 0., y: 0. }, coord! { x: 3., y: 4. });
        assert_relative_eq!(line0.euclidean_length(), 1.);
        assert_relative_eq!(line1.euclidean_length(), 5.);
    }
}
