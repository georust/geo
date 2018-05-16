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
        let one = T::one();
        let two = one + one;

        let lat1 =  self.y().to_radians();
        let lon1 =  self.x().to_radians();
        let lat2 = other.y().to_radians();
        let lon2 = other.x().to_radians();

        let k = (((lat1 - lat2).sin() / two).powi(2) +
                lat1.cos() * lat2.cos() *
                ((lon1 - lon2).sin() / two).powi(2)).sqrt();

        let d = two * k.asin();

        let a = ((one - f) * d).sin() / d.sin();
        let b = (f * d).sin() / d.sin();

        let x = a * lat1.cos() * lon1.cos() +
                b * lat2.cos() * lon2.cos();

        let y = a * lat1.cos() * lon1.sin() +
                b * lat2.cos() * lon2.sin();

        let z = a * lat1.sin() + b * lat2.sin();

        let dxy = (x.powi(2) + y.powi(2)).sqrt();

        let lat = z.atan2(dxy);
        let lon = y.atan2(x);

        Point::new(lon.to_degrees(), lat.to_degrees())
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
        assert_relative_eq!(i0.x(), p1.x(), epsilon=1.0e-6);
        assert_relative_eq!(i0.y(), p1.y(), epsilon=1.0e-6);
        assert_relative_eq!(i100.x(), p2.x(), epsilon=1.0e-6);
        assert_relative_eq!(i100.y(), p2.y(), epsilon=1.0e-6);
    }

    #[test]
    fn various_f_values_test() {
        let p1 = Point::<f64>::new(10.0,  20.0);
        let p2 = Point::<f64>::new(125.0, 25.0);
        let i20  = p1.haversine_intermediate(&p2, 0.2);
        let i50  = p1.haversine_intermediate(&p2, 0.5);
        let i80  = p1.haversine_intermediate(&p2, 0.8);
        let i20_should = Point::new(29.83519,  29.94841);
        let i50_should = Point::new(65.87471,  37.72201);
        let i80_should = Point::new(103.56036, 33.50518);
        assert_relative_eq!(i20.x(), i20_should.x(), epsilon=1.0e-6);
        assert_relative_eq!(i20.y(), i20_should.y(), epsilon=1.0e-6);
        assert_relative_eq!(i50.x(), i50_should.x(), epsilon=1.0e-6);
        assert_relative_eq!(i50.y(), i50_should.y(), epsilon=1.0e-6);
        assert_relative_eq!(i80.x(), i80_should.x(), epsilon=1.0e-6);
        assert_relative_eq!(i80.y(), i80_should.y(), epsilon=1.0e-6);
    }

    #[test]
    fn should_be_north_pole_test() {
        let p1 = Point::<f64>::new(0.0,   10.0);
        let p2 = Point::<f64>::new(180.0, 10.0);
        let i50  = p1.haversine_intermediate(&p2, 0.5);
        let i50_should = Point::new(90.0, 90.0);
        assert_relative_eq!(i50.x(), i50_should.x(), epsilon=1.0e-6);
        assert_relative_eq!(i50.y(), i50_should.y(), epsilon=1.0e-6);
    }
}

