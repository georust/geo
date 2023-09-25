use crate::{haversine_bearing, CoordFloat, Point};

pub use haversine_bearing::HaversineBearing;

#[deprecated(since = "0.24.1", note = "renamed to `HaversineBearing`")]
pub trait Bearing<T: CoordFloat> {
    /// Returns the bearing to another Point in degrees, where North is 0° and East is 90°.
    ///
    /// # Examples
    ///
    /// ```
    /// # use approx::assert_relative_eq;
    /// use geo::Bearing;
    /// use geo::Point;
    ///
    /// let p_1 = Point::new(9.177789688110352, 48.776781529534965);
    /// let p_2 = Point::new(9.274410083250379, 48.84033282787534);
    /// let bearing = p_1.bearing(p_2);
    /// assert_relative_eq!(bearing, 45., epsilon = 1.0e-6);
    /// ```
    #[deprecated(
        since = "0.24.1",
        note = "renamed to `HaversineBearing::haversine_bearing`"
    )]
    fn bearing(&self, point: Point<T>) -> T;
}

#[allow(deprecated)]
impl<T: CoordFloat, B: HaversineBearing<T>> Bearing<T> for B {
    fn bearing(&self, point: Point<T>) -> T {
        self.haversine_bearing(point)
    }
}
