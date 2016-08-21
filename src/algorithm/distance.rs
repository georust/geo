use num::{Float, FromPrimitive};
use types::{Point};

/// Returns the distance between two geometries.
trait Distance<T, Rhs = Self> {
    fn distance<A: DistanceAlgorithm>(&self, rhs: &Rhs) -> T;
}

trait EuclideanAlgorithm {
}

struct Euclidean;

impl DistanceAlgorithm for Euclidean {
    fn distance(&self, p: &Point<T>) -> T {
        let (dx, dy) = (self.x() - p.x(), self.y() - p.y());
        dx.hypot(dy)
    }
}

#[cfg(test)]
mod test {
    use types::{Point};
    use algorithm::distance::{Distance};
    #[test]
    fn distance1_test() {
        assert_eq!(Point::<f64>::new(0., 0.).distance(&Point::<f64>::new(1., 0.)), 1.);
    }
    #[test]
    fn distance2_test() {
        // Point::new(-72.1235, 42.3521).distance(&Point::new(72.1260, 70.612)) = 146.99163308930207
        let dist = Point::new(-72.1235, 42.3521).distance(&Point::new(72.1260, 70.612));
        assert!(dist < 147. && dist > 146.);
    }
}


/// Returns the distance between two geometries.

// pub trait HaversineDistance<T, Rhs = Self>
// {
//     /// Returns the distance between two points:
//     ///
//     /// ```
//     /// use geo::{COORD_PRECISION, Point};
//     /// use geo::algorithm::haversine_distance::HaversineDistance;
//     ///
//     /// let p = Point::new(-72.1235, 42.3521);
//     /// let dist = p.haversine_distance(&Point::new(-72.1260, 42.45));
//     /// assert_eq!(dist as i32, 10900)
//     /// ```
//     fn haversine_distance(&self, rhs: &Rhs) -> T;
// }
// 
// impl<T> HaversineDistance<T, Point<T>> for Point<T>
//     where T: Float + FromPrimitive
// {
//     // currently gives answer in meters
//     fn haversine_distance(&self, p: &Point<T>) -> T {
//         let a = (self.y().to_radians().sin() * p.y().to_radians().sin()) +
//             (self.y().to_radians().cos() * p.y().to_radians().cos()) *
//             (p.x() - self.x()).to_radians().cos();
//         T::from_i32(6378137).unwrap() * a.acos().min(T::one())
//     }
// }
// 
// #[cfg(test)]
// mod test {
//     use types::Point;
//     use algorithm::haversine_distance::HaversineDistance;
//     #[test]
//     fn distance3_test() {
//         let dist = Point::new(-101.60, 37.43)
//             .haversine_distance(&Point::new(-78.75, 40.97));
//         assert_eq!(dist, 2004106.44144124);
//     }
// }
