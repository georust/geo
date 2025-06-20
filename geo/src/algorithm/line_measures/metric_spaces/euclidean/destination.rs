use crate::CoordFloat;
use crate::Destination;
use crate::Euclidean;
use crate::Point;
use num_traits::FromPrimitive;

impl<F: CoordFloat + FromPrimitive> Destination<F> for Euclidean {
    /// Returns a new point having travelled the `distance` along a straight line
    /// from the `origin` point with the given `bearing`.
    ///
    /// # Units
    ///
    /// - `bearing`: degrees, where: North: 0°, East: 90°, South: 180°, West: 270°
    /// - `distance`: meters
    /// - returns: Point where x/y are cartesian coordinates
    ///
    /// # Examples
    ///
    /// ```
    /// # use approx::assert_relative_eq;
    /// use geo::{Point, Destination, Euclidean};
    ///
    /// // Determine the point 100 km NE of JFK airport.
    /// let origin = Point::new(0.0, 0.0);
    /// let northeast_bearing = 45.0;
    /// let distance = 100.0;
    ///
    /// let northeast_location= Euclidean.destination(origin, northeast_bearing, distance);
    /// assert_relative_eq!(
    ///     Point::new(70.71, 70.71),
    ///     northeast_location,
    ///     epsilon = 1.0e-2
    /// );
    /// ```
    fn destination(&self, origin: Point<F>, bearing: F, distance: F) -> Point<F> {
        let dy = distance * bearing.to_radians().cos();
        let dx = distance * bearing.to_radians().sin();

        Point::new(origin.x() + dx, origin.y() + dy)
    }
}

#[cfg(test)]
mod tests {
    use crate::algorithm::Bearing;
    use crate::algorithm::Destination;
    use crate::algorithm::Distance;
    use crate::{Euclidean, Point};

    #[test]
    fn test_360_bearing() {
        let origin = Point::new(0.0, 0.0);
        let distance = 1.0;

        for bearing in 0..360 {
            let bearing = bearing as f64;
            let destination = Euclidean.destination(origin, bearing, distance);

            assert_relative_eq!(
                Euclidean.bearing(origin, destination),
                bearing,
                epsilon = 1.0e-6
            );
            assert_relative_eq!(
                Euclidean.distance(origin, destination),
                distance,
                epsilon = 1.0e-6
            );
        }
    }
}
