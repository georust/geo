use crate::{
    algorithm::{euclidean_distance::EuclideanDistance, euclidean_length::EuclideanLength},
    CoordinateType, Line, LineString, Point,
};
use num_traits::{
    identities::{One, Zero},
    Float,
};
use std::ops::AddAssign;

/// Returns a fraction of the line's total length
/// representing the location
/// of the closest point on the line to the given point.
///
/// If the line has zero length the fraction returned is zero.
///
/// # Examples
///
/// ```
/// use geo::{LineString, point};
/// use geo::algorithm::line_locate_point::LineLocatePoint;
///
/// let linestring: LineString<f64> = vec![
///     [-1.0, 0.0],
///     [0.0, 0.0],
///     [0.0, 1.0]
/// ].into();
///
/// assert_eq!(linestring.line_locate_point(&point!(x: -1.0, y: 0.0)), 0.0);
/// assert_eq!(linestring.line_locate_point(&point!(x: -0.5, y: 0.0)), 0.25);
/// assert_eq!(linestring.line_locate_point(&point!(x: 0.0, y: 0.0)), 0.5);
/// assert_eq!(linestring.line_locate_point(&point!(x: 0.0, y: 0.5)), 0.75);
/// assert_eq!(linestring.line_locate_point(&point!(x: 0.0, y: 1.0)), 1.0);
/// ```
pub trait LineLocatePoint<T, Rhs> {
    type Output;
    type Rhs;

    fn line_locate_point(&self, p: &Rhs) -> Self::Output;
}

impl<T> LineLocatePoint<T, Point<T>> for Line<T>
where
    T: CoordinateType + Float + Zero + One,
{
    type Output = T;
    type Rhs = Point<T>;

    fn line_locate_point(&self, p: &Self::Rhs) -> Self::Output {
        let sp = [p.x() - self.start.x, p.y() - self.start.y];
        let v = [self.end.x - self.start.x, self.end.y - self.start.y];
        let v_sq = v[0] * v[0] + v[1] * v[1];
        if v_sq == T::zero() {
            // The line has zero length, return zero
            return T::zero();
        } else {
            let v_dot_sp = v[0] * sp[0] + v[1] * sp[1];
            let l = v_dot_sp / v_sq;
            if v_dot_sp.is_nan() | v_sq.is_nan() {
                return T::nan();
            } else {
                return l.max(T::zero()).min(T::one());
            }
        }
    }
}

impl<T> LineLocatePoint<T, Point<T>> for LineString<T>
where
    T: CoordinateType + Float + Zero + One + PartialOrd + AddAssign,
    Line<T>: EuclideanDistance<T, Point<T>> + EuclideanLength<T>,
{
    type Output = T;
    type Rhs = Point<T>;

    fn line_locate_point(&self, p: &Self::Rhs) -> Self::Output {
        let mut total_length = T::zero();
        let mut queue = Vec::new();
        for line in self.lines() {
            let length = line.euclidean_length();
            let distance_to_point = line.euclidean_distance(p);
            queue.push((
                total_length.clone(),
                length.clone(),
                distance_to_point.clone(),
                line.clone(),
            ));
            total_length += length;
        }
        if total_length == T::zero() {
            // linestring has zero legnth, return zero
            return T::zero();
        } else {
            let l = queue
                .iter()
                .min_by(|x, y| (x.2).partial_cmp(&y.2).unwrap())
                .unwrap();

            return (l.0 + l.1 * (l.3).line_locate_point(p)) / total_length
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::{point, Coordinate};

    #[test]
    fn test_line_locate_point_line() {
        let line = Line::new(
            Coordinate { x: -1.0, y: 0.0 },
            Coordinate { x: 1.0, y: 0.0 },
        );
        let point = Point::new(0.0, 1.0);
        assert_eq!(line.line_locate_point(&point), 0.5);

        let point = Point::new(1.0, 1.0);
        assert_eq!(line.line_locate_point(&point), 1.0);

        let point = Point::new(2.0, 1.0);
        assert_eq!(line.line_locate_point(&point), 1.0);

        let point = Point::new(-1.0, 1.0);
        assert_eq!(line.line_locate_point(&point), 0.0);

        let point = Point::new(-2.0, 1.0);
        assert_eq!(line.line_locate_point(&point), 0.0);

        let line = Line::new(
            Coordinate { x: 0.0, y: 0.0 },
            Coordinate {
                x: Float::infinity(),
                y: 0.0,
            },
        );
        let point = Point::new(1000.0, 1000.0);
        assert_eq!(line.line_locate_point(&point), 0.0);

        let line = Line::new(
            Coordinate { x: 0.0, y: 0.0 },
            Coordinate {
                x: Float::nan(),
                y: 0.0,
            },
        );
        let point = Point::new(1000.0, 1000.0);
        assert!(line.line_locate_point(&point).is_nan());

        let line: Line<f64> = Line::new(Coordinate { x: 1.0, y: 1.0},
                                        Coordinate { x: 1.0, y: 1.0});
        let pt = point!(x: 2.0, y: 2.0);
        assert_eq!(line.line_locate_point(&pt), 0.0);
    }

    #[test]
    fn test_line_locate_point_linestring() {
        let ring: LineString<f64> = include!("./test_fixtures/ring.rs").into();
        let pt = point!(x: 10.0, y: 1.0);
        assert_eq!(ring.line_locate_point(&pt), 0.0);

        let pt = point!(x: 10.0, y: 1.0000000000000742);
        assert_eq!(ring.line_locate_point(&pt), 0.9999999999999988);

        let pt = point!(x: 10.0, y: 1.0);
        assert_eq!(ring.line_locate_point(&pt), 0.0);

        let line: LineString<f64> = LineString(vec![(1.0, 1.0).into(),
                                                    (1.0, 1.0).into(),
                                                    (1.0, 1.0).into()]);
        let pt = point!(x: 2.0, y: 2.0);
        assert_eq!(line.line_locate_point(&pt), 0.0);

        let line: LineString<f64> = LineString(vec![Coordinate { x: 1.0, y: 1.0 },
                                                    Coordinate { x: f64::NAN, y: 1.0},
                                                    Coordinate { x: 0.0, y:0.0 }]);
        let pt = point!(x: 2.0, y: 2.0);
        assert!(line.line_locate_point(&pt).is_nan())
    }
}
