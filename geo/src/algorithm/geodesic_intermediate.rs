use crate::{CoordFloat, Point};
use geographiclib_rs::{DirectGeodesic, Geodesic, InverseGeodesic};

/// Returns a new Point along a route between two existing points on an ellipsoidal model of the earth

pub trait GeodesicIntermediate<T: CoordFloat> {
    /// Returns a new Point along a route between two existing points on an ellipsoidal model of the earth
    ///
    /// # Examples
    ///
    /// ```
    /// # #[macro_use] extern crate approx;
    /// #
    /// use geo::GeodesicIntermediate;
    /// use geo::Point;
    ///
    /// let p1 = Point::<f64>::new(10.0, 20.0);
    /// let p2 = Point::<f64>::new(125.0, 25.0);
    /// let i20 = p1.geodesic_intermediate(&p2, 0.2);
    /// let i50 = p1.geodesic_intermediate(&p2, 0.5);
    /// let i80 = p1.geodesic_intermediate(&p2, 0.8);
    /// let i20_should = Point::new(29.842907, 29.951445);
    /// let i50_should = Point::new(65.879360, 37.722253);
    /// let i80_should = Point::new(103.556796, 33.506196);
    /// assert_relative_eq!(i20, i20_should, epsilon = 1.0e-6);
    /// assert_relative_eq!(i50, i50_should, epsilon = 1.0e-6);
    /// assert_relative_eq!(i80, i80_should, epsilon = 1.0e-6);
    /// ```

    fn geodesic_intermediate(&self, other: &Point<T>, f: T) -> Point<T>;
    fn geodesic_intermediate_fill(
        &self,
        other: &Point<T>,
        max_dist: T,
        include_ends: bool,
    ) -> Vec<Point<T>>;
}

impl GeodesicIntermediate<f64> for Point<f64> {
    fn geodesic_intermediate(&self, other: &Point<f64>, f: f64) -> Point<f64> {
        let g = Geodesic::wgs84();
        let (total_distance, azi1, _azi2, _a12) =
            g.inverse(self.y(), self.x(), other.y(), other.x());
        let distance = total_distance * f;
        let (lat2, lon2) = g.direct(self.y(), self.x(), azi1, distance);

        Point::new(lon2, lat2)
    }

    fn geodesic_intermediate_fill(
        &self,
        other: &Point<f64>,
        max_dist: f64,
        include_ends: bool,
    ) -> Vec<Point<f64>> {
        let g = Geodesic::wgs84();
        let (total_distance, azi1, _azi2, _a12) =
            g.inverse(self.y(), self.x(), other.y(), other.x());

        if total_distance <= max_dist {
            return if include_ends {
                vec![*self, *other]
            } else {
                vec![]
            };
        }

        let number_of_points = (total_distance / max_dist).ceil();
        let interval = 1.0 / number_of_points;

        let mut current_step = interval;
        let mut points = if include_ends { vec![*self] } else { vec![] };

        while current_step < 1.0 {
            let (lat2, lon2) = g.direct(self.y(), self.x(), azi1, total_distance * current_step);
            let point = Point::new(lon2, lat2);
            points.push(point);
            current_step += interval;
        }

        if include_ends {
            points.push(*other);
        }

        points
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use approx::assert_relative_eq;

    #[test]
    fn f_is_zero_or_one_test() {
        let p1 = Point::<f64>::new(10.0, 20.0);
        let p2 = Point::<f64>::new(15.0, 25.0);
        let i0 = p1.geodesic_intermediate(&p2, 0.0);
        let i100 = p1.geodesic_intermediate(&p2, 1.0);
        assert_relative_eq!(i0, p1, epsilon = 1.0e-6);
        assert_relative_eq!(i100, p2, epsilon = 1.0e-6);
    }

    #[test]
    fn various_f_values_test() {
        let p1 = Point::<f64>::new(10.0, 20.0);
        let p2 = Point::<f64>::new(125.0, 25.0);
        let i20 = p1.geodesic_intermediate(&p2, 0.2);
        let i50 = p1.geodesic_intermediate(&p2, 0.5);
        let i80 = p1.geodesic_intermediate(&p2, 0.8);
        let i20_should = Point::new(29.842907, 29.951445);
        let i50_should = Point::new(65.879360, 37.722253);
        let i80_should = Point::new(103.556796, 33.506196);
        assert_relative_eq!(i20, i20_should, epsilon = 1.0e-6);
        assert_relative_eq!(i50, i50_should, epsilon = 1.0e-6);
        assert_relative_eq!(i80, i80_should, epsilon = 1.0e-6);
    }

    #[test]
    fn should_add_i50_test() {
        let p1 = Point::<f64>::new(30.0, 40.0);
        let p2 = Point::<f64>::new(40.0, 50.0);
        let max_dist = 1000000.0; // meters
        let include_ends = true;
        let i50 = p1.geodesic_intermediate(&p2, 0.5);
        let route = p1.geodesic_intermediate_fill(&p2, max_dist, include_ends);
        assert_eq!(route, vec![p1, i50, p2]);
    }
}
