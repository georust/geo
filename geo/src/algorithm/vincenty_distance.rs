// A few resources:
//
// - http://www.movable-type.co.uk/scripts/latlong-vincenty.html
// - https://nathanrooy.github.io/posts/2016-12-18/vincenty-formula-with-python/
// - https://github.com/janantala/GPS-distance/blob/master/java/Distance.java

use crate::{Point, EARTH_FLATTENING, EQUATORIAL_EARTH_RADIUS, POLAR_EARTH_RADIUS};
use num_traits::{Float, FromPrimitive};
use std::{error, fmt};

/// Determine the distance between two geometries using [Vincenty’s formulae].
///
/// [Vincenty’s formulae]: https://en.wikipedia.org/wiki/Vincenty%27s_formulae
pub trait VincentyDistance<T, Rhs = Self> {
    /// Determine the distance between two geometries using [Vincenty’s
    /// formulae].
    ///
    /// # Units
    ///
    /// - return value: meters
    ///
    /// # Examples
    ///
    /// ```
    /// use geo::prelude::*;
    /// use geo::point;
    ///
    /// // New York City
    /// let p1 = point!(x: -74.006f64, y: 40.7128f64);
    ///
    /// // London
    /// let p2 = point!(x: -0.1278f64, y: 51.5074f64);
    ///
    /// let distance = p1.vincenty_distance(&p2).unwrap();
    ///
    /// assert_eq!(
    ///     5_585_234., // meters
    ///     distance.round()
    /// );
    /// ```
    ///
    /// [Vincenty’s formulae]: https://en.wikipedia.org/wiki/Vincenty%27s_formulae
    fn vincenty_distance(&self, rhs: &Rhs) -> Result<T, FailedToConvergeError>;
}

impl<T> VincentyDistance<T, Point<T>> for Point<T>
where
    T: Float + FromPrimitive,
{
    #[allow(non_snake_case)]
    fn vincenty_distance(&self, rhs: &Point<T>) -> Result<T, FailedToConvergeError> {
        let t_1 = T::one();
        let t_2 = T::from(2).unwrap();
        let t_3 = T::from(3).unwrap();
        let t_4 = T::from(4).unwrap();
        let t_6 = T::from(6).unwrap();
        let t_47 = T::from(47).unwrap();
        let t_16 = T::from(16).unwrap();
        let t_74 = T::from(74).unwrap();
        let t_128 = T::from(128).unwrap();
        let t_175 = T::from(175).unwrap();
        let t_256 = T::from(256).unwrap();
        let t_320 = T::from(320).unwrap();
        let t_768 = T::from(768).unwrap();
        let t_1024 = T::from(1024).unwrap();
        let t_4096 = T::from(4096).unwrap();
        let t_16384 = T::from(16384).unwrap();

        let a = T::from(EQUATORIAL_EARTH_RADIUS).unwrap();
        let b = T::from(POLAR_EARTH_RADIUS).unwrap();
        let f = T::from(EARTH_FLATTENING).unwrap();
        // Difference in longitude
        let L = (rhs.lng() - self.lng()).to_radians();
        // Reduced latitude (latitude on the auxiliary sphere)
        let U1 = ((t_1 - f) * self.lat().to_radians().tan()).atan();
        // Reduced latitude (latitude on the auxiliary sphere)
        let U2 = ((t_1 - f) * rhs.lat().to_radians().tan()).atan();
        let (sinU1, cosU1) = U1.sin_cos();
        let (sinU2, cosU2) = U2.sin_cos();
        let mut cosSqAlpha;
        let mut sinSigma;
        let mut cos2SigmaM;
        let mut cosSigma;
        let mut sigma;
        // Longitude of the points on the auxiliary sphere
        let mut lambda = L;
        let mut lambdaP;
        let mut iterLimit = 100;

        loop {
            let (sinLambda, cosLambda) = lambda.sin_cos();
            sinSigma = ((cosU2 * sinLambda) * (cosU2 * sinLambda)
                + (cosU1 * sinU2 - sinU1 * cosU2 * cosLambda)
                    * (cosU1 * sinU2 - sinU1 * cosU2 * cosLambda))
                .sqrt();

            if sinSigma.is_zero() {
                if self == rhs {
                    // coincident points
                    return Ok(T::zero());
                } else {
                    // anitpodal points, for which vincenty does not converge
                    return Err(FailedToConvergeError);
                }
            }

            cosSigma = sinU1 * sinU2 + cosU1 * cosU2 * cosLambda;
            sigma = sinSigma.atan2(cosSigma);
            let sinAlpha = cosU1 * cosU2 * sinLambda / sinSigma;
            cosSqAlpha = t_1 - sinAlpha * sinAlpha;

            if cosSqAlpha.is_zero() {
                // equatorial geodesics require special handling
                // per [Algorithms for geodesics, Charles F. F. Karney](https://arxiv.org/pdf/1109.4448.pdf)
                cos2SigmaM = T::zero()
            } else {
                cos2SigmaM = cosSigma - t_2 * sinU1 * sinU2 / cosSqAlpha;
            }

            let C = f / t_16 * cosSqAlpha * (t_4 + f * (t_4 - t_3 * cosSqAlpha));
            lambdaP = lambda;
            lambda = L
                + (t_1 - C)
                    * f
                    * sinAlpha
                    * (sigma
                        + C * sinSigma
                            * (cos2SigmaM + C * cosSigma * (-t_1 + t_2 * cos2SigmaM * cos2SigmaM)));

            if (lambda - lambdaP).abs() <= T::from(1e-12).unwrap() {
                break;
            }

            iterLimit -= 1;

            if iterLimit == 0 {
                break;
            }
        }

        if iterLimit == 0 {
            return Err(FailedToConvergeError);
        }

        let uSq = cosSqAlpha * (a * a - b * b) / (b * b);
        let A = t_1 + uSq / t_16384 * (t_4096 + uSq * (-t_768 + uSq * (t_320 - t_175 * uSq)));
        let B = uSq / t_1024 * (t_256 + uSq * (-t_128 + uSq * (t_74 - t_47 * uSq)));

        let deltaSigma = B
            * sinSigma
            * (cos2SigmaM
                + B / t_4
                    * (cosSigma * (-t_1 + t_2 * cos2SigmaM * cos2SigmaM)
                        - B / t_6
                            * cos2SigmaM
                            * (-t_3 + t_4 * sinSigma * sinSigma)
                            * (-t_3 + t_4 * cos2SigmaM * cos2SigmaM)));

        let s = b * A * (sigma - deltaSigma);

        Ok(s)
    }
}

#[derive(Eq, PartialEq, Debug)]
pub struct FailedToConvergeError;

impl fmt::Display for FailedToConvergeError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Vincenty algorithm failed to converge")
    }
}

impl error::Error for FailedToConvergeError {
    fn description(&self) -> &str {
        "Vincenty algorithm failed to converge"
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_vincenty_distance_1() {
        let a = Point::<f64>::new(17.072561, 48.154563);
        let b = Point::<f64>::new(17.072562, 48.154564);
        assert_relative_eq!(
            a.vincenty_distance(&b).unwrap(),
            0.13378944117648012,
            epsilon = 1.0e-6
        );
    }

    #[test]
    fn test_vincenty_distance_2() {
        let a = Point::<f64>::new(17.072561, 48.154563);
        let b = Point::<f64>::new(17.064064, 48.158800);
        assert_relative_eq!(
            a.vincenty_distance(&b).unwrap(),
            788.4148295236967,
            epsilon = 1.0e-6
        );
    }

    #[test]
    fn test_vincenty_distance_3() {
        let a = Point::<f64>::new(17.107558, 48.148636);
        let b = Point::<f64>::new(16.372477, 48.208810);
        assert_relative_eq!(
            a.vincenty_distance(&b).unwrap(),
            55073.68246366003,
            epsilon = 1.0e-6
        );
    }

    #[test]
    fn test_vincenty_distance_equatorial() {
        let a = Point::<f64>::new(0.0, 0.0);
        let b = Point::<f64>::new(100.0, 0.0);
        assert_relative_eq!(
            a.vincenty_distance(&b).unwrap(),
            11131949.079,
            epsilon = 1.0e-3
        );
    }

    #[test]
    fn test_vincenty_distance_coincident() {
        let a = Point::<f64>::new(12.3, 4.56);
        let b = Point::<f64>::new(12.3, 4.56);
        assert_relative_eq!(a.vincenty_distance(&b).unwrap(), 0.0, epsilon = 1.0e-3);
    }

    #[test]
    fn test_vincenty_distance_antipodal() {
        let a = Point::<f64>::new(2.0, 4.0);
        let b = Point::<f64>::new(-178.0, -4.0);
        assert_eq!(a.vincenty_distance(&b), Err(FailedToConvergeError))
    }
}
