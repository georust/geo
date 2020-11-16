use num_traits::Float;
use std::ops::AddAssign;

use crate::{
    algorithm::euclidean_length::EuclideanLength, CoordinateType, Line, LineString, Point,
};

/// Returns an option of the point that lies a given fraction along the line.
///
/// If the given fraction is
///  * less than zero (including negative infinity): returns a `Some`
///    of the starting point
///  * greater than one (including infinity): returns a `Some` of the ending point
///
///  If either the fraction is NaN, or any coordinates of the line are not
///  finite, returns `None`.
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
/// assert_eq!(linestring.line_interpolate_point(-1.0), Some(point!(x: -1.0, y: 0.0)));
/// assert_eq!(linestring.line_interpolate_point(0.25), Some(point!(x: -0.5, y: 0.0)));
/// assert_eq!(linestring.line_interpolate_point(0.5), Some(point!(x: 0.0, y: 0.0)));
/// assert_eq!(linestring.line_interpolate_point(0.75), Some(point!(x: 0.0, y: 0.5)));
/// assert_eq!(linestring.line_interpolate_point(2.0), Some(point!(x: 0.0, y: 1.0)));
/// ```
pub trait LineInterpolatePoint<F: Float> {
    type Output;

    fn line_interpolate_point(&self, fraction: F) -> Self::Output;
}

impl<T> LineInterpolatePoint<T> for Line<T>
where
    T: CoordinateType + Float,
{
    type Output = Option<Point<T>>;

    fn line_interpolate_point(&self, fraction: T) -> Self::Output {
        if (fraction >= T::zero()) && (fraction <= T::one()) {
            // fraction between 0 and 1, return a point between start and end
            let diff = self.end - self.start;
            let r = self.start + diff * (fraction);
            if r.x.is_finite() && r.y.is_finite() {
                return Some(r.into());
            } else {
                return None;
            }
        } else if fraction < T::zero() {
            // negative fractions are replaced with zero
            return self.line_interpolate_point(T::zero());
        } else if fraction > T::one() {
            // fractions above one are replaced with one
            return self.line_interpolate_point(T::one());
        } else {
            // fraction is nan
            debug_assert!(fraction.is_nan());
            return None;
        }
    }
}

impl<T> LineInterpolatePoint<T> for LineString<T>
where
    T: CoordinateType + Float + AddAssign + std::fmt::Debug,
    Line<T>: EuclideanLength<T>,
    LineString<T>: EuclideanLength<T>,
{
    type Output = Option<Point<T>>;

    fn line_interpolate_point(&self, fraction: T) -> Self::Output {
        if (fraction >= T::zero()) && (fraction <= T::one()) {
            // find the point along the linestring which is fraction along it
            let total_length = self.euclidean_length();
            let fractional_length = total_length * fraction;
            let mut cum_length = T::zero();
            for segment in self.lines() {
                let length = segment.euclidean_length();
                if cum_length + length >= fractional_length {
                    let segment_fraction = (fractional_length - cum_length) / length;
                    return segment.line_interpolate_point(segment_fraction);
                }
                cum_length += length;
            }
            // either cum_length + length is never larger than fractional_length, i.e.
            // fractional_length is nan, or the linestring has no lines to loop through
            debug_assert!(fractional_length.is_nan() || (self.num_coords() == 0));
            return None;
        } else if fraction < T::zero() {
            // negative fractions replaced with zero
            return self.line_interpolate_point(T::zero());
        } else if fraction > T::one() {
            // fractions above one replaced with one
            return self.line_interpolate_point(T::one());
        } else {
            // fraction is nan
            debug_assert!(fraction.is_nan());
            return None;
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::{
        algorithm::{closest_point::ClosestPoint, line_locate_point::LineLocatePoint},
        point, Coordinate,
    };

    #[test]
    fn test_line_interpolate_point_line() {
        let line = Line::new(
            Coordinate { x: -1.0, y: 0.0 },
            Coordinate { x: 1.0, y: 0.0 },
        );
        // some finite examples
        assert_eq!(
            line.line_interpolate_point(-1.0),
            Some(point!(x: -1.0, y: 0.0))
        );
        assert_eq!(
            line.line_interpolate_point(0.5),
            Some(point!(x: 0.0, y: 0.0))
        );
        assert_eq!(
            line.line_interpolate_point(0.75),
            Some(point!(x: 0.5, y: 0.0))
        );
        assert_eq!(
            line.line_interpolate_point(0.0),
            Some(point!(x: -1.0, y: 0.0))
        );
        assert_eq!(
            line.line_interpolate_point(1.0),
            Some(point!(x: 1.0, y: 0.0))
        );
        assert_eq!(
            line.line_interpolate_point(2.0),
            Some(point!(x: 1.0, y: 0.0))
        );

        // fraction is nan or inf
        assert_eq!(line.line_interpolate_point(Float::nan()), None);
        assert_eq!(
            line.line_interpolate_point(Float::infinity()),
            Some(line.end_point())
        );
        assert_eq!(
            line.line_interpolate_point(Float::neg_infinity()),
            Some(line.start_point())
        );

        let line = Line::new(Coordinate { x: 0.0, y: 0.0 }, Coordinate { x: 1.0, y: 1.0 });
        assert_eq!(
            line.line_interpolate_point(0.5),
            Some(point!(x: 0.5, y: 0.5))
        );

        // line contains nans or infs
        let line = Line::new(
            Coordinate {
                x: Float::nan(),
                y: 0.0,
            },
            Coordinate { x: 1.0, y: 1.0 },
        );
        assert_eq!(line.line_interpolate_point(0.5), None);

        let line = Line::new(
            Coordinate {
                x: Float::infinity(),
                y: 0.0,
            },
            Coordinate { x: 1.0, y: 1.0 },
        );
        assert_eq!(line.line_interpolate_point(0.5), None);

        let line = Line::new(
            Coordinate { x: 0.0, y: 0.0 },
            Coordinate {
                x: 1.0,
                y: Float::infinity(),
            },
        );
        assert_eq!(line.line_interpolate_point(0.5), None);

        let line = Line::new(
            Coordinate {
                x: Float::neg_infinity(),
                y: 0.0,
            },
            Coordinate { x: 1.0, y: 1.0 },
        );
        assert_eq!(line.line_interpolate_point(0.5), None);

        let line = Line::new(
            Coordinate { x: 0.0, y: 0.0 },
            Coordinate {
                x: 1.0,
                y: Float::neg_infinity(),
            },
        );
        assert_eq!(line.line_interpolate_point(0.5), None);
    }

    #[test]
    fn test_line_interpolate_point_linestring() {
        // some finite examples
        let linestring: LineString<f64> = vec![[-1.0, 0.0], [0.0, 0.0], [1.0, 0.0]].into();
        assert_eq!(
            linestring.line_interpolate_point(0.0),
            Some(point!(x: -1.0, y: 0.0))
        );
        assert_eq!(
            linestring.line_interpolate_point(0.5),
            Some(point!(x: 0.0, y: 0.0))
        );
        assert_eq!(
            linestring.line_interpolate_point(1.0),
            Some(point!(x: 1.0, y: 0.0))
        );
        assert_eq!(
            linestring.line_interpolate_point(1.0),
            linestring.line_interpolate_point(2.0)
        );
        assert_eq!(
            linestring.line_interpolate_point(0.0),
            linestring.line_interpolate_point(-2.0)
        );

        // fraction is nan or inf
        assert_eq!(
            linestring.line_interpolate_point(Float::infinity()),
            linestring.points_iter().last()
        );
        assert_eq!(
            linestring.line_interpolate_point(Float::neg_infinity()),
            linestring.points_iter().next()
        );
        assert_eq!(linestring.line_interpolate_point(Float::nan()), None);

        let linestring: LineString<f64> = vec![[-1.0, 0.0], [0.0, 0.0], [0.0, 1.0]].into();
        assert_eq!(
            linestring.line_interpolate_point(0.5),
            Some(point!(x: 0.0, y: 0.0))
        );
        assert_eq!(
            linestring.line_interpolate_point(1.5),
            Some(point!(x: 0.0, y: 1.0))
        );

        // linestrings with nans/infs
        let linestring: LineString<f64> = vec![[-1.0, 0.0], [0.0, Float::nan()], [0.0, 1.0]].into();
        assert_eq!(linestring.line_interpolate_point(0.5), None);
        assert_eq!(linestring.line_interpolate_point(1.5), None);
        assert_eq!(linestring.line_interpolate_point(-1.0), None);

        let linestring: LineString<f64> =
            vec![[-1.0, 0.0], [0.0, Float::infinity()], [0.0, 1.0]].into();
        assert_eq!(linestring.line_interpolate_point(0.5), None);
        assert_eq!(linestring.line_interpolate_point(1.5), None);
        assert_eq!(linestring.line_interpolate_point(-1.0), None);

        let linestring: LineString<f64> =
            vec![[-1.0, 0.0], [0.0, Float::neg_infinity()], [0.0, 1.0]].into();
        assert_eq!(linestring.line_interpolate_point(0.5), None);
        assert_eq!(linestring.line_interpolate_point(1.5), None);
        assert_eq!(linestring.line_interpolate_point(-1.0), None);

        // Empty line
        let coords: Vec<Point<f64>> = Vec::new();
        let linestring: LineString<f64> = coords.into();
        assert_eq!(linestring.line_interpolate_point(0.5), None);
    }

    #[test]
    fn test_matches_closest_point() {
        // line_locate_point should return the fraction to the closest point,
        // so interpolating the line with that fraction should yield the closest point
        let linestring: LineString<f64> = vec![[-1.0, 0.0], [0.5, 1.0], [1.0, 2.0]].into();
        let pt = point!(x: 0.7, y: 0.7);
        let frac = linestring
            .line_locate_point(&pt)
            .expect("Should result in fraction between 0 and 1");
        println!("{:?}", &frac);
        let interpolated_point = linestring
            .line_interpolate_point(frac)
            .expect("Shouldn't return None");
        let closest_point = linestring.closest_point(&pt);
        match closest_point {
            crate::Closest::SinglePoint(p) => assert_eq!(interpolated_point, p),
            _ => panic!("The closest point should be a SinglePoint"), // example chosen to not be an intersection
        };
    }
}
