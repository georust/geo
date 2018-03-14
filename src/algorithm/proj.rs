//!
//! Two coordinate operations are currently provided via bindings to `libproj`:
//! **projection** (and inverse projection)
//! and **conversion**. Projection is intended for transformations between geodetic and projected coordinates,
//! and vice versa (inverse projection), while conversion is intended for transformations and conversions
//! between projected coordinate systems.
//! From the PROJ [documentation](http://proj4.org/operations/index.html):
//!
//! - At the most generic level, a coordinate operation is a change
//! of coordinates, based on a one-to-one relationship, from one
//! coordinate reference system to another.
//! - A **transformation** is a coordinate operation in which the two
//! coordinate reference systems are based on different datums, e.g.
//! a change from a global reference frame to a regional frame.
//! - A **conversion** is a coordinate operation in which both
//! coordinate reference systems are based on the same datum,
//! e.g. change of units of coordinates.
//! - A **projection** is a coordinate conversion from an _ellipsoidal_
//! coordinate system to a _plane_. Although projections are simply
//! conversions according to the standard, they are treated as
//! separate entities in PROJ.
//!
//! The `Project` and `Convert` traits are available to any `Geometry` implementing `MapCoords`.
use std::marker::Sized;
use proj_sys::proj_errno;
use libc::{c_char, c_double, c_int};
use std::ffi::CString;
use types::{CoordinateType, Point};
use algorithm::map_coords::MapCoordsFallible;
use std::ffi::CStr;
use std::str;
use failure::Error;
use proj_sys::{pj_strerrno, proj_context_create, proj_create, proj_create_crs_to_crs,
               proj_destroy, proj_pj_info, proj_trans, PJconsts, PJ_AREA, PJ_COORD,
               PJ_DIRECTION_PJ_FWD, PJ_DIRECTION_PJ_INV, PJ_LP, PJ_XY};

/// Easily get a String from the external library
fn _string(raw_ptr: *const c_char) -> String {
    let c_str = unsafe { CStr::from_ptr(raw_ptr) };
    str::from_utf8(c_str.to_bytes()).unwrap().to_string()
}

/// Look up an error message using the error code
fn error_message(code: c_int) -> String {
    let rv = unsafe { pj_strerrno(code) };
    return _string(rv);
}

/// A `PROJ` projection or transformation instance
pub struct Proj {
    c_proj: *mut PJconsts,
}

impl Proj {
    /// Try to instantiate a new `PROJ` instance
    ///
    /// **Note:** for projection operations, `definition` specifies
    /// the **output** projection; input coordinates
    /// are assumed to be geodetic in radians, unless an inverse projection is intended.
    ///
    /// For conversion operations, `definition` defines input, output, and
    /// any intermediate steps that are required. See the `convert` example for more details.

    // In contrast to PROJ v4.x, the type of transformation
    // is signalled by the choice of enum used as input to the PJ_COORD union
    // PJ_LP signals projection of geodetic coordinates, with output being PJ_XY
    // and vice versa, or using PJ_XY for conversion operations
    pub fn new(definition: &str) -> Option<Proj> {
        let c_definition = CString::new(definition.as_bytes()).unwrap();
        let ctx = unsafe { proj_context_create() };
        let new_c_proj = unsafe { proj_create(ctx, c_definition.as_ptr()) };
        if new_c_proj.is_null() {
            None
        } else {
            Some(Proj { c_proj: new_c_proj })
        }
    }

    // FIXME: we can't implement this yet because PJ_AREA isn't implemented
    // /// Create a transformation object from two known EPSG CRS codes
    // pub fn new_known_crs(from: &str, to: &str) -> Option<Proj> {
    //     let from_c = CString::new(from.as_bytes()).unwrap();
    //     let to_c = CString::new(to.as_bytes()).unwrap();
    //     let ctx = unsafe { proj_context_create() };
    //     // not implemented yet, see http://proj4.org/development/reference/datatypes.html#c.PJ_AREA
    //     let mut area = PJ_AREA { area: 0 };
    //     let raw_area = &mut area as *mut PJ_AREA;
    //     let new_c_proj =
    //         unsafe { proj_create_crs_to_crs(ctx, from_c.as_ptr(), to_c.as_ptr(), raw_area) };
    //     if new_c_proj.is_null() {
    //         None
    //     } else {
    //         Some(Proj { c_proj: new_c_proj })
    //     }
    // }

    /// Get the current definition from `PROJ`
    pub fn def(&self) -> String {
        let rv = unsafe { proj_pj_info(self.c_proj) };
        _string(rv.definition)
    }
    /// Project geodetic `Point` coordinates (in radians) into the projection specified by `definition`
    ///
    /// **Note:** specifying `inverse` as `true` carries out an inverse projection *to* geodetic coordinates
    /// (in radians) from the projection specified by `definition`.
    pub fn project<T>(&self, point: Point<T>, inverse: bool) -> Result<Point<T>, Error>
    where
        T: CoordinateType,
    {
        let inv = if inverse {
            PJ_DIRECTION_PJ_INV
        } else {
            PJ_DIRECTION_PJ_FWD
        };
        let c_x: c_double = point.x().to_f64().unwrap();
        let c_y: c_double = point.y().to_f64().unwrap();
        let new_x;
        let new_y;
        let err;
        // Input coords are defined in terms of lambda & phi, using the PJ_LP struct.
        // This signals that we wish to project geodetic coordinates.
        // For conversion (i.e. between projected coordinates) you should use
        // PJ_XY {x: , y: }
        let coords = PJ_LP { lam: c_x, phi: c_y };
        unsafe {
            // PJ_DIRECTION_* determines a forward or inverse projection
            let trans = proj_trans(self.c_proj, inv, PJ_COORD { lp: coords });
            // output of coordinates uses the PJ_XY struct
            new_x = trans.xy.x;
            new_y = trans.xy.y;
            err = proj_errno(self.c_proj);
        }
        if err == 0 {
            Ok(Point::new(T::from(new_x).unwrap(), T::from(new_y).unwrap()))
        } else {
            Err(format_err!(
                "The projection failed with the following error: {}",
                error_message(err)
            ))
        }
    }

    /// Convert or transform `Point` coordinates using the PROJ `pipeline` operator
    ///
    /// This method makes use of the [`pipeline`](http://proj4.org/operations/pipeline.html)
    /// functionality available since v5.0.0, which differs significantly from the v4.x series
    ///
    /// It has the advantage of being able to chain an arbitrary combination of conversion
    /// and transformation steps, allowing for extremely complex operations.
    ///
    /// The following example converts from NAD83 US Survey Feet (EPSG 2230) to NAD83 Metres (EPSG 26946)
    /// Note the steps:
    ///
    /// - define the operation as a `pipeline` operation
    /// - define `step` 1 as an `inv`erse transform, yielding geodetic coordinates
    /// - define `step` 2 as a forward transform to projected coordinates, yielding metres.
    ///
    /// ```rust
    /// use geo::Point;
    /// use geo::algorithm::proj::Proj;
    ///
    /// let nad_ft_to_m = Proj::new("
    ///     +proj=pipeline
    ///     +step +inv +proj=lcc +lat_1=33.88333333333333
    ///     +lat_2=32.78333333333333 +lat_0=32.16666666666666
    ///     +lon_0=-116.25 +x_0=2000000.0001016 +y_0=500000.0001016001 +ellps=GRS80
    ///     +towgs84=0,0,0,0,0,0,0 +units=us-ft +no_defs
    ///     +step +proj=lcc +lat_1=33.88333333333333 +lat_2=32.78333333333333 +lat_0=32.16666666666666
    ///     +lon_0=-116.25 +x_0=2000000 +y_0=500000
    ///     +ellps=GRS80 +towgs84=0,0,0,0,0,0,0 +units=m +no_defs
    /// ").unwrap();
    /// let result = nad_ft_to_m.convert(Point::new(4760096.421921, 3744293.729449)).unwrap();
    /// assert_eq!(result.x(), 1450880.2910605003);
    /// assert_eq!(result.y(), 1141263.01116045);
    ///
    /// ```
    pub fn convert<T>(&self, point: Point<T>) -> Result<Point<T>, Error>
    where
        T: CoordinateType,
    {
        let c_x: c_double = point.x().to_f64().unwrap();
        let c_y: c_double = point.y().to_f64().unwrap();
        let new_x;
        let new_y;
        let err;
        let coords = PJ_XY { x: c_x, y: c_y };
        unsafe {
            let trans = proj_trans(self.c_proj, PJ_DIRECTION_PJ_FWD, PJ_COORD { xy: coords });
            new_x = trans.xy.x;
            new_y = trans.xy.y;
            err = proj_errno(self.c_proj);
        }
        if err == 0 {
            Ok(Point::new(T::from(new_x).unwrap(), T::from(new_y).unwrap()))
        } else {
            Err(format_err!(
                "The conversion failed with the following error: {}",
                error_message(err)
            ))
        }
    }
}

impl Drop for Proj {
    fn drop(&mut self) {
        unsafe {
            proj_destroy(self.c_proj);
        }
    }
}

/// Project or inverse-project the coordinates of a `Geometry`
///
/// ```rust
/// use geo::Point;
/// use geo::algorithm::proj::{Proj, Project};
///
/// let osgb36 = Proj::new("
///     +proj=tmerc +lat_0=49 +lon_0=-2 +k=0.9996012717 +x_0=400000 +y_0=-100000 +ellps=airy
///     +towgs84=446.448,-125.157,542.06,0.15,0.247,0.842,-20.489 +units=m +no_defs
/// ").unwrap();
/// // Inverse-project OSGB36 (EPSG 27700) to Geodetic coordinates in radians
/// let p = Point::new(548295.39, 182498.46);
/// let t = p.project(&osgb36, true).unwrap();
/// assert_eq!(t.x(), 0.0023755864848281206);
/// assert_eq!(t.y(), 0.8992274896304518);
///
///```
pub trait Project<T> {
    /// Project (or inverse-projects) the geometry's coordinates using the specified
    /// PROJ operation
    fn project(&self, proj: &Proj, inverse: bool) -> Result<Self, Error>
    where
        T: CoordinateType,
        Self: Sized;
}

impl<T, G> Project<T> for G
where
    T: CoordinateType,
    G: MapCoordsFallible<T, T, Output = G>,
{
    fn project(&self, proj: &Proj, inverse: bool) -> Result<Self, Error> {
        self.map_coords_fallible(&|&(x, y)| {
            let converted = proj.project(Point::new(x, y), inverse)?;
            Ok((converted.x(), converted.y()))
        })
    }
}

/// Convert or transform the coordinates of a `Geometry` using the PROJ `pipeline`
///
/// This method makes use of the [`pipeline`](http://proj4.org/operations/pipeline.html)
/// functionality available since v5.0.0, which differs significantly from the v4.x series
///
/// It has the advantage of being able to chain an arbitrary combination of conversion
/// and transformation steps, allowing for extremely complex operations.
///
/// ```rust
/// use geo::Point;
/// use geo::algorithm::proj::{Proj, Convert};
///
/// // Carry out a conversion from NAD83 feet (EPSG 2230) to NAD83 metres (EPSG 26946)
/// let nad_ft_to_m = Proj::new("
///     +proj=pipeline
///     +step +inv +proj=lcc +lat_1=33.88333333333333
///     +lat_2=32.78333333333333 +lat_0=32.16666666666666
///     +lon_0=-116.25 +x_0=2000000.0001016 +y_0=500000.0001016001 +ellps=GRS80
///     +towgs84=0,0,0,0,0,0,0 +units=us-ft +no_defs
///     +step +proj=lcc +lat_1=33.88333333333333 +lat_2=32.78333333333333 +lat_0=32.16666666666666
///     +lon_0=-116.25 +x_0=2000000 +y_0=500000
///     +ellps=GRS80 +towgs84=0,0,0,0,0,0,0 +units=m +no_defs
/// ").unwrap();
/// let p = Point::new(4760096.421921, 3744293.729449);
/// let result = p.convert(&nad_ft_to_m).unwrap();
/// assert_eq!(result.x(), 1450880.2910605003);
/// assert_eq!(result.y(), 1141263.01116045);
/// ```
pub trait Convert<T> {
    /// Convert or transform the geometry's coordinates using the specified
    /// PROJ operation
    fn convert(&self, proj: &Proj) -> Result<Self, Error>
    where
        T: CoordinateType,
        Self: Sized;
}

impl<T, G> Convert<T> for G
where
    T: CoordinateType,
    G: MapCoordsFallible<T, T, Output = G>,
{
    fn convert(&self, proj: &Proj) -> Result<Self, Error> {
        self.map_coords_fallible(&|&(x, y)| {
            let reprojected = proj.convert(Point::new(x, y))?;
            Ok((reprojected.x(), reprojected.y()))
        })
    }
}

#[cfg(test)]
mod test {
    use super::*;

    fn assert_almost_eq(a: f64, b: f64) {
        let f: f64 = a / b;
        assert!(f < 1.00001);
        assert!(f > 0.99999);
    }
    #[test]
    fn test_definition() {
        let wgs84 = "+proj=longlat +datum=WGS84 +no_defs";
        let proj = Proj::new(wgs84).unwrap();
        assert_eq!(
            proj.def(),
            "proj=longlat datum=WGS84 no_defs ellps=WGS84 towgs84=0,0,0"
        );
    }
    #[test]
    // Carry out a projection from geodetic coordinates
    fn test_projection() {
        let stereo70 = Proj::new(
            "+proj=sterea +lat_0=46 +lon_0=25 +k=0.99975 +x_0=500000 +y_0=500000
            +ellps=krass +towgs84=33.4,-146.6,-76.3,-0.359,-0.053,0.844,-0.84 +units=m +no_defs",
        ).unwrap();
        // Geodetic -> Pulkovo 1942(58) / Stereo70 (EPSG 3844)
        let t = stereo70
            .project(Point::new(0.436332, 0.802851), false)
            .unwrap();
        assert_almost_eq(t.x(), 500119.70352012233);
        assert_almost_eq(t.y(), 500027.77896348457);
    }
    #[test]
    // Carry out an inverse projection to geodetic coordinates
    fn test_inverse_projection() {
        let stereo70 = Proj::new(
            "+proj=sterea +lat_0=46 +lon_0=25 +k=0.99975 +x_0=500000 +y_0=500000
            +ellps=krass +towgs84=33.4,-146.6,-76.3,-0.359,-0.053,0.844,-0.84 +units=m +no_defs",
        ).unwrap();
        // Pulkovo 1942(58) / Stereo70 (EPSG 3844) -> Geodetic
        let t = stereo70
            .project(Point::new(500119.70352012233, 500027.77896348457), true)
            .unwrap();
        assert_almost_eq(t.x(), 0.436332);
        assert_almost_eq(t.y(), 0.802851);
    }
    #[test]
    // Carry out an inverse projection to geodetic coordinates
    fn test_london_inverse() {
        let osgb36 = Proj::new(
            "
            +proj=tmerc +lat_0=49 +lon_0=-2 +k=0.9996012717 +x_0=400000 +y_0=-100000 +ellps=airy
            +towgs84=446.448,-125.157,542.06,0.15,0.247,0.842,-20.489 +units=m +no_defs
            ",
        ).unwrap();
        // OSGB36 (EPSG 27700) -> Geodetic
        let t = osgb36
            .project(Point::new(548295.39, 182498.46), true)
            .unwrap();
        assert_almost_eq(t.x(), 0.0023755864848281206);
        assert_almost_eq(t.y(), 0.8992274896304518);
    }
    #[test]
    // Carry out a conversion from NAD83 feet (EPSG 2230) to NAD83 metres (EPSG 26946)
    fn test_conversion() {
        let nad83_m = Proj::new("
            +proj=pipeline
            +step +inv +proj=lcc +lat_1=33.88333333333333
            +lat_2=32.78333333333333 +lat_0=32.16666666666666
            +lon_0=-116.25 +x_0=2000000.0001016 +y_0=500000.0001016001 +ellps=GRS80
            +towgs84=0,0,0,0,0,0,0 +units=us-ft +no_defs
            +step +proj=lcc +lat_1=33.88333333333333 +lat_2=32.78333333333333 +lat_0=32.16666666666666
            +lon_0=-116.25 +x_0=2000000 +y_0=500000
            +ellps=GRS80 +towgs84=0,0,0,0,0,0,0 +units=m +no_defs
        ").unwrap();
        // Presidio, San Francisco
        let t = nad83_m
            .convert(Point::new(4760096.421921, 3744293.729449))
            .unwrap();
        assert_almost_eq(t.x(), 1450880.29);
        assert_almost_eq(t.y(), 1141263.01);
    }
    #[test]
    // Test that instantiation fails wth bad proj string input
    fn test_init_error() {
        assert!(Proj::new("ugh").is_none());
    }
    #[test]
    fn test_conversion_error() {
        // because step 1 isn't an inverse conversion, it's expecting lon lat input
        let nad83_m = Proj::new("
            +proj=pipeline
            +step +proj=lcc +lat_1=33.88333333333333
            +lat_2=32.78333333333333 +lat_0=32.16666666666666
            +lon_0=-116.25 +x_0=2000000.0001016 +y_0=500000.0001016001 +ellps=GRS80
            +towgs84=0,0,0,0,0,0,0 +units=us-ft +no_defs
            +step +proj=lcc +lat_1=33.88333333333333 +lat_2=32.78333333333333 +lat_0=32.16666666666666
            +lon_0=-116.25 +x_0=2000000 +y_0=500000
            +ellps=GRS80 +towgs84=0,0,0,0,0,0,0 +units=m +no_defs
        ").unwrap();
        let err = nad83_m
            .convert(Point::new(4760096.421921, 3744293.729449))
            .unwrap_err();
        assert_eq!(
            "The conversion failed with the following error: latitude or longitude exceeded limits",
            err.root_cause().to_string()
        );
    }
    #[test]
    fn test_geometry_inverse() {
        let osgb36 = Proj::new(
            "
            +proj=tmerc +lat_0=49 +lon_0=-2 +k=0.9996012717 +x_0=400000 +y_0=-100000 +ellps=airy
            +towgs84=446.448,-125.157,542.06,0.15,0.247,0.842,-20.489 +units=m +no_defs
            ",
        ).unwrap();
        // OSGB36 (EPSG 27700) -> Geodetic
        let orig = Point::new(548295.39, 182498.46);
        let geodetic = orig.project(&osgb36, true).unwrap();
        assert_almost_eq(geodetic.x(), 0.0023755864848281206);
        assert_almost_eq(geodetic.y(), 0.8992274896304518);
    }
}
