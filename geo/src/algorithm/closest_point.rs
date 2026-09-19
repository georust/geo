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
        let t = if (squared_length == F::zero() && numerator == F::zero())
            || !squared_length.is_finite()
            || !numerator.is_finite()
        {
            // Avoid 0/0 from underflow and inf/inf from overflow, but preserve a
            // non-zero numerator over a zero squared length so that infinite
            // projections still clamp to an endpoint.
            scaled_projection(direction_vector, to_p)
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
        let c = Point::from(self.start + (t * x, t * y).into());

        if self.intersects(p) {
            Closest::Intersection(c)
        } else {
            Closest::SinglePoint(c)
        }
    }
}

// For a positive, finite magnitude, return the largest power of two no greater
// than it, together with its exponent. Strip redundant significand zeros before
// converting back to F: integer_decode may use a wider representation than F.
fn projection_scale<F: GeoFloat>(magnitude: F) -> (F, i32) {
    let (mantissa, exponent, _) = magnitude.integer_decode();
    let trailing = mantissa.trailing_zeros();
    let reduced = mantissa >> trailing;
    let leading = u64::BITS - 1 - reduced.leading_zeros();
    let fraction = F::from(reduced).unwrap() / F::from(1_u64 << leading).unwrap();
    (
        magnitude / fraction,
        i32::from(exponent) + trailing as i32 + leading as i32,
    )
}

// Compensate for the rounded product when the two dot-product terms cancel.
fn compensated_dot<F: GeoFloat>(a: Point<F>, b: Point<F>) -> F {
    let product = a.y() * b.y();
    let correction = a.y().mul_add(b.y(), -product);
    a.x().mul_add(b.x(), product) + correction
}

// Restore the projection's exponent only as far as endpoint comparisons need.
// Negative values need only retain their sign; 2 means beyond the end, not an
// exact projection. Apply powers of two without constructing an infinite factor.
fn rescale_projection<F: GeoFloat>(mut value: F, mut exponent: i32) -> F {
    if value <= F::zero() {
        return value;
    }
    let two = F::one() + F::one();
    while exponent > 0 && value < F::one() {
        value = value + value;
        exponent -= 1;
    }
    if exponent > 0 {
        return two;
    }
    while exponent < 0 && value != F::zero() {
        value = value / two;
        exponent += 1;
    }
    value
}

// Called only for a non-zero direction whose original projection is 0/0.
fn scaled_projection<F: GeoFloat>(direction: Point<F>, to_point: Point<F>) -> F {
    let zero = F::zero();
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
    let (direction_scale, direction_exponent) =
        projection_scale(direction.x().abs().max(direction.y().abs()));
    let (query_scale, query_exponent) = projection_scale(query_magnitude);
    // Scale independently by powers of two, preserving component ratios while
    // keeping both dot products in range. Use the same compensated dot product
    // in the denominator so an end-point query still gives exactly t == 1.
    let direction = direction / direction_scale;
    let query = to_point / query_scale;
    let numerator = compensated_dot(query, direction);
    let denominator = compensated_dot(direction, direction);
    rescale_projection(numerator / denominator, query_exponent - direction_exponent)
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
    use crate::algorithm::{Contains, Translate};
    use crate::{point, polygon};

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

    fn assert_finite(closest: Closest<f64>) {
        let point = match closest {
            Closest::Intersection(point) | Closest::SinglePoint(point) => point,
            Closest::Indeterminate => panic!("expected a finite closest point"),
        };

        assert!(
            point.x().is_finite(),
            "non-finite x coordinate: {closest:?}"
        );
        assert!(
            point.y().is_finite(),
            "non-finite y coordinate: {closest:?}"
        );
    }

    #[test]
    fn tiny_axis_aligned_line_has_finite_closest_points() {
        let tiny = 1e-162;
        let start = Point::new(0.0, 0.0);
        let end = Point::new(0.0, tiny);
        let line = Line::new(start, end);

        let at_start = line.closest_point(&start);
        assert_finite(at_start);
        assert_eq!(at_start, Closest::Intersection(start));

        let at_end = line.closest_point(&end);
        assert_finite(at_end);
        assert_eq!(at_end, Closest::Intersection(end));

        let off_segment = Point::new(tiny, tiny / 2.0);
        let closest = line.closest_point(&off_segment);
        assert_finite(closest);
        let Closest::SinglePoint(closest) = closest else {
            panic!("expected a single closest point, got {closest:?}");
        };
        assert_eq!(closest.x(), 0.0);
        assert_relative_eq!(closest.y() / tiny, 0.5, max_relative = 1e-15);
    }

    #[test]
    fn tiny_diagonal_line_has_finite_closest_points() {
        let tiny = 1e-162;
        let start = Point::new(0.0, 0.0);
        let end = Point::new(tiny, tiny);
        let line = Line::new(start, end);

        let at_start = line.closest_point(&start);
        assert_finite(at_start);
        assert_eq!(at_start, Closest::Intersection(start));

        let at_end = line.closest_point(&end);
        assert_finite(at_end);
        assert_eq!(at_end, Closest::Intersection(end));

        let off_segment = Point::new(-tiny, 2.0 * tiny);
        let closest = line.closest_point(&off_segment);
        assert_finite(closest);
        let Closest::SinglePoint(closest) = closest else {
            panic!("expected a single closest point, got {closest:?}");
        };
        assert_relative_eq!(closest.x() / tiny, 0.5, max_relative = 1e-15);
        assert_relative_eq!(closest.y() / tiny, 0.5, max_relative = 1e-15);
    }

    #[test]
    fn line_with_infinite_length_start_intersection() {
        let line = Line::<f64>::new((0.0, 0.0), (1.3e308, 1.3e308));
        let start = line.start_point();
        assert!(Euclidean.length(&line).is_infinite());
        assert_eq!(line.closest_point(&start), Closest::Intersection(start));
    }

    #[test]
    fn line_projection_preserves_cancellation() {
        let line = Line::<f64>::new((0.0, 0.0), (3.0, 4.0));
        let k = 2.0_f64.powi(54);
        let point = Point::new(-4.0 * k, 3.0 * k);
        assert_eq!(
            line.closest_point(&point),
            Closest::SinglePoint(line.start_point())
        );
    }

    #[test]
    fn tiny_line_preserves_nonzero_projection_clamping() {
        let tiny = 2.0_f64.powi(-550);
        let k = 2.0_f64.powi(54);
        let line = Line::new((0.0, 0.0), (3.0 * tiny, 4.0 * tiny));
        let point = Point::new(4.0 * k - 64.0, -3.0 * k + 56.0);
        assert_eq!(
            line.closest_point(&point),
            Closest::SinglePoint(line.end_point())
        );
    }

    #[test]
    fn tiny_f32_line_has_finite_closest_points() {
        let tiny = 1e-23_f32;
        let line = Line::new((0.0, 0.0), (0.0, tiny));
        for endpoint in [line.start_point(), line.end_point()] {
            assert_eq!(
                line.closest_point(&endpoint),
                Closest::Intersection(endpoint)
            );
        }
        let point = Point::new(tiny, tiny / 2.0);
        assert_eq!(
            line.closest_point(&point),
            Closest::SinglePoint(Point::new(0.0, tiny / 2.0))
        );
    }

    #[test]
    fn tiny_unequal_component_line_clamps_in_both_directions() {
        let tiny = 1e-163;
        let start = Point::new(0.0, 0.0);
        let end = Point::new(3.0 * tiny, 4.0 * tiny);
        let before_start = Point::new(-3.0 * tiny, -4.0 * tiny);
        let beyond_end = Point::new(6.0 * tiny, 8.0 * tiny);

        for line in [Line::new(start, end), Line::new(end, start)] {
            for endpoint in [start, end] {
                assert_eq!(
                    line.closest_point(&endpoint),
                    Closest::Intersection(endpoint)
                );
            }
            assert_eq!(
                line.closest_point(&before_start),
                Closest::SinglePoint(start)
            );
            assert_eq!(line.closest_point(&beyond_end), Closest::SinglePoint(end));

            // The offset (-4, 3) is perpendicular to the direction (3, 4).
            let point = Point::new(-2.5 * tiny, 5.0 * tiny);
            let closest = line.closest_point(&point);
            assert_finite(closest);
            let Closest::SinglePoint(closest) = closest else {
                panic!("expected a single closest point, got {closest:?}");
            };
            assert_relative_eq!(closest.x() / tiny, 1.5, max_relative = 1e-15);
            assert_relative_eq!(closest.y() / tiny, 2.0, max_relative = 1e-15);
        }
    }

    #[test]
    fn tiny_line_preserves_perpendicular_projection() {
        let tiny = 2.0_f64.powi(-550);
        let line = Line::new((0.0, 0.0), (3.0 * tiny, 4.0 * tiny));
        for k in [2.0_f64.powi(54), 2.0_f64.powi(1021)] {
            let point = Point::new(-4.0 * k, 3.0 * k);
            assert_eq!(
                line.closest_point(&point),
                Closest::SinglePoint(line.start_point())
            );
        }
    }

    #[test]
    fn tiny_f32_line_preserves_perpendicular_projection() {
        let tiny = 2.0_f32.powi(-90);
        let k = 2.0_f32.powi(24);
        let line = Line::new((0.0, 0.0), (tiny, 41.0 * tiny));
        let point = Point::new(-41.0 * k, k);
        assert_eq!(
            line.closest_point(&point),
            Closest::SinglePoint(line.start_point())
        );
    }

    #[test]
    fn tiny_line_preserves_near_perpendicular_projection() {
        let offset = 2.0_f64.powi(-26);
        let a = 1.5 + offset;
        let b = 1.5 - offset;
        let tiny = 2.0_f64.powi(-900);
        let k = 2.0_f64.powi(54);
        let line = Line::new((0.0, 0.0), (a * tiny, b * tiny));
        // Increase the perpendicular query's y by one ULP. The exact dot
        // product is positive, but independently rounded products cancel.
        let point = Point::new(-b * k, (a + f64::EPSILON) * k);
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
        let start = Point::new(0.0, 0.0);
        let end = Point::new(0.0, 3.0 * q);
        for line in [Line::new(start, end), Line::new(end, start)] {
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
        let start = Point::new(0.0, 0.0);
        let end = Point::new(0.0, 3.0 * q);
        for line in [Line::new(start, end), Line::new(end, start)] {
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
