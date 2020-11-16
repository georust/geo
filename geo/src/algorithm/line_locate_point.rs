use crate::{
    algorithm::{euclidean_distance::EuclideanDistance, euclidean_length::EuclideanLength},
    CoordinateType, Line, LineString, Point,
};
use num_traits::Float;
use std::ops::AddAssign;

/// Returns a (option of the) fraction of the line's total length
/// representing the location of the closest point on the line to
/// the given point.
///
/// If the line has zero length the fraction returned is zero.
///
/// If either the point's coordinates or any coordinates of the line
/// are not finite, returns `None`.
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
/// assert_eq!(linestring.line_locate_point(&point!(x: -1.0, y: 0.0)), Some(0.0));
/// assert_eq!(linestring.line_locate_point(&point!(x: -0.5, y: 0.0)), Some(0.25));
/// assert_eq!(linestring.line_locate_point(&point!(x: 0.0, y: 0.0)), Some(0.5));
/// assert_eq!(linestring.line_locate_point(&point!(x: 0.0, y: 0.5)), Some(0.75));
/// assert_eq!(linestring.line_locate_point(&point!(x: 0.0, y: 1.0)), Some(1.0));
/// ```
pub trait LineLocatePoint<T, Rhs> {
    type Output;
    type Rhs;

    fn line_locate_point(&self, p: &Rhs) -> Self::Output;
}

impl<T> LineLocatePoint<T, Point<T>> for Line<T>
where
    T: CoordinateType + Float,
{
    type Output = Option<T>;
    type Rhs = Point<T>;

    fn line_locate_point(&self, p: &Self::Rhs) -> Self::Output {
        // let $s$ be the starting point of the line, and $v$ its
        // direction vector. We want to find $l$ such that
        // $(p - (s + lv)) \cdot v = 0$, i.e. the vector from
        // $l$ along the line to $p$ is perpendicular to $v$.a

        // vector $p - s$
        let sp: Point<_> = *p - self.start_point();

        // direction vector of line, $v$
        let v: Point<_> = (self.end - self.start).into();

        // $v \cdot v$
        let v_sq = v.dot(v);
        if v_sq == T::zero() {
            // The line has zero length, return zero
            return Some(T::zero());
        } else {
            // $v \cdot (p - s)$
            let v_dot_sp = v.dot(sp);
            let l = v_dot_sp / v_sq;
            if l.is_finite() {
                return Some(l.max(T::zero()).min(T::one()));
            } else {
                return None;
            }
        }
    }
}

impl<T> LineLocatePoint<T, Point<T>> for LineString<T>
where
    T: CoordinateType + Float + AddAssign,
    Line<T>: EuclideanDistance<T, Point<T>> + EuclideanLength<T>,
    LineString<T>: EuclideanLength<T>,
{
    type Output = Option<T>;
    type Rhs = Point<T>;

    fn line_locate_point(&self, p: &Self::Rhs) -> Self::Output {
        let total_length = (*self).euclidean_length();
        if total_length == T::zero() {
            return Some(T::zero());
        }
        let mut cum_length = T::zero();
        let mut closest_dist_to_point = T::infinity();
        let mut fraction = T::zero();
        for segment in self.lines() {
            let segment_distance_to_point = segment.euclidean_distance(p);
            let segment_length = segment.euclidean_length();
            let segment_fraction = segment.line_locate_point(p)?; // if any segment has a None fraction, return None
            if segment_distance_to_point < closest_dist_to_point {
                closest_dist_to_point = segment_distance_to_point;
                fraction = (cum_length + segment_fraction * segment_length) / total_length;
            }
            cum_length += segment_length;
        }
        Some(fraction)
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::{point, Coordinate};

    #[test]
    fn test_line_locate_point_line() {
        // Some finite examples
        let line = Line::new(
            Coordinate { x: -1.0, y: 0.0 },
            Coordinate { x: 1.0, y: 0.0 },
        );
        let point = Point::new(0.0, 1.0);
        assert_eq!(line.line_locate_point(&point), Some(0.5));

        let point = Point::new(1.0, 1.0);
        assert_eq!(line.line_locate_point(&point), Some(1.0));

        let point = Point::new(2.0, 1.0);
        assert_eq!(line.line_locate_point(&point), Some(1.0));

        let point = Point::new(-1.0, 1.0);
        assert_eq!(line.line_locate_point(&point), Some(0.0));

        let point = Point::new(-2.0, 1.0);
        assert_eq!(line.line_locate_point(&point), Some(0.0));

        // point contains inf or nan
        let point = Point::new(Float::nan(), 1.0);
        assert_eq!(line.line_locate_point(&point), None);

        let point = Point::new(Float::infinity(), 1.0);
        assert_eq!(line.line_locate_point(&point), None);

        let point = Point::new(Float::neg_infinity(), 1.0);
        assert_eq!(line.line_locate_point(&point), None);

        // line contains inf or nan
        let line = Line::new(
            Coordinate { x: 0.0, y: 0.0 },
            Coordinate {
                x: Float::infinity(),
                y: 0.0,
            },
        );
        let point = Point::new(1000.0, 1000.0);
        assert_eq!(line.line_locate_point(&point), None);

        let line = Line::new(
            Coordinate { x: 0.0, y: 0.0 },
            Coordinate {
                x: Float::neg_infinity(),
                y: 0.0,
            },
        );
        let point = Point::new(1000.0, 1000.0);
        assert_eq!(line.line_locate_point(&point), None);

        let line = Line::new(
            Coordinate { x: 0.0, y: 0.0 },
            Coordinate {
                x: Float::nan(),
                y: 0.0,
            },
        );
        let point = Point::new(1000.0, 1000.0);
        assert_eq!(line.line_locate_point(&point), None);

        // zero length line
        let line: Line<f64> =
            Line::new(Coordinate { x: 1.0, y: 1.0 }, Coordinate { x: 1.0, y: 1.0 });
        let pt = point!(x: 2.0, y: 2.0);
        assert_eq!(line.line_locate_point(&pt), Some(0.0));

        // another concrete example
        let line: Line<f64> = Line::new(
            Coordinate { x: 0.0, y: 0.0 },
            Coordinate { x: 10.0, y: 0.0 },
        );
        let pt = Point::new(555.0, 555.0);
        assert_eq!(line.line_locate_point(&pt), Some(1.0));
        let pt = Point::new(10.0000001, 0.0);
        assert_eq!(line.line_locate_point(&pt), Some(1.0));
        let pt = Point::new(9.0, 0.001);
        assert_eq!(line.line_locate_point(&pt), Some(0.9));
    }

    #[test]
    fn test_line_locate_point_linestring() {
        // finite example using the ring
        let ring: LineString<f64> = include!("./test_fixtures/ring.rs").into();
        let pt = point!(x: 10.0, y: 1.0);
        assert_eq!(ring.line_locate_point(&pt), Some(0.0));

        let pt = point!(x: 10.0, y: 1.0000000000000742);
        assert_eq!(ring.line_locate_point(&pt), Some(0.9999999999999988));

        let pt = point!(x: 10.0, y: 1.0);
        assert_eq!(ring.line_locate_point(&pt), Some(0.0));

        // point contains inf or nan
        let pt = point!(x: Float::nan(), y: 1.0);
        assert_eq!(ring.line_locate_point(&pt), None);

        let pt = point!(x: Float::infinity(), y: 1.0);
        assert_eq!(ring.line_locate_point(&pt), None);

        let pt = point!(x: Float::neg_infinity(), y: 1.0);
        assert_eq!(ring.line_locate_point(&pt), None);

        // point is equidistant to two line segments - return the fraction from the first closest
        let line: LineString<f64> = LineString(vec![
            (0.0, 0.0).into(),
            (1.0, 0.0).into(),
            (1.0, 1.0).into(),
            (0.0, 1.0).into(),
        ]);
        let pt = point!(x: 0.0, y: 0.5);
        assert_eq!(line.line_locate_point(&pt), Some(0.0));

        let line: LineString<f64> = LineString(vec![
            (1.0, 1.0).into(),
            (1.0, 1.0).into(),
            (1.0, 1.0).into(),
        ]);
        let pt = point!(x: 2.0, y: 2.0);
        assert_eq!(line.line_locate_point(&pt), Some(0.0));

        // line contains inf or nan
        let line: LineString<f64> = LineString(vec![
            Coordinate { x: 1.0, y: 1.0 },
            Coordinate {
                x: Float::nan(),
                y: 1.0,
            },
            Coordinate { x: 0.0, y: 0.0 },
        ]);
        let pt = point!(x: 2.0, y: 2.0);
        assert_eq!(line.line_locate_point(&pt), None);

        let line: LineString<f64> = LineString(vec![
            Coordinate { x: 1.0, y: 1.0 },
            Coordinate {
                x: Float::infinity(),
                y: 1.0,
            },
            Coordinate { x: 0.0, y: 0.0 },
        ]);
        let pt = point!(x: 2.0, y: 2.0);
        assert_eq!(line.line_locate_point(&pt), None);
        let line: LineString<f64> = LineString(vec![
            Coordinate { x: 1.0, y: 1.0 },
            Coordinate {
                x: Float::neg_infinity(),
                y: 1.0,
            },
            Coordinate { x: 0.0, y: 0.0 },
        ]);
        let pt = point!(x: 2.0, y: 2.0);
        assert_eq!(line.line_locate_point(&pt), None);
    }
}
