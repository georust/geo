use std::{convert::TryInto, error::Error, fmt};

/// Transform a geometry using PROJ.
pub trait Transform<T> {
    type Output;

    /// Transform a geometry.
    ///
    /// # Examples
    ///
    /// Transform a geometry using a PROJ string definition:
    ///
    /// ```
    /// use geo::{self, prelude::*};
    ///
    /// let point = geo::point!(x: -36.508f32, y: -54.2815f32);
    ///
    /// assert_eq!(
    ///     point.transform("+proj=axisswap +order=2,1,3,4").unwrap(),
    ///     geo::point!(x: -54.2815f32, y: -36.508f32)
    /// );
    /// ```
    ///
    /// Transform a geometry from one CRS to another CRS:
    ///
    /// ```
    /// use geo::{self, prelude::*};
    ///
    /// let point = geo::point!(x: -36.508f32, y: -54.2815f32);
    ///
    /// assert_eq!(
    ///     point.transform(("EPSG:4326", "EPSG:3857")).unwrap(),
    ///     geo::point!(x: -4064052.0f32, y: -7223650.5f32)
    /// );
    /// ```
    fn transform(
        &self,
        proj: impl TryInto<proj::Proj, Error = proj::ProjCreateError>,
    ) -> Result<Self::Output, TransformError>;

    /// Transform a geometry from one CRS to another CRS.
    ///
    /// # Examples
    ///
    /// ```
    /// use geo::{self, prelude::*};
    ///
    /// let point: geo::Point<f32> = geo::point!(x: -36.508f32, y: -54.2815f32);
    ///
    /// assert_eq!(
    ///     point.transform_crs_to_crs("EPSG:4326", "EPSG:3857").unwrap(),
    ///     geo::point!(x: -4064052.0f32, y: -7223650.5f32)
    /// );
    /// ```
    fn transform_crs_to_crs(
        &self,
        source_crs: &str,
        target_crs: &str,
    ) -> Result<Self::Output, TransformError>;
}

impl<T, G> Transform<T> for G
where
    T: crate::CoordFloat,
    G: crate::algorithm::map_coords::TryMapCoords<T, T, proj::ProjError>,
{
    type Output = G::Output;

    fn transform(
        &self,
        proj: impl TryInto<proj::Proj, Error = proj::ProjCreateError>,
    ) -> Result<Self::Output, TransformError> {
        let transformer: proj::Proj = proj.try_into()?;
        let result: Result<G::Output, proj::ProjError> =
            self.try_map_coords(|&(x, y)| transformer.convert((x, y)));
        Ok(result?)
    }

    fn transform_crs_to_crs(
        &self,
        source_crs: &str,
        target_crs: &str,
    ) -> Result<Self::Output, TransformError> {
        self.transform((source_crs, target_crs))
    }
}

#[derive(Debug)]
pub enum TransformError {
    UnknownCrs,
    ProjCreateError(proj::ProjCreateError),
    ProjError(proj::ProjError),
}

impl From<proj::ProjError> for TransformError {
    fn from(e: proj::ProjError) -> Self {
        TransformError::ProjError(e)
    }
}

impl From<proj::ProjCreateError> for TransformError {
    fn from(e: proj::ProjCreateError) -> Self {
        TransformError::ProjCreateError(e)
    }
}

impl fmt::Display for TransformError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            TransformError::UnknownCrs => write!(f, "Unknown CRS"),
            TransformError::ProjCreateError(err) => err.fmt(f),
            TransformError::ProjError(err) => err.fmt(f),
        }
    }
}

impl Error for TransformError {}
