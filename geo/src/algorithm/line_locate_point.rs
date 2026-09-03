// This algorithm will be deprecated in the future, replaced by a unified implementation
// rather than being Euclidean specific. Until the alternative is available, lets allow deprecations
// so as not to change the method signature for existing users.
#[allow(deprecated)]
use crate::{
    CoordFloat, Line, LineString, Point,
    {euclidean_distance::EuclideanDistance, euclidean_length::EuclideanLength},
};
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
/// use geo::LineLocatePoint;
///
/// let linestring: LineString = vec![
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
    T: CoordFloat,
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
            Some(T::zero())
        } else {
            // $v \cdot (p - s)$
            let v_dot_sp = v.dot(sp);
            let l = v_dot_sp / v_sq;
            if l.is_finite() {
                Some(l.max(T::zero()).min(T::one()))
            } else {
                None
            }
        }
    }
}

#[allow(deprecated)]
impl<T> LineLocatePoint<T, Point<T>> for LineString<T>
where
    T: CoordFloat + AddAssign,
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
    use crate::{coord, point};
    use num_traits::Float;

    #[test]
    fn test_line_locate_point_line() {
        // Some finite examples
        let line = Line::new(coord! { x: -1.0, y: 0.0 }, coord! { x: 1.0, y: 0.0 });
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
            coord! { x: 0.0, y: 0.0 },
            coord! {
                x: Float::infinity(),
                y: 0.0,
            },
        );
        let point = Point::new(1000.0, 1000.0);
        assert_eq!(line.line_locate_point(&point), None);

        let line = Line::new(
            coord! { x: 0.0, y: 0.0 },
            coord! {
                x: Float::neg_infinity(),
                y: 0.0,
            },
        );
        let point = Point::new(1000.0, 1000.0);
        assert_eq!(line.line_locate_point(&point), None);

        let line = Line::new(
            coord! { x: 0.0, y: 0.0 },
            coord! {
                x: Float::nan(),
                y: 0.0,
            },
        );
        let point = Point::new(1000.0, 1000.0);
        assert_eq!(line.line_locate_point(&point), None);

        // zero length line
        let line: Line = Line::new(coord! { x: 1.0, y: 1.0 }, coord! { x: 1.0, y: 1.0 });
        let pt = point!(x: 2.0, y: 2.0);
        assert_eq!(line.line_locate_point(&pt), Some(0.0));

        // another concrete example
        let line: Line = Line::new(coord! { x: 0.0, y: 0.0 }, coord! { x: 10.0, y: 0.0 });
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
        let ring: LineString = geo_test_fixtures::ring::<f64>();
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
        let line: LineString = LineString::new(vec![
            (0.0, 0.0).into(),
            (1.0, 0.0).into(),
            (1.0, 1.0).into(),
            (0.0, 1.0).into(),
        ]);
        let pt = point!(x: 0.0, y: 0.5);
        assert_eq!(line.line_locate_point(&pt), Some(0.0));

        let line: LineString = LineString::new(vec![
            (1.0, 1.0).into(),
            (1.0, 1.0).into(),
            (1.0, 1.0).into(),
        ]);
        let pt = point!(x: 2.0, y: 2.0);
        assert_eq!(line.line_locate_point(&pt), Some(0.0));

        // line contains inf or nan
        let line: LineString = LineString::new(vec![
            coord! { x: 1.0, y: 1.0 },
            coord! {
                x: Float::nan(),
                y: 1.0,
            },
            coord! { x: 0.0, y: 0.0 },
        ]);
        let pt = point!(x: 2.0, y: 2.0);
        assert_eq!(line.line_locate_point(&pt), None);

        let line: LineString = LineString::new(vec![
            coord! { x: 1.0, y: 1.0 },
            coord! {
                x: Float::infinity(),
                y: 1.0,
            },
            coord! { x: 0.0, y: 0.0 },
        ]);
        let pt = point!(x: 2.0, y: 2.0);
        assert_eq!(line.line_locate_point(&pt), None);
        let line: LineString = LineString::new(vec![
            coord! { x: 1.0, y: 1.0 },
            coord! {
                x: Float::neg_infinity(),
                y: 1.0,
            },
            coord! { x: 0.0, y: 0.0 },
        ]);
        let pt = point!(x: 2.0, y: 2.0);
        assert_eq!(line.line_locate_point(&pt), None);
    }
}

#[cfg(test)]
mod hegel_props {
    use super::LineLocatePoint;
    use crate::utils::hegel_gens::{coords, monotone_line_strings};
    use crate::{Closest, ClosestPoint, Distance, Euclidean, InterpolateLine, Length, Point};

    // "Returns a (option of the) fraction of the line's total length
    // representing the location of the closest point on the line", so the
    // fraction lies in `[0, 1]`; the `Line` impl clamps it there explicitly.
    #[hegel::test]
    fn the_located_fraction_lies_between_zero_and_one(tc: hegel::TestCase) {
        let line_string = tc.draw(monotone_line_strings(1e3, 10));
        let point = Point::from(tc.draw(coords(2e3)));
        let fraction = line_string
            .line_locate_point(&point)
            .expect("finite coordinates");
        assert!(
            (0.0..=1.0).contains(&fraction),
            "fraction {fraction} is outside [0, 1]"
        );
    }

    // "line_locate_point should return the fraction to the closest point, so
    // interpolating the line with that fraction should yield the closest point"
    // — the comment on `test_matches_closest_point` in
    // `line_interpolate_point`.
    #[hegel::test]
    fn interpolating_the_located_fraction_gives_the_closest_point(tc: hegel::TestCase) {
        let line_string = tc.draw(monotone_line_strings(1e3, 10));
        let point = Point::from(tc.draw(coords(2e3)));
        let fraction = line_string
            .line_locate_point(&point)
            .expect("finite coordinates");
        let interpolated = Euclidean
            .point_at_ratio_from_start(&line_string, fraction)
            .expect("a non-empty line string");
        let (Closest::SinglePoint(closest) | Closest::Intersection(closest)) =
            line_string.closest_point(&point)
        else {
            return;
        };
        let scale = Euclidean.length(&line_string).max(1.0);
        assert!(
            Euclidean.distance(&interpolated, &closest) <= 1e-9 * scale,
            "interpolating {fraction} gave {interpolated:?}, closest point is {closest:?}"
        );
    }

    // A vertex of the line string is on it, so the located fraction points back
    // at that vertex.
    #[hegel::test]
    fn locating_a_vertex_returns_a_fraction_that_interpolates_back_to_it(tc: hegel::TestCase) {
        let line_string = tc.draw(monotone_line_strings(1e3, 10));
        let index =
            tc.draw(hegel::generators::integers::<usize>().max_value(line_string.0.len() - 1));
        let vertex = Point::from(line_string.0[index]);
        let fraction = line_string
            .line_locate_point(&vertex)
            .expect("finite coordinates");
        let interpolated = Euclidean
            .point_at_ratio_from_start(&line_string, fraction)
            .expect("a non-empty line string");
        let scale = Euclidean.length(&line_string).max(1.0);
        assert!(
            Euclidean.distance(&interpolated, &vertex) <= 1e-9 * scale,
            "vertex {vertex:?} located at {fraction} interpolates to {interpolated:?}"
        );
    }
}
