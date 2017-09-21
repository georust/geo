use num_traits::Float;
use prelude::*;
use {Line, LineString, Point, Polygon};

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum Closest<F: Float> {
    Intersection(Point<F>),
    SinglePoint(Point<F>),
    Indeterminate,
}

impl<F: Float> Closest<F> {
    fn best_of_two(&self, other: &Self, p: &Point<F>) -> Self {
        let left = match *self {
            Closest::Indeterminate => return *other,
            Closest::Intersection(_) => return *self,
            Closest::SinglePoint(l) => l,
        };
        let right = match *other {
            Closest::Indeterminate => return *self,
            Closest::Intersection(_) => return *other,
            Closest::SinglePoint(r) => r,
        };

        unimplemented!()
    }
}

/// Find the closest point between two objects, where the other object is
/// assumed to be a `Point` by default.
///
/// # Examples
///
/// Here's a simple example where we've got a horizontal line which goes
/// through `(-50, 0) -> (50, 0)` and want to find the closest point to
/// `(0, 100)`. If you draw it out on paper the point on the line which is
/// closest to `(0, 100)` will be the origin.
///
/// ```rust
/// # use geo::algorithm::closest_point::{Closest, ClosestPoint};
/// # use geo::{Point, Line};
/// let p: Point<f32> = Point::new(0.0, 100.0);
/// let horizontal_line: Line<f32> = Line::new(Point::new(-50.0, 0.0), Point::new(50.0, 0.0));
///
/// let closest = horizontal_line.closest_point(&p);
/// assert_eq!(closest, Closest::SinglePoint(Point::new(0.0, 0.0)));
/// ```
pub trait ClosestPoint<F: Float, Rhs = Point<F>> {
    /// Find the closest point between `self` and `p`.
    fn closest_point(&self, p: &Rhs) -> Closest<F>;
}

impl<'a, F, C> ClosestPoint<F> for &'a C
where
    C: ClosestPoint<F>,
    F: Float,
{
    fn closest_point(&self, p: &Point<F>) -> Closest<F> {
        (*self).closest_point(p)
    }
}

impl<F: Float> ClosestPoint<F> for Point<F> {
    fn closest_point(&self, p: &Self) -> Closest<F> {
        if self == p {
            Closest::Intersection(*self)
        } else {
            Closest::SinglePoint(*self)
        }
    }
}


impl<F: Float> ClosestPoint<F> for Line<F> {
    fn closest_point(&self, p: &Point<F>) -> Closest<F> {
        let line_length = self.length();
        if line_length == F::zero() {
            // if we've got a zero length line, technically the entire line
            // is the closest point...
            return Closest::Indeterminate;
        }

        // For some line AB, there is some point, C, which will be closest to
        // P. The line AB will be perpendicular to CP.
        //
        // Line equation: P = start + t * (end - start)

        let direction_vector = self.end - self.start;
        let to_p = *p - self.start;

        let t = to_p.dot(&direction_vector) / direction_vector.dot(&direction_vector);

        // check the cases where the closest point is "outside" the line
        if t < F::zero() {
            return Closest::SinglePoint(self.start);
        } else if t > F::one() {
            return Closest::SinglePoint(self.end);
        }

        let (x, y) = direction_vector.coords();
        let displacement = Point::new(t * x, t * y);
        let c = self.start + displacement;

        if self.contains(p) {
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
fn closest_of<C, F, I>(iter: I, p: &Point<F>) -> Closest<F>
where
    F: Float,
    I: IntoIterator<Item = C>,
    C: ClosestPoint<F>,
{
    let mut single_points = Vec::new();

    for line_segment in iter {
        let closest = line_segment.closest_point(p);
        match closest {
            Closest::Intersection(_) => return closest,
            Closest::SinglePoint(close) => single_points.push(close),
            _ => {},
        }
    }

    single_points
        .into_iter()
        .min_by(|l, r| {
            l.distance(p)
                .partial_cmp(&r.distance(p))
                .expect("Should never get NaN")
        })
        .map(|closest_point| Closest::SinglePoint(closest_point))
        .unwrap_or(Closest::Indeterminate)
}

impl<F: Float> ClosestPoint<F> for LineString<F> {
    fn closest_point(&self, p: &Point<F>) -> Closest<F> {
        closest_of(self.lines(), p)
    }
}

impl<F: Float> ClosestPoint<F> for Polygon<F> {
    fn closest_point(&self, p: &Point<F>) -> Closest<F> {
        let closest_of_interior = closest_of(self.interiors.iter(), p);
        if let Closest::Intersection(_) = closest_of_interior {
            return closest_of_interior;
        }

        let initial_guess = self.exterior.closest_point(p);

        let interior = match closest_of_interior {
            Closest::Indeterminate => return initial_guess,
            Closest::SinglePoint(interior) => interior,
            Closest::Intersection(_) => unreachable!(),
        };

        let exterior = match initial_guess {
            Closest::Intersection(_) => return initial_guess,
            Closest::SinglePoint(exterior) => exterior,
            Closest::Indeterminate => unreachable!(),
        };

        if exterior.distance(p) <= interior.distance(p) {
            initial_guess
        } else {
            closest_of_interior
        }
    }
}


#[cfg(test)]
mod tests {
    use super::*;
    use algorithm::orient::Direction;

    /// Create a test which checks that we get `$should_be` when trying to find
    /// the closest distance between `$p` and the line `(0, 0) -> (100, 100)`.
    macro_rules! closest {
        (intersects: $name:ident, $p:expr) => {
            closest!($name, $p => Closest::Intersection($p.into()));
        };
        ($name:ident, $p:expr => $should_be:expr) => {
            #[test]
            fn $name() {
                let line: Line<f32> = Line::new(Point::new(0.0, 0.0), Point::new(100.0, 100.0));
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


    fn collect_points<F, P, C>(points: &[P]) -> C
    where
        F: Float,
        P: Into<Point<F>> + Clone,
        C: ::std::iter::FromIterator<Point<F>>,
    {
        points.iter().map(|p| p.clone().into()).collect()
    }

    fn a_square(width: f32) -> LineString<f32> {
        let points = vec![
            (0.0, 0.0),
            (width, 0.0),
            (width, width),
            (0.0, width),
            (0.0, 0.0),
        ];

        collect_points(&points)
    }

    fn random_looking_points() -> Vec<Point<f32>> {
        let mut points = vec![
            (0.0, 0.0),
            (100.0, 100.0),
            (1000.0, 1000.0),
            (100.0, 0.0),
            (50.0, 50.0),
            (1234.567, -987.6543),
        ];

        collect_points(&points)
    }

    fn fuzz_two_impls<A, B>(left: A, right: B)
    where
        A: ClosestPoint<f32>,
        B: ClosestPoint<f32>,
    {
        let some_random_points = random_looking_points();

        for (i, random_point) in some_random_points.into_iter().enumerate() {
            let p: Point<_> = random_point.into();

            let got_from_left = left.closest_point(&p);
            let got_from_right = right.closest_point(&p);

            assert_eq!(
                got_from_left,
                got_from_right,
                "{}: {:?} got {:?} and {:?}",
                i,
                p,
                got_from_left,
                got_from_right
            );
        }
    }

    #[test]
    fn zero_length_line_is_indeterminate() {
        let line: Line<f32> = Line::new(Point::new(0.0, 0.0), Point::new(0.0, 0.0));
        let p: Point<f32> = Point::new(100.0, 100.0);
        let should_be: Closest<f32> = Closest::Indeterminate;

        let got = line.closest_point(&p);
        assert_eq!(got, should_be);
    }

    #[test]
    fn line_string_with_single_element_behaves_like_line() {
        let points = vec![(0.0, 0.0), (100.0, 100.0)];
        let line_string: LineString<f32> = collect_points(&points);
        let line = Line::new(points[0].into(), points[1].into());

        fuzz_two_impls(line, line_string);
    }

    #[test]
    fn empty_line_string_is_indeterminate() {
        let ls: LineString<f32> = LineString(Vec::new());
        let p = Point::new(0.0, 0.0);

        let got = ls.closest_point(&p);
        assert_eq!(got, Closest::Indeterminate);
    }

    #[test]
    fn simple_polygon_is_same_as_linestring() {
        let square: LineString<f32> = a_square(100.0);
        let poly = Polygon::new(square.clone(), Vec::new());

        fuzz_two_impls(square, poly);
    }

    /// A polygon with 2 holes in it.
    fn holy_polygon() -> Polygon<f32> {
        let square: LineString<f32> = a_square(100.0);
        let ring_1 = a_square(20.0).translate(20.0, 10.0);
        let ring_2 = a_square(10.0).translate(70.0, 60.0);
        Polygon::new(square.clone(), vec![ring_1, ring_2])
    }

    #[test]
    fn polygon_without_rings_and_point_outside_is_same_as_linestring() {
        let poly = holy_polygon();
        let p = Point::new(1000.0, 12345.6789);
        assert!(!poly.exterior.contains(&p), "`p` should be outside the polygon!");

        let poly_closest = poly.closest_point(&p);
        let exterior_closest = poly.exterior.closest_point(&p);

        assert_eq!(poly_closest, exterior_closest);
    }

    #[test]
    fn polygon_with_point_on_interior_ring() {
        let poly = holy_polygon();
        let p = poly.interiors[0].0[3];
        let should_be = Closest::Intersection(p);

        let got = poly.closest_point(&p);

        assert_eq!(got, should_be);
    }

    #[test]
    fn polygon_with_point_near_interior_ring() {
        let poly = holy_polygon();
        let random_ring_corner = poly.interiors[0].0[3];
        let p = random_ring_corner.translate(-3.0, 3.0);

        let should_be = Closest::SinglePoint(random_ring_corner);
        println!("{:?} {:?}", p, random_ring_corner);

        let got = poly.closest_point(&p);

        assert_eq!(got, should_be);
    }
}
