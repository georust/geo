use num_traits::Float;
use prelude::*;
use {Line, Point};

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
    fn closest_point(&self, other: &Rhs) -> Closest<F>;
}

impl<F: Float> ClosestPoint<F> for Point<F> {
    fn closest_point(&self, other: &Self) -> Closest<F> {
        if self == other {
            Closest::Intersection(*self)
        } else {
            Closest::SinglePoint(*self)
        }
    }
}


impl<F: Float> ClosestPoint<F> for Line<F> {
    fn closest_point(&self, other: &Point<F>) -> Closest<F> {
        let line_length = self.length();
        if line_length == F::zero() {
            // if we've got a zero length line, technically the entire line
            // is the closest point...
            return Closest::Indeterminate;
        }

        // For some line AB, there is some point, C, which will be closest to
        // P (`other`). The line AB will be perpendicular to CP.

        let direction_vector = self.end - self.start;
        let to_other = *other - self.start;

        let t = to_other.dot(&direction_vector) / direction_vector.dot(&direction_vector);

        // check the cases where the closest point is "outside" the line
        if t < F::zero() {
            return Closest::SinglePoint(self.start);
        } else if t > F::one() {
            return Closest::SinglePoint(self.end);
        }

        let (x, y) = direction_vector.coords();
        let displacement = Point::new(t * x, t * y);
        let c = self.start + displacement;

        if self.contains(other) {
            Closest::Intersection(c)
        } else {
            Closest::SinglePoint(c)
        }
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
}
