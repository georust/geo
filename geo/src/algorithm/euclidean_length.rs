use num_traits::Float;

use ::{Line, LineString, MultiLineString};

/// Calculation of the length

pub trait EuclideanLength<T, RHS = Self> {
    /// Calculation of the length of a Line
    ///
    /// # Examples
    ///
    /// ```
    /// use geo::{Point, LineString, Coordinate};
    /// use geo::algorithm::euclidean_length::EuclideanLength;
    ///
    /// let mut vec = Vec::new();
    /// vec.push(Point::new(40.02f64, 116.34));
    /// vec.push(Point::new(42.02f64, 116.34));
    /// let linestring = LineString(vec);
    ///
    /// println!("EuclideanLength {}", linestring.euclidean_length());
    /// ```
    ///
    fn euclidean_length(&self) -> T;
}

impl<T> EuclideanLength<T> for Line<T>
where
    T: Float,
{
    fn euclidean_length(&self) -> T {
        self.dx().hypot(self.dy())
    }
}

impl<T> EuclideanLength<T> for LineString<T>
where
    T: Float,
{
    fn euclidean_length(&self) -> T {
        self.lines()
            .map(|line| line.euclidean_length())
            .fold(T::zero(), |total_length, length| total_length + length)
    }
}

impl<T> EuclideanLength<T> for MultiLineString<T>
where
    T: Float,
{
    fn euclidean_length(&self) -> T {
        self.0
            .iter()
            .fold(T::zero(), |total, line| {
                total + line.euclidean_length()
            })
    }
}

#[cfg(test)]
mod test {
    use ::{Coordinate, Line, LineString, MultiLineString, Point};
    use algorithm::euclidean_length::EuclideanLength;

    #[test]
    fn empty_linestring_test() {
        let linestring = LineString::<f64>(Vec::new());
        assert_eq!(0.0_f64, linestring.euclidean_length());
    }
    #[test]
    fn linestring_one_point_test() {
        let linestring = LineString(vec![Point::new(0., 0.)]);
        assert_eq!(0.0_f64, linestring.euclidean_length());
    }
    #[test]
    fn linestring_test() {
        let p = |x| Point(Coordinate { x: x, y: 1. });
        let linestring = LineString(vec![p(1.), p(7.), p(8.), p(9.), p(10.), p(11.)]);
        assert_eq!(10.0_f64, linestring.euclidean_length());
    }
    #[test]
    fn multilinestring_test() {
        let p = |x, y| Point(Coordinate { x: x, y: y });
        let mline = MultiLineString(vec![
            LineString(vec![
                p(1., 0.),
                p(7., 0.),
                p(8., 0.),
                p(9., 0.),
                p(10., 0.),
                p(11., 0.),
            ]),
            LineString(vec![p(0., 0.), p(0., 5.)]),
        ]);
        assert_eq!(15.0_f64, mline.euclidean_length());
    }
    #[test]
    fn line_test() {
        let line0 = Line::new(Point::new(0., 0.), Point::new(0., 1.));
        let line1 = Line::new(Point::new(0., 0.), Point::new(3., 4.));
        assert_eq!(line0.euclidean_length(), 1.);
        assert_eq!(line1.euclidean_length(), 5.);
    }
}
