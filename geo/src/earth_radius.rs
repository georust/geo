use crate::CoordNum;
use num_traits::FromPrimitive;

/// Trait representing different mean earth radii
pub trait MeanEarthRadius {
    const VALUE: f64;

    fn value<T: CoordNum + FromPrimitive>() -> T {
        T::from(Self::VALUE).unwrap()
    }
}

/// Represents the WGS84 mean earth radius.
///
/// The WGS84 (World Geodetic System 1984) is a standard for use in cartography, geodesy, and satellite navigation.
/// The mean earth radius for WGS84 is approximately 6378137.0 meters.
pub struct Wgs84;

/// Represents the GRS80 mean earth radius.
///
/// The GRS80 (Geodetic Reference System 1980) is a geodetic reference system consisting of a global reference ellipsoid and a gravity field model.
/// The mean earth radius for GRS80 is approximately 6378137.0 meters.
pub struct Grs80;

/// Represents the mean earth radius as recommended by the IUGG.
///
/// The mean earth radius for this model is approximately 6371008.8 meters.
/// This value is recommended by the IUGG:
/// Moritz, H. (2000). Geodetic Reference System 1980. Journal of Geodesy, 74(1), 128–133. doi:10.1007/s001900050278
/// "Derived Geometric Constants: mean radius" (p133)
/// https://link.springer.com/article/10.1007/s001900050278
/// https://sci-hub.se/https://doi.org/10.1007/s001900050278
/// https://en.wikipedia.org/wiki/Earth_radius#Mean_radius
pub struct Iugg;

/// Represents a spherical mean earth radius.
///
/// This is a simplified model assuming the Earth is a perfect sphere.
/// The mean earth radius for a sphere is approximately 6371000.0 meters.
/// This value is commonly used in various scientific, educational, and engineering contexts for simplicity.
pub struct Sphere;

impl MeanEarthRadius for Wgs84 {
    const VALUE: f64 = 6378137.0;
}

impl MeanEarthRadius for Grs80 {
    const VALUE: f64 = 6378137.0;
}

impl MeanEarthRadius for Iugg {
    const VALUE: f64 = 6371008.8;
}

impl MeanEarthRadius for Sphere {
    const VALUE: f64 = 6371000.0;
}
