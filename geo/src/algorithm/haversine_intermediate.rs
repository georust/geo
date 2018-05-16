use num_traits::{Float, FromPrimitive};
use ::Point;

/// Returns a new Point along a great circle route between two existing points

pub trait HaversineIntermediate<T: Float> {
    /// Returns a new Point along a great circle route between two existing points.
    ///
    /// ```
    /// use geo::Point;
    /// use geo::algorithm::haversine_intermediate::HaversineIntermediate;
    ///
    /// let p1 = Point::<f64>::new(10.0, 20.0);
    /// let p2 = Point::<f64>::new(15.0, 25.0);
    /// let pi = p1.haversine_intermediate(&p2, 0.5);
    /// assert_eq!(pi, Point::<f64>::new(0.0, 0.0))
    /// ```
    fn haversine_intermediate(&self, other: &Point<T>, f: T) -> Point<T>;
}

impl<T> HaversineIntermediate<T> for Point<T>
where
    T: Float + FromPrimitive,
{
    fn haversine_intermediate(&self, other: &Point<T>, f: T) -> Point<T> {
        Point::new(T::zero(), T::zero())
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use algorithm::haversine_intermediate::HaversineIntermediate;
    use num_traits::pow;

    #[test]
    fn f_is_zero_or_one_test() {
        let p1 = Point::<f64>::new(10.0, 20.0);
        let p2 = Point::<f64>::new(15.0, 25.0);
        let i0   = p1.haversine_intermediate(&p2, 0.0);
        let i100 = p1.haversine_intermediate(&p2, 1.0);
        assert_eq!(i0,   p1);
        assert_eq!(i100, p2);
    }

    #[test]
    fn various_f_values_test() {
        let p1 = Point::<f64>::new(10.0,  20.0);
        let p2 = Point::<f64>::new(125.0, 25.0);
        let i20  = p1.haversine_intermediate(&p2, 0.2);
        let i50  = p1.haversine_intermediate(&p2, 0.5);
        let i80  = p1.haversine_intermediate(&p2, 0.8);
        assert_eq!(i20, Point::new(29.83519,  29.94841));
        assert_eq!(i50, Point::new(65.87471,  37.72201));
        assert_eq!(i80, Point::new(103.56036, 33.50518));
    }

    #[test]
    fn should_be_north_pole_test() {
        let p1 = Point::<f64>::new(0.0, 0.0);
        let p2 = Point::<f64>::new(0.0, 180.0);
        let i50  = p1.haversine_intermediate(&p2, 0.5);
        assert_eq!(i50, Point::new(90.0, 0.0));
    }
}

