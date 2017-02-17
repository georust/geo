use num::{Float, FromPrimitive, ToPrimitive};
use types::{Point};

/// Returns the distance between two geometries.

pub trait VincentyDistance<T, Rhs = Self>
{
    /// Returns the distance between two points:
    ///
    /// ```
    /// use geo::{COORD_PRECISION, Point};
    /// use geo::algorithm::vincenty_distance::VincentyDistance;
    ///
    /// let p = Point::new(-72.1235, 42.3521);
    /// let dist = p.vincenty_distance(&Point::new(-72.1260, 42.45));
    /// assert_eq!(dist, 10876.77787360232)
    /// ```
    fn vincenty_distance(&self, rhs: &Rhs) -> T;
}

impl<T> VincentyDistance<T, Point<T>> for Point<T>
where T: Float + FromPrimitive + ToPrimitive
{
    // https://gist.githubusercontent.com/mathiasbynens/354587/raw/4df3cce91f08b769207ee051c54b9f29c5d895a1/JavaScript%2520function%2520to%2520calculate%2520the%2520geodetic%2520distance%2520between%2520two%2520points%2520specified%2520by%2520latitude%2520and%2520longitude%2520using%2520the%2520Vincenty%2520inverse%2520formula%2520for%2520ellipsoids.
    fn vincenty_distance(&self, p: &Point<T>) -> T {
        let lat1 = self.y();
        let lon1 = self.x();
        let lat2 = p.y();
        let lon2 = p.x();
        let a = T::from_i32(6378137).unwrap();
        let b = T::from_f64(6356752.3142).unwrap();
        let f = T::one() / T::from_f64(298.257223563).unwrap();
        let l = (lon2 - lon1).to_radians();
        let u1 = ((T::one() - f) * lat1.to_radians().tan()).atan();
        let u2 = ((T::one() - f) * lat2.to_radians().tan()).atan();
        let sin_u1 = u1.sin();
        let cos_u1 = u1.cos();
        let sin_u2 = u2.sin();
        let cos_u2 = u2.cos();
        let iter_limit = 100;
        let mut cos_sq_alpha;
        let mut cos_sigma;
        let mut cos_2_sigma_m;
        let mut sin_sigma;
        let mut sigma;
        let mut lambda = l;

        loop {
            let sin_lambda = lambda.sin();
            let cos_lambda = lambda.cos();
            sin_sigma = ((cos_u2 * sin_lambda) *
                            (cos_u2 * sin_lambda) +
                            (cos_u1 * sin_u2 - sin_u1 * cos_u2 * cos_lambda) *
                            (cos_u1 * sin_u2 - sin_u1 * cos_u2 * cos_lambda)).sqrt();

            if T::zero() == sin_sigma {
                return T::zero(); // co-incident points
            }

            cos_sigma = sin_u1 * sin_u2 + cos_u1 * cos_u2 * cos_lambda;
            sigma = sin_sigma.atan2(cos_sigma);
            let sin_alpha = cos_u1 * cos_u2 * sin_lambda / sin_sigma;
            cos_sq_alpha = T::one() - sin_alpha * sin_alpha;
            cos_2_sigma_m = cos_sigma - T::from_i32(2).unwrap() * sin_u1 * sin_u2 / cos_sq_alpha;
            let sixteen = T::from_i32(16).unwrap();
            let four = T::from_i32(4).unwrap();
            let three = T::from_i32(3).unwrap();
            let c = f / sixteen *
                cos_sq_alpha *
                (four + f * (four - three * cos_sq_alpha));

            if cos_2_sigma_m.is_nan() {
                cos_2_sigma_m = T::zero(); // equatorial line: cos_sq_alpha = 0 (§6)
            }

            let lambda_p = lambda;
            lambda = l + (T::one() - c) * f * sin_alpha * (sigma + c * sin_sigma * (cos_2_sigma_m + c * cos_sigma * (T::from_i32(-1).unwrap() + T::from_i32(2).unwrap() * cos_2_sigma_m * cos_2_sigma_m)));
            if (lambda - lambda_p).abs() > T::from_f64(1e-12).unwrap() && --iter_limit > 0 {
                break
            }
        }

        let u_sq = cos_sq_alpha * (a * a - b * b) / (b * b);
        let a = T::one() + u_sq / T::from_i32(16384).unwrap() * 
            (T::from_i32(4096).unwrap() +
             u_sq * (T::from_i32(-768).unwrap() +
                    u_sq * (T::from_i32(320).unwrap() - 
                           T::from_i32(175).unwrap() * u_sq)));
        let b = u_sq / T::from_i32(1024).unwrap() *
            (T::from_i32(256).unwrap() +
             u_sq * (T::from_i32(-128).unwrap() +
                    u_sq * (T::from_i32(74).unwrap() -
                           T::from_i32(47).unwrap() * u_sq)));
        let delta_sigma = b * sin_sigma *
            (cos_2_sigma_m + b / T::from_i32(4).unwrap() *
             (cos_sigma * (T::from_i32(-1).unwrap() +
                          T::from_i32(2).unwrap() * 
                          cos_2_sigma_m * cos_2_sigma_m) - b /
              T::from_i32(6).unwrap() * cos_2_sigma_m *
              (T::from_i32(-3).unwrap() +
               T::from_i32(4).unwrap() * sin_sigma * sin_sigma) *
              (T::from_i32(-3).unwrap() +
               T::from_i32(4).unwrap() *
               cos_2_sigma_m * cos_2_sigma_m)));
        b * a * (sigma - delta_sigma)
    }
}

#[cfg(test)]
mod test {
    use types::{Point};
    use algorithm::vincenty_distance::{VincentyDistance};
    #[test]
    fn distance1_test() {
        assert_eq!(Point::<f64>::new(0., 0.).vincenty_distance(&Point::<f64>::new(1., 0.)), 110946.25761655909);
    }
    #[test]
    fn distance2_test() {
        // Point::new(-72.1235, 42.3521).distance(&Point::new(72.1260, 70.612)) = 146.99163308930207
        let dist = Point::new(-72.1235, 42.3521).vincenty_distance(&Point::new(72.1260, 70.612));
        assert_eq!(dist, 7150707.429008841_f64);
    }
}
