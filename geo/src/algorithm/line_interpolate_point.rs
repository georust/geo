use geo_types::{MultiLineString, MultiPolygon};

use crate::coords_iter::CoordsIter;
use std::ops::AddAssign;

use crate::{CoordFloat, EuclideanLength, Line, LineString, Point, Polygon};

/// Return a new linear geometry containing both existing and new interpolated coordinates with
/// a maximum distance of `max_distance` between them.
///
/// # Examples
/// ```
/// use geo::{coord, Line, LineString};
/// use geo::line_interpolate_point::Densify;
///
/// let line: Line<f64> = Line::new(coord! {x: 0.0, y: 6.0}, coord! {x: 1.0, y: 8.0});
/// let correct: LineString<f64> = vec![[0.0, 6.0], [0.5, 7.0], [1.0, 8.0]].into();
/// let max_dist = 2.0;
/// let densified = line.densify(max_dist);
/// assert_eq!(densified, correct);
///```
pub trait Densify<F: CoordFloat> {
    type Output;

    fn densify(&self, max_distance: F) -> Self::Output;
}

// Helper for densification trait
fn densify_line<T: CoordFloat>(line: Line<T>, container: &mut Vec<Point<T>>, max_distance: T) {
    container.push(line.start_point());
    let num_segments = (line.euclidean_length() / max_distance).ceil() - T::one();
    if num_segments > T::zero() {
        // distance "unit" for this line segment
        let frac = T::one() / (T::from(num_segments).unwrap() + T::one());
        // conversion to int should be OK because we've already called ceil
        (0..num_segments.to_u32().unwrap())
            .enumerate()
            .for_each(|(seg, _)| {
                // multiply distance unit by 1-indexed segment number to get correct offset
                let np = line
                    .line_interpolate_point(frac * (T::from(seg).unwrap() + T::one()))
                    .unwrap();
                container.push(np);
            })
    }
}

impl<T> Densify<T> for MultiPolygon<T>
where
    T: CoordFloat,
    Line<T>: EuclideanLength<T>,
    LineString<T>: EuclideanLength<T>,
{
    type Output = MultiPolygon<T>;

    fn densify(&self, max_distance: T) -> Self::Output {
        MultiPolygon::new(
            self.iter()
                .map(|polygon| polygon.densify(max_distance))
                .collect(),
        )
    }
}

impl<T> Densify<T> for Polygon<T>
where
    T: CoordFloat,
    Line<T>: EuclideanLength<T>,
    LineString<T>: EuclideanLength<T>,
{
    type Output = Polygon<T>;

    fn densify(&self, max_distance: T) -> Self::Output {
        let densified_exterior = self.exterior().densify(max_distance);
        let densified_interiors = self
            .interiors()
            .iter()
            .map(|ring| ring.densify(max_distance))
            .collect();
        Polygon::new(densified_exterior, densified_interiors)
    }
}

impl<T> Densify<T> for MultiLineString<T>
where
    T: CoordFloat,
    Line<T>: EuclideanLength<T>,
    LineString<T>: EuclideanLength<T>,
{
    type Output = MultiLineString<T>;

    fn densify(&self, max_distance: T) -> Self::Output {
        MultiLineString::new(
            self.iter()
                .map(|linestring| linestring.densify(max_distance))
                .collect(),
        )
    }
}

impl<T> Densify<T> for LineString<T>
where
    T: CoordFloat,
    Line<T>: EuclideanLength<T>,
    LineString<T>: EuclideanLength<T>,
{
    type Output = LineString<T>;

    fn densify(&self, max_distance: T) -> Self::Output {
        let mut new_line = vec![];
        self.lines()
            .for_each(|line| densify_line(line, &mut new_line, max_distance));
        // we're done, push the last coordinate on to finish
        new_line.push(self.points().last().unwrap());
        LineString::from(new_line)
    }
}

impl<T> Densify<T> for Line<T>
where
    T: CoordFloat,
    Line<T>: EuclideanLength<T>,
    LineString<T>: EuclideanLength<T>,
{
    type Output = LineString<T>;

    fn densify(&self, max_distance: T) -> Self::Output {
        let mut new_line = vec![];
        densify_line(*self, &mut new_line, max_distance);
        // we're done, push the last coordinate on to finish
        new_line.push(self.end_point());
        LineString::from(new_line)
    }
}

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
/// use geo::LineInterpolatePoint;
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
pub trait LineInterpolatePoint<F: CoordFloat> {
    type Output;

    fn line_interpolate_point(&self, fraction: F) -> Self::Output;
}

impl<T> LineInterpolatePoint<T> for Line<T>
where
    T: CoordFloat,
{
    type Output = Option<Point<T>>;

    fn line_interpolate_point(&self, fraction: T) -> Self::Output {
        if (fraction >= T::zero()) && (fraction <= T::one()) {
            // fraction between 0 and 1, return a point between start and end
            let diff = self.end - self.start;
            let r = self.start + diff * (fraction);
            if r.x.is_finite() && r.y.is_finite() {
                Some(r.into())
            } else {
                None
            }
        } else if fraction < T::zero() {
            // negative fractions are replaced with zero
            self.line_interpolate_point(T::zero())
        } else if fraction > T::one() {
            // fractions above one are replaced with one
            self.line_interpolate_point(T::one())
        } else {
            // fraction is nan
            debug_assert!(fraction.is_nan());
            None
        }
    }
}

impl<T> LineInterpolatePoint<T> for LineString<T>
where
    T: CoordFloat + AddAssign + std::fmt::Debug,
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
            debug_assert!(fractional_length.is_nan() || (self.coords_count() == 0));
            None
        } else if fraction < T::zero() {
            // negative fractions replaced with zero
            self.line_interpolate_point(T::zero())
        } else if fraction > T::one() {
            // fractions above one replaced with one
            self.line_interpolate_point(T::one())
        } else {
            // fraction is nan
            debug_assert!(fraction.is_nan());
            None
        }
    }
}

#[cfg(test)]
mod test {

    use super::*;
    use crate::{coord, point};
    use crate::{ClosestPoint, LineLocatePoint};
    use num_traits::Float;

    #[test]
    fn test_line_interpolate_point_line() {
        let line = Line::new(coord! { x: -1.0, y: 0.0 }, coord! { x: 1.0, y: 0.0 });
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

        let line = Line::new(coord! { x: 0.0, y: 0.0 }, coord! { x: 1.0, y: 1.0 });
        assert_eq!(
            line.line_interpolate_point(0.5),
            Some(point!(x: 0.5, y: 0.5))
        );

        // line contains nans or infs
        let line = Line::new(
            coord! {
                x: Float::nan(),
                y: 0.0,
            },
            coord! { x: 1.0, y: 1.0 },
        );
        assert_eq!(line.line_interpolate_point(0.5), None);

        let line = Line::new(
            coord! {
                x: Float::infinity(),
                y: 0.0,
            },
            coord! { x: 1.0, y: 1.0 },
        );
        assert_eq!(line.line_interpolate_point(0.5), None);

        let line = Line::new(
            coord! { x: 0.0, y: 0.0 },
            coord! {
                x: 1.0,
                y: Float::infinity(),
            },
        );
        assert_eq!(line.line_interpolate_point(0.5), None);

        let line = Line::new(
            coord! {
                x: Float::neg_infinity(),
                y: 0.0,
            },
            coord! { x: 1.0, y: 1.0 },
        );
        assert_eq!(line.line_interpolate_point(0.5), None);

        let line = Line::new(
            coord! { x: 0.0, y: 0.0 },
            coord! {
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
            linestring.points().last()
        );
        assert_eq!(
            linestring.line_interpolate_point(Float::neg_infinity()),
            linestring.points().next()
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
        let interpolated_point = linestring
            .line_interpolate_point(frac)
            .expect("Shouldn't return None");
        let closest_point = linestring.closest_point(&pt);
        match closest_point {
            crate::Closest::SinglePoint(p) => assert_eq!(interpolated_point, p),
            _ => panic!("The closest point should be a SinglePoint"), // example chosen to not be an intersection
        };
    }
    #[test]
    fn test_linestring_densify() {
        let linestring: LineString<f64> =
            vec![[-1.0, 0.0], [0.0, 0.0], [0.0, 6.0], [1.0, 8.0]].into();
        let correct: LineString<f64> = vec![
            [-1.0, 0.0],
            [0.0, 0.0],
            [0.0, 2.0],
            [0.0, 4.0],
            [0.0, 6.0],
            [0.5, 7.0],
            [1.0, 8.0],
        ]
        .into();
        let max_dist = 2.0;
        let densified = linestring.densify(max_dist);
        assert_eq!(densified, correct);
    }

    #[test]
    fn test_line_densify() {
        let line: Line<f64> = Line::new(coord! {x: 0.0, y: 6.0}, coord! {x: 1.0, y: 8.0});
        let correct: LineString<f64> = vec![[0.0, 6.0], [0.5, 7.0], [1.0, 8.0]].into();
        let max_dist = 2.0;
        let densified = line.densify(max_dist);
        assert_eq!(densified, correct);
    }
}
