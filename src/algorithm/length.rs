use num::{Float};

use types::{LineString};
use algorithm::distance::Distance;

/// Calculation of the length

pub trait Length<T, RHS = Self> {
    /// Calculation the length of a Line
    ///
    /// ```
    /// use geo::{Point, LineString, Coordinate};
    /// use geo::algorithm::length::Length;
    ///
    /// let mut vec = Vec::new();
    /// vec.push(Point::new(40.02f64, 116.34));
    /// vec.push(Point::new(42.02f64, 116.34));
    /// let linestring = LineString(vec);
    ///
    /// println!("Length {}", linestring.length());
    /// ```
    ///
    fn length(&self) -> T;
}

impl<T> Length<T> for LineString<T>
    where T: Float
{
    ///
    /// Length of a LineString.
    ///
    fn length(&self) -> T {
        let vect = &self.0;
        if vect.is_empty() || vect.len() == 1 {
            return T::zero();
        } else {
            let ipoints = vect.iter().zip(vect[1..].iter());
            ipoints.fold(T::zero(), |total_length, (p1, p2)| total_length + p1.distance(&p2))
        }
    }
}

#[cfg(test)]
mod test {
    use types::{Coordinate, Point, LineString};
    use algorithm::length::Length;

    #[test]
    fn empty_linestring_test() {
        let linestring = LineString::<f64>(Vec::new());
        assert_eq!(0.0_f64, linestring.length());
    }
    #[test]
    fn linestring_one_point_test() {
        let linestring = LineString(vec![Point::new(0., 0.)]);
        assert_eq!(0.0_f64, linestring.length());
    }
    #[test]
    fn linestring_test() {
        let p = |x| Point(Coordinate { x: x, y: 1. });
        let linestring = LineString(vec![p(1.), p(7.), p(8.), p(9.), p(10.), p(11.)]);
        assert_eq!(10.0_f64, linestring.length());
    }
}
