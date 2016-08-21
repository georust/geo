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
        let L = (lon2 - lon1).to_radians();
        let U1 = ((T::one() - f) * lat1.to_radians().tan()).atan();
        let U2 = ((T::one() - f) * lat2.to_radians().tan()).atan();
        let sinU1 = U1.sin();
        let cosU1 = U1.cos();
        let sinU2 = U2.sin();
        let cosU2 = U2.cos();
        let iterLimit = 100;
        let mut cosSqAlpha = T::zero();
        let mut cosSigma = T::zero();
        let mut cos2SigmaM = T::zero();
        let mut sinSigma = T::zero();
        let mut sigma = T::zero();
        let mut lambda = L;

        loop {
            let sinLambda = lambda.sin();
            let cosLambda = lambda.cos();
            sinSigma = ((cosU2 * sinLambda) *
                            (cosU2 * sinLambda) +
                            (cosU1 * sinU2 - sinU1 * cosU2 * cosLambda) *
                            (cosU1 * sinU2 - sinU1 * cosU2 * cosLambda)).sqrt();

            if T::zero() == sinSigma {
                return T::zero(); // co-incident points
            }

            cosSigma = sinU1 * sinU2 + cosU1 * cosU2 * cosLambda;
            sigma = sinSigma.atan2(cosSigma);
            let sinAlpha = cosU1 * cosU2 * sinLambda / sinSigma;
            cosSqAlpha = T::one() - sinAlpha * sinAlpha;
            cos2SigmaM = cosSigma - T::from_i32(2).unwrap() * sinU1 * sinU2 / cosSqAlpha;
            let sixteen = T::from_i32(16).unwrap();
            let four = T::from_i32(4).unwrap();
            let three = T::from_i32(3).unwrap();
            let C = f / sixteen *
                cosSqAlpha *
                (four + f * (four - three * cosSqAlpha));

            if cos2SigmaM.is_nan() {
                cos2SigmaM = T::zero(); // equatorial line: cosSqAlpha = 0 (§6)
            }

            let lambdaP = lambda;
            lambda = L + (T::one() - C) * f * sinAlpha * (sigma + C * sinSigma * (cos2SigmaM + C * cosSigma * (T::from_i32(-1).unwrap() + T::from_i32(2).unwrap() * cos2SigmaM * cos2SigmaM)));
            if (lambda - lambdaP).abs() > T::from_f64(1e-12).unwrap() && --iterLimit > 0 {
                break
            }
        }

        let uSq = cosSqAlpha * (a * a - b * b) / (b * b);
        let A = T::one() + uSq / T::from_i32(16384).unwrap() * 
            (T::from_i32(4096).unwrap() +
             uSq * (T::from_i32(-768).unwrap() +
                    uSq * (T::from_i32(320).unwrap() - 
                           T::from_i32(175).unwrap() * uSq)));
        let B = uSq / T::from_i32(1024).unwrap() *
            (T::from_i32(256).unwrap() +
             uSq * (T::from_i32(-128).unwrap() +
                    uSq * (T::from_i32(74).unwrap() -
                           T::from_i32(47).unwrap() * uSq)));
        let deltaSigma = B * sinSigma *
            (cos2SigmaM + B / T::from_i32(4).unwrap() *
             (cosSigma * (T::from_i32(-1).unwrap() +
                          T::from_i32(2).unwrap() * 
                          cos2SigmaM * cos2SigmaM) - B /
              T::from_i32(6).unwrap() * cos2SigmaM *
              (T::from_i32(-3).unwrap() +
               T::from_i32(4).unwrap() * sinSigma * sinSigma) *
              (T::from_i32(-3).unwrap() +
               T::from_i32(4).unwrap() *
               cos2SigmaM * cos2SigmaM)));
        b * A * (sigma - deltaSigma)
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
