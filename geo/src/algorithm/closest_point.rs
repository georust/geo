use crate::Closest;
use crate::GeoFloat;
use crate::algorithm::{Euclidean, Intersects, Length};
use crate::geometry::*;

use std::iter;

/// Find the closest `Point` between a given geometry and an input `Point`.
/// The closest point may intersect the geometry, be a single
/// point, or be indeterminate, as indicated by the value of the returned enum.
///
/// # Examples
///
/// We have a horizontal line which goes through `(-50, 0) -> (50, 0)`,
/// and want to find the closest point to the point `(0, 100)`.
/// Drawn on paper, the point on the line which is closest to `(0, 100)` is the origin (0, 0).
///
/// ```rust
/// # use geo::ClosestPoint;
/// # use geo::{Point, Line, Closest};
/// let p: Point<f32> = Point::new(0.0, 100.0);
/// let horizontal_line: Line<f32> = Line::new(Point::new(-50.0, 0.0), Point::new(50.0, 0.0));
///
/// let closest = horizontal_line.closest_point(&p);
/// assert_eq!(closest, Closest::SinglePoint(Point::new(0.0, 0.0)));
/// ```
pub trait ClosestPoint<F: GeoFloat, Rhs = Point<F>> {
    /// Find the closest point between `self` and `p`.
    fn closest_point(&self, p: &Rhs) -> Closest<F>;
}

impl<F, C> ClosestPoint<F> for &'_ C
where
    C: ClosestPoint<F>,
    F: GeoFloat,
{
    fn closest_point(&self, p: &Point<F>) -> Closest<F> {
        (*self).closest_point(p)
    }
}

impl<F: GeoFloat> ClosestPoint<F> for Point<F> {
    fn closest_point(&self, p: &Self) -> Closest<F> {
        if self == p {
            Closest::Intersection(*self)
        } else {
            Closest::SinglePoint(*self)
        }
    }
}

#[allow(clippy::many_single_char_names)]
impl<F: GeoFloat> ClosestPoint<F> for Line<F> {
    fn closest_point(&self, p: &Point<F>) -> Closest<F> {
        let line_length = Euclidean.length(self);
        if line_length == F::zero() {
            // if we've got a zero length line, technically the entire line
            // is the closest point...
            return Closest::Indeterminate;
        }

        // For some line AB, there is some point, C, which will be closest to
        // P. The line AB will be perpendicular to CP.
        //
        // Line equation: P = start + t * (end - start)

        let direction_vector = Point::from(self.end - self.start);
        let to_p = Point::from(p.0 - self.start);

        let numerator = to_p.dot(direction_vector);
        let squared_length = direction_vector.dot(direction_vector);
        // The dot products overflow or underflow for long or short segments.
        // Use the same test as `line_segment_distance` in geo-types. A NaN or
        // infinite input keeps the unscaled quotient.
        let scaled = !(squared_length.is_normal() && numerator.is_finite())
            && is_finite(self.start)
            && is_finite(self.end)
            && is_finite(p.0);
        let t = if scaled {
            scaled_projection(self.start, self.end, p.0)
        } else {
            numerator / squared_length
        };

        // check the cases where the closest point is "outside" the line
        if t < F::zero() {
            return Closest::SinglePoint(self.start.into());
        } else if t > F::one() {
            return Closest::SinglePoint(self.end.into());
        }

        let x = direction_vector.x();
        let y = direction_vector.y();
        let c = if scaled && !(x.is_finite() && y.is_finite()) {
            // The difference of the endpoints overflows. Interpolate between
            // them instead.
            let s = F::one() - t;
            Point::new(
                self.start.x * s + self.end.x * t,
                self.start.y * s + self.end.y * t,
            )
        } else {
            Point::from(self.start + (t * x, t * y).into())
        };

        if self.intersects(p) {
            Closest::Intersection(c)
        } else {
            Closest::SinglePoint(c)
        }
    }
}

// The exponent of the largest power of two no greater than a positive, finite
// magnitude.
fn binary_exponent<F: GeoFloat>(magnitude: F) -> i32 {
    let (mantissa, exponent, _) = magnitude.integer_decode();
    i32::from(exponent) + (u64::BITS - 1 - mantissa.leading_zeros()) as i32
}

// Multiply by 2^-exponent in two steps. Each factor is a normal power of two,
// so the product is exact.
fn scale_down<F: GeoFloat>(point: Point<F>, exponent: i32) -> Point<F> {
    let first = -exponent / 2;
    let second = -exponent - first;
    point * power_of_two(first) * power_of_two(second)
}

// Compensate for the rounded product when the two dot-product terms cancel.
fn compensated_dot<F: GeoFloat>(a: Point<F>, b: Point<F>) -> F {
    let product = a.y() * b.y();
    let correction = a.y().mul_add(b.y(), -product);
    a.x().mul_add(b.x(), product) + correction
}

fn is_finite<F: GeoFloat>(coord: Coord<F>) -> bool {
    coord.x.is_finite() && coord.y.is_finite()
}

// 2^exponent, for an exponent that is normal in F.
fn power_of_two<F: GeoFloat>(exponent: i32) -> F {
    F::from(f64::from_bits(((exponent + 1023) as u64) << 52)).unwrap()
}

// Multiply by 2^exponent in three steps. The magnitude of the exponent is at
// most about twice the largest exponent of F, so each factor is finite and
// normal. A product that overflows is beyond the end of the segment.
fn rescale_projection<F: GeoFloat>(value: F, exponent: i32) -> F {
    let first = exponent / 3;
    let second = (exponent - first) / 2;
    let third = exponent - first - second;
    value * power_of_two(first) * power_of_two(second) * power_of_two(third)
}

// The projection parameter for finite inputs and distinct endpoints, where the
// unscaled dot products overflow or underflow.
#[cold]
#[inline(never)]
fn scaled_projection<F: GeoFloat>(start: Coord<F>, end: Coord<F>, point: Coord<F>) -> F {
    let zero = F::zero();
    let two = F::one() + F::one();
    let mut direction = end - start;
    let mut to_point = point - start;
    // The difference of two finite coordinates can overflow. Halve the
    // coordinates before the subtraction. Each halving changes the projection
    // by a factor of 2.
    let mut exponent = 0;
    if !is_finite(direction) {
        direction = end / two - start / two;
        exponent -= 1;
    }
    if !is_finite(to_point) {
        to_point = point / two - start / two;
        exponent += 1;
    }
    let direction = Point::from(direction);
    let to_point = Point::from(to_point);
    // A coordinate with zero direction contributes nothing to the dot product.
    // Exclude it so a large perpendicular offset cannot erase a small, relevant
    // query component when choosing the query's scale.
    let to_point = Point::new(
        if direction.x() == zero {
            zero
        } else {
            to_point.x()
        },
        if direction.y() == zero {
            zero
        } else {
            to_point.y()
        },
    );
    let query_magnitude = to_point.x().abs().max(to_point.y().abs());
    if query_magnitude == zero {
        return zero;
    }
    let direction_exponent = binary_exponent(direction.x().abs().max(direction.y().abs()));
    let query_exponent = binary_exponent(query_magnitude);
    // Scale independently by powers of two, preserving component ratios while
    // keeping both dot products in range. Use the same compensated dot product
    // in the denominator so an end-point query still gives exactly t == 1.
    let direction = scale_down(direction, direction_exponent);
    let query = scale_down(to_point, query_exponent);
    let numerator = compensated_dot(query, direction);
    let denominator = compensated_dot(direction, direction);
    rescale_projection(
        numerator / denominator,
        exponent + query_exponent - direction_exponent,
    )
}

/// A generic function which takes some iterator of points and gives you the
/// "best" `Closest` it can find. Where "best" is the first intersection or
/// the `Closest::SinglePoint` which is closest to `p`.
///
/// If the iterator is empty, we get `Closest::Indeterminate`.
fn closest_of<C, F, I>(iter: I, p: Point<F>) -> Closest<F>
where
    F: GeoFloat,
    I: IntoIterator<Item = C>,
    C: ClosestPoint<F>,
{
    let mut best = Closest::Indeterminate;

    for element in iter {
        let got = element.closest_point(&p);
        best = got.best_of_two(&best, p);
        if matches!(best, Closest::Intersection(_)) {
            // short circuit - nothing can be closer than an intersection
            return best;
        }
    }

    best
}

impl<F: GeoFloat> ClosestPoint<F> for LineString<F> {
    fn closest_point(&self, p: &Point<F>) -> Closest<F> {
        closest_of(self.lines(), *p)
    }
}

impl<F: GeoFloat> ClosestPoint<F> for Polygon<F> {
    fn closest_point(&self, p: &Point<F>) -> Closest<F> {
        if self.intersects(p) {
            return Closest::Intersection(*p);
        }
        let prospectives = self.interiors().iter().chain(iter::once(self.exterior()));
        closest_of(prospectives, *p)
    }
}

impl<F: GeoFloat> ClosestPoint<F> for Coord<F> {
    fn closest_point(&self, p: &Point<F>) -> Closest<F> {
        Point::from(*self).closest_point(p)
    }
}

impl<F: GeoFloat> ClosestPoint<F> for Triangle<F> {
    fn closest_point(&self, p: &Point<F>) -> Closest<F> {
        if self.intersects(p) {
            return Closest::Intersection(*p);
        }
        closest_of(self.to_lines(), *p)
    }
}

impl<F: GeoFloat> ClosestPoint<F> for Rect<F> {
    fn closest_point(&self, p: &Point<F>) -> Closest<F> {
        if self.intersects(p) {
            return Closest::Intersection(*p);
        }
        closest_of(self.to_lines(), *p)
    }
}

impl<F: GeoFloat> ClosestPoint<F> for MultiPolygon<F> {
    fn closest_point(&self, p: &Point<F>) -> Closest<F> {
        closest_of(self.iter(), *p)
    }
}

impl<F: GeoFloat> ClosestPoint<F> for MultiPoint<F> {
    fn closest_point(&self, p: &Point<F>) -> Closest<F> {
        closest_of(self.iter(), *p)
    }
}

impl<F: GeoFloat> ClosestPoint<F> for MultiLineString<F> {
    fn closest_point(&self, p: &Point<F>) -> Closest<F> {
        closest_of(self.iter(), *p)
    }
}

impl<F: GeoFloat> ClosestPoint<F> for GeometryCollection<F> {
    fn closest_point(&self, p: &Point<F>) -> Closest<F> {
        closest_of(self.iter(), *p)
    }
}
impl<F: GeoFloat> ClosestPoint<F> for Geometry<F> {
    crate::geometry_delegate_impl! {
        fn closest_point(&self, p: &Point<F>) -> Closest<F>;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::algorithm::{Contains, Distance, Translate};
    use crate::{point, polygon, wkt};

    /// Create a test which checks that we get `$should_be` when trying to find
    /// the closest distance between `$p` and the line `(0, 0) -> (100, 100)`.
    macro_rules! closest {
        (intersects: $name:ident, $p:expr) => {
            closest!($name, $p => Closest::Intersection($p.into()));
        };
        ($name:ident, $p:expr => $should_be:expr) => {
            #[test]
            fn $name() {
                let line: Line<f32> = Line::from([(0., 0.), (100.0, 100.0)]);
                let p: Point<f32> = $p.into();
                let should_be: Closest<f32> = $should_be;

                let got = line.closest_point(&p);
                assert_eq!(got, should_be);
            }
        };
    }

    closest!(intersects: start_point, (0.0, 0.0));
    closest!(intersects: end_point, (100.0, 100.0));
    closest!(intersects: mid_point, (50.0, 50.0));
    closest!(in_line_far_away, (1000.0, 1000.0) => Closest::SinglePoint(Point::new(100.0, 100.0)));
    closest!(perpendicular_from_50_50, (0.0, 100.0) => Closest::SinglePoint(Point::new(50.0, 50.0)));

    fn single_point<F: GeoFloat>(closest: Closest<F>) -> Point<F> {
        let Closest::SinglePoint(point) = closest else {
            panic!("expected a single closest point, got {closest:?}");
        };
        point
    }

    fn intersection<F: GeoFloat>(closest: Closest<F>) -> Point<F> {
        let Closest::Intersection(point) = closest else {
            panic!("expected an intersection, got {closest:?}");
        };
        point
    }

    #[test]
    fn tiny_axis_aligned_line_has_finite_closest_points() {
        let line = wkt!(LINE(0.0 0.0,0.0 1e-162));
        for endpoint in [line.start_point(), line.end_point()] {
            assert_eq!(
                line.closest_point(&endpoint),
                Closest::Intersection(endpoint)
            );
        }
        let closest = single_point(line.closest_point(&wkt!(POINT(1e-162 5e-163))));
        assert_relative_eq!(
            closest,
            wkt!(POINT(0.0 5e-163)),
            epsilon = 0.0,
            max_relative = 1e-15
        );
    }

    #[test]
    fn tiny_diagonal_line_has_finite_closest_points() {
        let line = wkt!(LINE(0.0 0.0,1e-162 1e-162));
        for endpoint in [line.start_point(), line.end_point()] {
            assert_eq!(
                line.closest_point(&endpoint),
                Closest::Intersection(endpoint)
            );
        }
        let closest = single_point(line.closest_point(&wkt!(POINT(-1e-162 2e-162))));
        assert_relative_eq!(
            closest,
            wkt!(POINT(5e-163 5e-163)),
            epsilon = 0.0,
            max_relative = 1e-15
        );
    }

    // The squared length is subnormal and keeps only two significant bits.
    #[test]
    fn subnormal_squared_length_projects_accurately() {
        let line = wkt!(LINE(0.0 0.0,0.0 3e-162));
        let closest = intersection(line.closest_point(&wkt!(POINT(0.0 2e-162))));
        assert_relative_eq!(
            closest,
            wkt!(POINT(0.0 2e-162)),
            epsilon = 0.0,
            max_relative = 1e-15
        );
    }

    // Squared length overflows to +inf, so the ordinary path is inf / inf.
    #[test]
    fn long_line_has_finite_closest_points() {
        let line = wkt!(LINE(0.0 0.0,1e200 1e200));
        let closest = single_point(line.closest_point(&wkt!(POINT(1e200 0.0))));
        assert_relative_eq!(closest, wkt!(POINT(5e199 5e199)), max_relative = 1e-15);
    }

    // 1e160 is finite; its square is not.
    #[test]
    fn line_with_overflowing_squared_length_projects_in_range() {
        let line = wkt!(LINE(0.0 0.0,1e160 0.0));
        let closest = single_point(line.closest_point(&wkt!(POINT(5e159 1.0))));
        assert_relative_eq!(closest, wkt!(POINT(5e159 0.0)), max_relative = 1e-15);
    }

    #[test]
    fn line_with_infinite_length_start_intersection() {
        let line: Line<f64> = wkt!(LINE(0.0 0.0,1.3e308 1.3e308));
        let start = line.start_point();
        assert!(Euclidean.length(&line).is_infinite());
        assert_eq!(line.closest_point(&start), Closest::Intersection(start));
    }

    // The difference of the endpoints overflows, but the distance does not.
    #[test]
    fn line_with_overflowing_direction_projects_in_range() {
        let line = wkt!(LINE(-1e308 0.0,1e308 0.0));
        let point = wkt!(POINT(0.0 1.0));
        let closest = single_point(line.closest_point(&point));
        assert_eq!(closest, wkt!(POINT(0.0 0.0)));
        assert_relative_eq!(
            Euclidean.distance(&point, &closest),
            Euclidean.distance(&point, &line)
        );

        let line: Line<f32> = wkt!(LINE(-3e38 0.0,3e38 0.0));
        let closest = single_point(line.closest_point(&wkt!(POINT(0.0 1.0))));
        assert_eq!(closest, wkt!(POINT(0.0 0.0)));
    }

    // The offsets from the start to the points also overflow.
    #[test]
    fn line_with_overflowing_offset_projects_in_range() {
        let line = wkt!(LINE(-1.5e308 0.0,1.5e308 0.0));
        let closest = single_point(line.closest_point(&wkt!(POINT(6e307 1.0))));
        assert_relative_eq!(closest, wkt!(POINT(6e307 0.0)), max_relative = 1e-15);
        assert_eq!(
            line.closest_point(&wkt!(POINT(1.7e308 0.0))),
            Closest::SinglePoint(line.end_point())
        );

        let line = wkt!(LINE(1.5e308 0.0,-1.5e308 0.0));
        assert_eq!(
            line.closest_point(&wkt!(POINT(1.7e308 5.0))),
            Closest::SinglePoint(line.start_point())
        );
    }

    // (-4, 3) * 2^54 is perpendicular to (3, 4).
    #[test]
    fn line_projection_preserves_cancellation() {
        let line = wkt!(LINE(0.0 0.0,3.0 4.0));
        let point = wkt!(POINT(-72057594037927936.0 54043195528445952.0));
        assert_eq!(
            line.closest_point(&point),
            Closest::SinglePoint(line.start_point())
        );
    }

    // The direction is (3, 4) * 2^-550 and the point is (4, -3) * 2^54 + (-64, 56).
    #[test]
    fn tiny_line_preserves_nonzero_projection_clamping() {
        let line = wkt!(LINE(0.0 0.0,8.139985654852579e-166 1.0853314206470105e-165));
        let point = wkt!(POINT(72057594037927872.0 - 54043195528445896.0));
        assert_eq!(
            line.closest_point(&point),
            Closest::SinglePoint(line.end_point())
        );
    }

    #[test]
    fn tiny_f32_line_has_finite_closest_points() {
        let line: Line<f32> = wkt!(LINE(0.0 0.0,0.0 1e-23));
        for endpoint in [line.start_point(), line.end_point()] {
            assert_eq!(
                line.closest_point(&endpoint),
                Closest::Intersection(endpoint)
            );
        }
        assert_eq!(
            line.closest_point(&wkt!(POINT(1e-23 5e-24))),
            Closest::SinglePoint(wkt!(POINT(0.0 5e-24)))
        );
    }

    #[test]
    fn tiny_unequal_component_line_clamps_in_both_directions() {
        let start = wkt!(POINT(0.0 0.0));
        let end = wkt!(POINT(3e-163 4e-163));
        let lines = [
            wkt!(LINE(0.0 0.0,3e-163 4e-163)),
            wkt!(LINE(3e-163 4e-163,0.0 0.0)),
        ];
        for line in lines {
            for endpoint in [start, end] {
                assert_eq!(
                    line.closest_point(&endpoint),
                    Closest::Intersection(endpoint)
                );
            }
            assert_eq!(
                line.closest_point(&wkt!(POINT(-3e-163 - 4e-163))),
                Closest::SinglePoint(start)
            );
            assert_eq!(
                line.closest_point(&wkt!(POINT(6e-163 8e-163))),
                Closest::SinglePoint(end)
            );

            // The offset (-4, 3) is perpendicular to the direction (3, 4).
            let closest = single_point(line.closest_point(&wkt!(POINT(-2.5e-163 5e-163))));
            assert_relative_eq!(
                closest,
                wkt!(POINT(1.5e-163 2e-163)),
                epsilon = 0.0,
                max_relative = 1e-15
            );
        }
    }

    // The direction is (3, 4) * 2^-550 and the points are (-4, 3) * 2^54 and
    // (-4, 3) * 2^1021.
    #[test]
    fn tiny_line_preserves_perpendicular_projection() {
        let line = wkt!(LINE(0.0 0.0,8.139985654852579e-166 1.0853314206470105e-165));
        for point in [
            wkt!(POINT(-72057594037927936.0 54043195528445952.0)),
            wkt!(POINT(-8.98846567431158e307 6.741349255733685e307)),
        ] {
            assert_eq!(
                line.closest_point(&point),
                Closest::SinglePoint(line.start_point())
            );
        }
    }

    // The direction is (1, 41) * 2^-90 and the point is (-41, 1) * 2^24.
    #[test]
    fn tiny_f32_line_preserves_perpendicular_projection() {
        let line: Line<f32> = wkt!(LINE(0.0 0.0,8.077936e-28 3.3119536e-26));
        let point = wkt!(POINT(-687865856.0 16777216.0));
        assert_eq!(
            line.closest_point(&point),
            Closest::SinglePoint(line.start_point())
        );
    }

    // The direction is (1.5 + 2^-26, 1.5 - 2^-26) * 2^-900. The point is
    // perpendicular to it at 2^54 times its length, with y increased by one ULP.
    // The exact dot product is positive, but independently rounded products
    // cancel.
    #[test]
    fn tiny_line_preserves_near_perpendicular_projection() {
        let line = wkt!(LINE(0.0 0.0,1.7745782968790134e-271 1.7745782616213107e-271));
        let point = wkt!(POINT(-2.702159749578752e16 2.7021598032658436e16));
        assert_eq!(point.dot(Point::from(line.end)), 0.0);
        assert_eq!(
            line.closest_point(&point),
            Closest::SinglePoint(line.end_point())
        );
        assert_eq!(
            line.closest_point(&(-point)),
            Closest::SinglePoint(line.start_point())
        );
    }

    #[test]
    fn subnormal_line_preserves_points_and_endpoint_clamping() {
        let q = f64::from_bits(1);
        let lines = [
            wkt!(LINE(0.0 0.0,0.0 1.5e-323)),
            wkt!(LINE(0.0 1.5e-323,0.0 0.0)),
        ];
        for line in lines {
            for step in -1..=4 {
                let y = f64::from(step) * q;
                let expected = Point::new(0.0, f64::from(step.clamp(0, 3)) * q);
                for x in [-f64::MAX, -12.0 * q, 0.0, 12.0 * q, f64::MAX] {
                    let closest = if x == 0.0 && (0..=3).contains(&step) {
                        Closest::Intersection(expected)
                    } else {
                        Closest::SinglePoint(expected)
                    };
                    assert_eq!(line.closest_point(&Point::new(x, y)), closest);
                }
            }
        }
    }

    #[test]
    fn subnormal_f32_line_preserves_points_and_endpoint_clamping() {
        let q = f32::from_bits(1);
        let lines: [Line<f32>; 2] = [wkt!(LINE(0.0 0.0,0.0 4e-45)), wkt!(LINE(0.0 4e-45,0.0 0.0))];
        for line in lines {
            for step in -1_i16..=4 {
                let y = f32::from(step) * q;
                let expected = Point::new(0.0, f32::from(step.clamp(0, 3)) * q);
                for x in [-f32::MAX, -12.0 * q, 0.0, 12.0 * q, f32::MAX] {
                    let closest = if x == 0.0 && (0..=3).contains(&step) {
                        Closest::Intersection(expected)
                    } else {
                        Closest::SinglePoint(expected)
                    };
                    assert_eq!(line.closest_point(&Point::new(x, y)), closest);
                }
            }
        }
    }

    // wkt! cannot express NaN.
    #[test]
    fn nan_input_gives_nan_closest_point() {
        let nan = f64::NAN;
        let line = wkt!(LINE(0.0 0.0,1.0 1.0));
        let cases = [
            (Line::new((0.0, 0.0), (nan, 0.0)), Point::new(1.0, 0.0)),
            (line, Point::new(nan, 5.0)),
            (line, Point::new(nan, 0.0)),
        ];
        for (line, point) in cases {
            let closest = single_point(line.closest_point(&point));
            assert!(
                closest.x().is_nan() && closest.y().is_nan(),
                "{line:?} {point:?}: {closest:?}"
            );
        }
    }

    fn a_square(width: f32) -> LineString<f32> {
        LineString::from(vec![
            (0.0, 0.0),
            (width, 0.0),
            (width, width),
            (0.0, width),
            (0.0, 0.0),
        ])
    }

    #[test]
    fn zero_length_line_is_indeterminate() {
        let line: Line<f32> = Line::from([(0.0, 0.0), (0.0, 0.0)]);
        let p: Point<f32> = Point::new(100.0, 100.0);
        let should_be: Closest<f32> = Closest::Indeterminate;

        let got = line.closest_point(&p);
        assert_eq!(got, should_be);
    }

    #[test]
    fn line_string_with_single_element_behaves_like_line() {
        let points = vec![(0.0, 0.0), (100.0, 100.0)];
        let line_string = LineString::<f32>::from(points.clone());
        let line = Line::new(points[0], points[1]);

        let some_random_points = vec![
            point!(x: 0.0, y: 0.0),
            point!(x: 100.0, y: 100.0),
            point!(x: 1000.0, y: 1000.0),
            point!(x: 100.0, y: 0.0),
            point!(x: 50.0, y: 50.0),
            point!(x: 1234.567, y: -987.6543),
        ];

        for p in some_random_points {
            assert_eq!(
                line_string.closest_point(&p),
                line.closest_point(&p),
                "closest point to: {p:?}",
            );
        }
    }

    #[test]
    fn empty_line_string_is_indeterminate() {
        let ls = LineString::empty();
        let p = Point::new(0.0, 0.0);

        let got = ls.closest_point(&p);
        assert_eq!(got, Closest::Indeterminate);
    }

    /// A polygon with 2 holes in it.
    fn holy_polygon() -> Polygon<f32> {
        let square: LineString<f32> = a_square(100.0);
        let ring_1 = a_square(20.0).translate(10.0, 10.0);
        let ring_2 = a_square(10.0).translate(70.0, 60.0);
        Polygon::new(square, vec![ring_1, ring_2])
    }

    #[test]
    fn polygon_without_rings_and_point_outside_is_same_as_linestring() {
        let poly = holy_polygon();
        let p = Point::new(1000.0, 12345.678);
        assert!(
            !poly.exterior().contains(&p),
            "`p` should be outside the polygon!"
        );

        let poly_closest = poly.closest_point(&p);
        let exterior_closest = poly.exterior().closest_point(&p);

        assert_eq!(poly_closest, exterior_closest);
    }

    #[test]
    fn polygon_with_point_on_interior_ring() {
        let poly = holy_polygon();
        let p = poly.interiors()[0][3];
        let should_be = Closest::Intersection(p.into());

        let got = poly.closest_point(&p.into());

        assert_eq!(got, should_be);
    }

    #[test]
    fn polygon_with_point_near_interior_ring() {
        let poly = holy_polygon();
        let p = point!(x: 17.0, y: 33.0);
        assert!(poly.intersects(&p), "sanity check");

        assert_eq!(Closest::Intersection(p), poly.closest_point(&p));
    }

    #[test]
    fn polygon_with_interior_point() {
        let square = polygon![
            (x: 0.0, y: 0.0),
            (x: 10.0, y: 0.0),
            (x: 10.0, y: 10.0),
            (x: 0.0, y: 10.0)
        ];
        let result = square.closest_point(&point!(x: 1.0, y: 2.0));

        // the point is within the square, so the closest point should be the point itself.
        assert_eq!(result, Closest::Intersection(point!(x: 1.0, y: 2.0)));
    }

    #[test]
    fn multi_polygon_with_internal_and_external_points() {
        use crate::{point, polygon};

        let square_1 = polygon![
            (x: 0.0, y: 0.0),
            (x: 1.0, y: 0.0),
            (x: 1.0, y: 1.0),
            (x: 0.0, y: 1.0)
        ];
        use crate::Translate;
        let square_10 = square_1.translate(10.0, 10.0);
        let square_50 = square_1.translate(50.0, 50.0);

        let multi_polygon = MultiPolygon::new(vec![square_1, square_10, square_50]);
        let result = multi_polygon.closest_point(&point!(x: 8.0, y: 8.0));
        assert_eq!(result, Closest::SinglePoint(point!(x: 10.0, y: 10.0)));

        let result = multi_polygon.closest_point(&point!(x: 10.5, y: 10.5));
        assert_eq!(result, Closest::Intersection(point!(x: 10.5, y: 10.5)));
    }
}
