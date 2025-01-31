use crate::{CoordFloat, Point};

#[deprecated(
    since = "0.29.0",
    note = "Please use the `Haversine.bearing` method from the `Bearing` trait instead"
)]
/// Returns the bearing to another Point in degrees.
///
/// Bullock, R.: Great Circle Distances and Bearings Between Two Locations, 2007.
/// (<https://dtcenter.org/met/users/docs/write_ups/gc_simple.pdf>)
pub trait HaversineBearing<T: CoordFloat> {
    /// Returns the bearing to another Point in degrees, where North is 0° and East is 90°.
    ///
    /// # Examples
    ///
    /// ```
    /// # use approx::assert_relative_eq;
    /// # #[allow(deprecated)]
    /// use geo::HaversineBearing;
    /// use geo::Point;
    ///
    /// let p_1 = Point::new(9.177789688110352, 48.776781529534965);
    /// let p_2 = Point::new(9.274410083250379, 48.84033282787534);
    /// # #[allow(deprecated)]
    /// let bearing = p_1.haversine_bearing(p_2);
    /// assert_relative_eq!(bearing, 45., epsilon = 1.0e-6);
    /// ```
    fn haversine_bearing(&self, point: Point<T>) -> T;
}

#[allow(deprecated)]
impl<T> HaversineBearing<T> for Point<T>
where
    T: CoordFloat,
{
    fn haversine_bearing(&self, point: Point<T>) -> T {
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
    use crate::point;
    #[allow(deprecated)]
    use crate::HaversineBearing;
    #[allow(deprecated)]
    use crate::HaversineDestination;

    #[test]
    fn north_bearing() {
        let p_1 = point!(x: 9., y: 47.);
        let p_2 = point!(x: 9., y: 48.);
        #[allow(deprecated)]
        let bearing = p_1.haversine_bearing(p_2);
        assert_relative_eq!(bearing, 0.);
    }

    #[test]
    fn equatorial_east_bearing() {
        let p_1 = point!(x: 9., y: 0.);
        let p_2 = point!(x: 10., y: 0.);
        #[allow(deprecated)]
        let bearing = p_1.haversine_bearing(p_2);
        assert_relative_eq!(bearing, 90.);
    }

    #[test]
    fn east_bearing() {
        let p_1 = point!(x: 9., y: 10.);
        let p_2 = point!(x: 18.12961917258341, y: 9.875828894123304);

        #[allow(deprecated)]
        let bearing = p_1.haversine_bearing(p_2);
        assert_relative_eq!(bearing, 90.);
    }

    #[test]
    fn northeast_bearing() {
        let p_1 = point!(x: 9.177789688110352f64, y: 48.776781529534965);
        let p_2 = point!(x: 9.274409949623548, y: 48.84033274015048);
        #[allow(deprecated)]
        let bearing = p_1.haversine_bearing(p_2);
        assert_relative_eq!(bearing, 45., epsilon = 1.0e-6);
    }

    #[test]
    fn consistent_with_destination() {
        let p_1 = point!(x: 9.177789688110352f64, y: 48.776781529534965);
        #[allow(deprecated)]
        let p_2 = p_1.haversine_destination(45., 10000.);

        #[allow(deprecated)]
        let b_1 = p_1.haversine_bearing(p_2);
        assert_relative_eq!(b_1, 45., epsilon = 1.0e-6);
    }
}
