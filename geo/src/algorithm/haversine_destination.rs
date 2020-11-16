use crate::{Point, MEAN_EARTH_RADIUS};
use num_traits::{Float, FromPrimitive};

/// Returns a new Point using the distance to the existing Point and a bearing for the direction
///
/// *Note*: this implementation uses a mean earth radius of 6371.088 km, based on the [recommendation of
/// the IUGG](ftp://athena.fsv.cvut.cz/ZFG/grs80-Moritz.pdf)
pub trait HaversineDestination<T: Float> {
    /// Returns a new Point using distance to the existing Point and a bearing for the direction
    ///
    /// # Units
    ///
    /// - `bearing`: degrees
    /// - `distance`: meters
    ///
    /// # Examples
    ///
    /// ```
    /// use geo::algorithm::haversine_destination::HaversineDestination;
    /// use geo::Point;
    ///
    /// let p_1 = Point::<f64>::new(9.177789688110352, 48.776781529534965);
    /// let p_2 = p_1.haversine_destination(45., 10000.);
    /// assert_eq!(p_2, Point::<f64>::new(9.274409949623548, 48.84033274015048))
    /// ```
    fn haversine_destination(&self, bearing: T, distance: T) -> Point<T>;
}

impl<T> HaversineDestination<T> for Point<T>
where
    T: Float + FromPrimitive,
{
    fn haversine_destination(&self, bearing: T, distance: T) -> Point<T> {
        let center_lng = self.x().to_radians();
        let center_lat = self.y().to_radians();
        let bearing_rad = bearing.to_radians();

        let rad = distance / T::from(MEAN_EARTH_RADIUS).unwrap();

        let lat =
            { center_lat.sin() * rad.cos() + center_lat.cos() * rad.sin() * bearing_rad.cos() }
                .asin();
        let lng = { bearing_rad.sin() * rad.sin() * center_lat.cos() }
            .atan2(rad.cos() - center_lat.sin() * lat.sin())
            + center_lng;

        Point::new(lng.to_degrees(), lat.to_degrees())
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::algorithm::haversine_distance::HaversineDistance;
    use num_traits::pow;

    #[test]
    fn returns_a_new_point() {
        let p_1 = Point::<f64>::new(9.177789688110352, 48.776781529534965);
        let p_2 = p_1.haversine_destination(45., 10000.);
        assert_eq!(p_2, Point::<f64>::new(9.274409949623548, 48.84033274015048));
        let distance = p_1.haversine_distance(&p_2);
        assert_relative_eq!(distance, 10000., epsilon = 1.0e-6)
    }

    #[test]
    fn direct_and_indirect_destinations_are_close() {
        let p_1 = Point::<f64>::new(9.177789688110352, 48.776781529534965);
        let p_2 = p_1.haversine_destination(45., 10000.);
        let square_edge = { pow(10000., 2) / 2. }.sqrt();
        let p_3 = p_1.haversine_destination(0., square_edge);
        let p_4 = p_3.haversine_destination(90., square_edge);
        assert_relative_eq!(p_4.x(), p_2.x(), epsilon = 1.0e-6);
        assert_relative_eq!(p_4.y(), p_2.y(), epsilon = 1.0e-6);
    }
}
