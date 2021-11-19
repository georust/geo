use crate::coords_iter::CoordsIter;
use crate::prelude::*;
use crate::{
    Closest, Coordinate, GeoFloat, Geometry, GeometryCollection, Line, LineString, MultiLineString,
    MultiPoint, MultiPolygon, Point, Polygon, Rect, Triangle,
};
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
/// # use geo::algorithm::closest_point::ClosestPoint;
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

impl<'a, F, C> ClosestPoint<F> for &'a C
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
        let line_length = self.euclidean_length();
        if line_length == F::zero() {
            // if we've got a zero length line, technically the entire line
            // is the closest point...
            return Closest::Indeterminate;
        }

        // For some line AB, there is some point, C, which will be closest to
        // P. The line AB will be perpendicular to CP.
        //
        // Line equation: P = start + t * (end - start)

        let direction_vector = Point(self.end - self.start);
        let to_p = Point(p.0 - self.start);

        let t = to_p.dot(direction_vector) / direction_vector.dot(direction_vector);

        // check the cases where the closest point is "outside" the line
        if t < F::zero() {
            return Closest::SinglePoint(self.start.into());
        } else if t > F::one() {
            return Closest::SinglePoint(self.end.into());
        }

        let x = direction_vector.x();
        let y = direction_vector.y();
        let c = Point(self.start + (t * x, t * y).into());

        if self.intersects(p) {
            Closest::Intersection(c)
        } else {
            Closest::SinglePoint(c)
        }
    }
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

impl<F: GeoFloat> ClosestPoint<F> for Coordinate<F> {
    fn closest_point(&self, p: &Point<F>) -> Closest<F> {
        Point(*self).closest_point(p)
    }
}

impl<F: GeoFloat> ClosestPoint<F> for Triangle<F> {
    fn closest_point(&self, p: &Point<F>) -> Closest<F> {
        if self.intersects(p) {
            return Closest::Intersection(*p);
        }
        closest_of(&self.to_lines(), *p)
    }
}

impl<F: GeoFloat> ClosestPoint<F> for Rect<F> {
    fn closest_point(&self, p: &Point<F>) -> Closest<F> {
        if self.intersects(p) {
            return Closest::Intersection(*p);
        }
        closest_of(&self.to_lines(), *p)
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
                "closest point to: {:?}",
                p
            );
        }
    }

    #[test]
    fn empty_line_string_is_indeterminate() {
        let ls: LineString<f32> = LineString(Vec::new());
        let p = Point::new(0.0, 0.0);

        let got = ls.closest_point(&p);
        assert_eq!(got, Closest::Indeterminate);
    }

    /// A polygon with 2 holes in it.
    fn holy_polygon() -> Polygon<f32> {
        let square: LineString<f32> = a_square(100.0);
        let ring_1 = a_square(20.0).translate(10.0, 10.0);
        let ring_2 = a_square(10.0).translate(70.0, 60.0);
        Polygon::new(square.clone(), vec![ring_1, ring_2])
    }

    #[test]
    fn polygon_without_rings_and_point_outside_is_same_as_linestring() {
        let poly = holy_polygon();
        let p = Point::new(1000.0, 12345.6789);
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
        let p = poly.interiors()[0].0[3];
        let should_be = Closest::Intersection(p.into());

        let got = poly.closest_point(&p.into());

        assert_eq!(got, should_be);
    }

    #[test]
    fn polygon_with_point_near_interior_ring() {
        let poly = holy_polygon();
        let random_ring_corner = poly.interiors()[0].0[3];
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
        use crate::translate::Translate;
        let square_10 = square_1.translate(10.0, 10.0);
        let square_50 = square_1.translate(50.0, 50.0);

        let multi_polygon = MultiPolygon(vec![square_1, square_10, square_50]);
        let result = multi_polygon.closest_point(&point!(x: 8.0, y: 8.0));
        assert_eq!(result, Closest::SinglePoint(point!(x: 10.0, y: 10.0)));

        let result = multi_polygon.closest_point(&point!(x: 10.5, y: 10.5));
        assert_eq!(result, Closest::Intersection(point!(x: 10.5, y: 10.5)));
    }
}
