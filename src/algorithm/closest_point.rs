use num_traits::Float;
use prelude::*;
use {Line, LineString, Point};

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum Closest<F: Float> {
    Intersection(Point<F>),
    SinglePoint(Point<F>),
    Indeterminate,
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

impl<F: Float> ClosestPoint<F> for LineString<F> {
    fn closest_point(&self, p: &Point<F>) -> Closest<F> {
        let mut single_points = Vec::new();

        for line_segment in self.lines() {
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
}


#[cfg(test)]
mod tests {
    use super::*;

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

    #[test]
    fn zero_length_line_is_indeterminate() {
        let line: Line<f32> = Line::new(Point::new(0.0, 0.0), Point::new(0.0, 0.0));
        let p: Point<f32> = Point::new(100.0, 100.0);
        let should_be: Closest<f32> = Closest::Indeterminate;

        let got = line.closest_point(&p);
        assert_eq!(got, should_be);
    }

    fn points_to_line<F, P>(points: &[P]) -> LineString<F>
    where
        F: Float,
        P: Into<Point<F>> + Clone,
    {
        points.iter().map(|p| p.clone().into()).collect()
    }

    #[test]
    fn line_string_with_single_element_behaves_like_line() {
        let points = vec![(0.0, 0.0), (100.0, 100.0)];
        let line_string = points_to_line(&points);
        let line = Line::new(points[0].into(), points[1].into());

        let some_random_points = vec![
            (0.0, 0.0),
            (100.0, 100.0),
            (50.0, 50.0),
            (1000.0, 1000.0),
            (100.0, 0.0),
        ];

        for (i, random_point) in some_random_points.into_iter().enumerate() {
            let p: Point<f32> = random_point.into();

            let got_from_line = line.closest_point(&p);
            let got_from_line_string = line_string.closest_point(&p);

            assert_eq!(got_from_line, got_from_line_string, "{}: {:?}", i, p);
        }
    }

    #[test]
    fn empty_line_string_is_indeterminate() {
        let ls: LineString<f32> = LineString(Vec::new());
        let p = Point::new(0.0, 0.0);

        let got = ls.closest_point(&p);
        assert_eq!(got, Closest::Indeterminate);
    }
}
