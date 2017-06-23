use types::Point;
use float_ord::FloatOrd;
use num_traits::{Float, ToPrimitive};
use core::cmp::{Eq, Ord, Ordering, PartialOrd};

/// Implement Ord for Point.
/// This ordering is used by the line sweep algorithm to efficiently find intersections of a
/// set of line of line segments. Points are jointly ordered (y DESC, x ASC) that is, from top left
/// to bottom right.

impl<T> Eq for Point<T> where T: Float {}

fn to_float_ord<T>(x: T) -> FloatOrd<f64>
    where T: Float + ToPrimitive
{
    FloatOrd(x.to_f64().unwrap())
}

impl<T> PartialOrd for Point<T>
    where T: Float + ToPrimitive
{
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        match to_float_ord(self.y()).partial_cmp(&to_float_ord(other.y()))
                                    .map(|o| o.reverse())
        {
            Some(Ordering::Equal) => {
                to_float_ord(self.x()).partial_cmp(&to_float_ord(other.x()))
            },
            value => value
        }
    }
}

impl<T> Ord for Point<T>
    where T: Float + ToPrimitive
{
    fn cmp(&self, other: &Self) -> Ordering {
        match to_float_ord(self.y()).cmp(&to_float_ord(other.y()))
                                    .reverse()
        {
            Ordering::Equal => {
                to_float_ord(self.x()).cmp(&to_float_ord(other.x()))
            },
            value => value
        }
    }
}

#[cfg(test)]
mod test {
    use types::Point;
    #[test]
    fn test_point_ord() {
        let a = Point::new(0., 0.);
        let b = Point::new(0., 1.);
        let c = Point::new(2., 1.);
        let d = Point::new(2., 0.);
        assert!(a == a);
        assert!(b < d);
        assert!(d > b);
        assert!(b < c);
        assert!(c < d);
        assert!(a > c);
    }
}
