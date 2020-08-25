use num_traits::{Float, One, Zero};
use std::ops::AddAssign;

use crate::{
    algorithm::euclidean_length::EuclideanLength, Coordinate, CoordinateType, Line, LineString,
    Point,
};

/// Returns the point that lies a given fraction along the line.
///
/// If the given fraction is
///  * less than zero: returns the starting point
///  * greater than one: returns the end point
///
/// # Examples
///
/// ```
/// use geo::{LineString, point};
/// use geo::algorithm::line_interpolate_point::LineInterpolatePoint;
///
/// let linestring: LineString<f64> = vec![
///     [-1.0, 0.0],
///     [0.0, 0.0],
///     [0.0, 1.0]
/// ].into();
///
/// assert_eq!(linestring.line_interpolate_point(&-1.0), point!(x: -1.0, y: 0.0));
/// assert_eq!(linestring.line_interpolate_point(&0.25), point!(x: -0.5, y: 0.0));
/// assert_eq!(linestring.line_interpolate_point(&0.5), point!(x: 0.0, y: 0.0));
/// assert_eq!(linestring.line_interpolate_point(&0.75), point!(x: 0.0, y: 0.5));
/// assert_eq!(linestring.line_interpolate_point(&2.0), point!(x: 0.0, y: 1.0));
/// ```
pub trait LineInterpolatePoint<F: Float> {
    type Output;

    fn line_interpolate_point(&self, fraction: &F) -> Self::Output;
}

impl<T> LineInterpolatePoint<T> for Line<T>
where
    T: CoordinateType + Float + Zero + One,
{
    type Output = Point<T>;

    fn line_interpolate_point(&self, fraction: &T) -> Self::Output {
        if fraction < &T::zero() {
            return self.start.into();
        };
        if fraction > &T::one() {
            return self.end.into();
        };
        let s = [self.start.x, self.start.y];
        let v = [self.end.x - self.start.x, self.end.y - self.start.y];
        let r = [*fraction * v[0] + s[0], *fraction * v[1] + s[1]];
        Coordinate { x: r[0], y: r[1] }.into()
    }
}

impl<T> LineInterpolatePoint<T> for LineString<T>
where
    T: CoordinateType + Float + Zero + AddAssign + One,
    Line<T>: EuclideanLength<T>,
    LineString<T>: EuclideanLength<T>,
{
    type Output = Point<T>;

    fn line_interpolate_point(&self, fraction: &T) -> Self::Output {
        let total_length = self.euclidean_length();
        let fractional_length = total_length.clone() * *fraction;
        let mut cum_length = T::zero();
        let mut queue = Vec::new();
        for line in self.lines() {
            let length = line.euclidean_length();
            queue.push((
                cum_length.clone(),
                cum_length.clone() + length.clone(),
                length.clone(),
                line.clone(),
            ));
            cum_length += length;
        }
        match queue.iter().find(|x| x.1 >= fractional_length) {
            Some(x) => {
                let line_frac = (fractional_length - x.0) / x.2;
                (x.3).line_interpolate_point(&line_frac)
            }
            None => self.points_iter().last().unwrap(),
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::{
        algorithm::{closest_point::ClosestPoint, line_locate_point::LineLocatePoint},
        point,
    };

    #[test]
    fn test_line_interpolate_point_line() {
        let line = Line::new(
            Coordinate { x: -1.0, y: 0.0 },
            Coordinate { x: 1.0, y: 0.0 },
        );
        assert_eq!(line.line_interpolate_point(&-1.0), point!(x: -1.0, y: 0.0));
        assert_eq!(line.line_interpolate_point(&0.5), point!(x: 0.0, y: 0.0));
        assert_eq!(line.line_interpolate_point(&0.75), point!(x: 0.5, y: 0.0));
        assert_eq!(line.line_interpolate_point(&0.0), point!(x: -1.0, y: 0.0));
        assert_eq!(line.line_interpolate_point(&1.0), point!(x: 1.0, y: 0.0));
        assert_eq!(line.line_interpolate_point(&2.0), point!(x: 1.0, y: 0.0));

        let line = Line::new(Coordinate { x: 0.0, y: 0.0 }, Coordinate { x: 1.0, y: 1.0 });
        assert_eq!(line.line_interpolate_point(&0.5), point!(x: 0.5, y: 0.5));
    }

    #[test]
    fn test_line_interpolate_point_linestring() {
        let linestring: LineString<f64> = vec![[-1.0, 0.0], [0.0, 0.0], [1.0, 0.0]].into();
        assert_eq!(
            linestring.line_interpolate_point(&0.5),
            point!(x: 0.0, y: 0.0)
        );
        assert_eq!(
            linestring.line_interpolate_point(&1.0),
            point!(x: 1.0, y: 0.0)
        );

        let linestring: LineString<f64> = vec![[-1.0, 0.0], [0.0, 0.0], [0.0, 1.0]].into();
        assert_eq!(
            linestring.line_interpolate_point(&1.5),
            point!(x: 0.0, y: 1.0)
        );
    }

    #[test]
    fn test_matches_closest_point() {
        let linestring: LineString<f64> = vec![[-1.0, 0.0], [0.5, 1.0], [1.0, 2.0]].into();
        let pt = point!(x: 0.7, y: 0.7);
        let frac = linestring.line_locate_point(&pt);
        println!("{:?}", &frac);
        let interpolated_point = linestring.line_interpolate_point(&frac);
        let closest_point = linestring.closest_point(&pt);
        match closest_point {
            crate::Closest::SinglePoint(p) => assert_eq!(interpolated_point, p),
            _ => panic!("The closest point should be a SinglePoint"),
        };
    }
}
