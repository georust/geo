use crate::Point;
use num_traits::Float;

/// Returns the bearing to another Point in degrees.
///
/// Bullock, R.: Great Circle Distances and Bearings Between Two Locations, 2007.
/// (https://dtcenter.org/met/users/docs/write_ups/gc_simple.pdf)

pub trait Bearing<T: Float> {
    /// Returns the bearing to another Point in degrees, where North is 0° and East is 90°.
    ///
    /// # Examples
    ///
    /// ```
    /// # #[macro_use] extern crate approx;
    /// #
    /// use geo::algorithm::bearing::Bearing;
    /// use geo::Point;
    ///
    /// let p_1 = Point::<f64>::new(9.177789688110352, 48.776781529534965);
    /// let p_2 = Point::<f64>::new(9.274410083250379, 48.84033282787534);
    /// let bearing = p_1.bearing(p_2);
    /// assert_relative_eq!(bearing, 45., epsilon = 1.0e-6);
    /// ```
    fn bearing(&self, point: Point<T>) -> T;
}

impl<T> Bearing<T> for Point<T>
where
    T: Float,
{
    fn bearing(&self, point: Point<T>) -> T {
        let (lng_a, lat_a) = (self.x().to_radians(), self.y().to_radians());
        let (lng_b, lat_b) = (point.x().to_radians(), point.y().to_radians());
        let delta_lng = lng_b - lng_a;
        let s = lat_b.cos() * delta_lng.sin();
        let c = lat_a.cos() * lat_b.sin() - lat_a.sin() * lat_b.cos() * delta_lng.cos();

        T::atan2(s, c).to_degrees()
    }
}

#[cfg(test)]
mod test {
    use crate::algorithm::bearing::Bearing;
    use crate::algorithm::haversine_destination::HaversineDestination;
    use crate::point;

    #[test]
    fn returns_the_proper_bearing_to_another_point() {
        let p_1 = point!(x: 9.177789688110352f64, y: 48.776781529534965);
        let p_2 = p_1.haversine_destination(45., 10000.);
        let b_1 = p_1.bearing(p_2);
        assert_relative_eq!(b_1, 45., epsilon = 1.0e-6);

        let p_3 = point!(x: 9., y: 47.);
        let p_4 = point!(x: 9., y: 48.);
        let b_2 = p_3.bearing(p_4);
        assert_relative_eq!(b_2, 0., epsilon = 1.0e-6);
    }
}
