use crate::{
    CoordinateType, GeometryCollection, Line, LineString, MultiLineString, MultiPoint,
    MultiPolygon, Point, Polygon, Rect, Triangle,
};
use num_traits::Float;
use std::convert::TryFrom;
use std::error::Error;
use std::fmt;

/// An enum representing any possible geometry type.
///
/// All `Geo` types can be converted to a `Geometry` member using `.into()` (as part of the
/// `std::convert::Into` pattern), and `Geo` types implement the `TryFrom` trait in order to
/// convert _back_ from enum members.
///
/// # Example
///
/// ```
/// use std::convert::TryFrom;
/// use geo_types::{Point, point, Geometry, GeometryCollection};
/// let p = point!(x: 1.0, y: 1.0);
/// let pe: Geometry<f64> = p.into();
/// let pn = Point::try_from(pe).unwrap();
/// ```
///
#[derive(Eq, PartialEq, Clone, Debug, Hash)]
pub enum Geometry<T>
where
    T: CoordinateType,
{
    Point(Point<T>),
    Line(Line<T>),
    LineString(LineString<T>),
    Polygon(Polygon<T>),
    MultiPoint(MultiPoint<T>),
    MultiLineString(MultiLineString<T>),
    MultiPolygon(MultiPolygon<T>),
    GeometryCollection(GeometryCollection<T>),
    Rect(Rect<T>),
    Triangle(Triangle<T>),
}

impl<T: CoordinateType> From<Point<T>> for Geometry<T> {
    fn from(x: Point<T>) -> Geometry<T> {
        Geometry::Point(x)
    }
}
impl<T: CoordinateType> From<Line<T>> for Geometry<T> {
    fn from(x: Line<T>) -> Geometry<T> {
        Geometry::Line(x)
    }
}
impl<T: CoordinateType> From<LineString<T>> for Geometry<T> {
    fn from(x: LineString<T>) -> Geometry<T> {
        Geometry::LineString(x)
    }
}
impl<T: CoordinateType> From<Polygon<T>> for Geometry<T> {
    fn from(x: Polygon<T>) -> Geometry<T> {
        Geometry::Polygon(x)
    }
}
impl<T: CoordinateType> From<MultiPoint<T>> for Geometry<T> {
    fn from(x: MultiPoint<T>) -> Geometry<T> {
        Geometry::MultiPoint(x)
    }
}
impl<T: CoordinateType> From<MultiLineString<T>> for Geometry<T> {
    fn from(x: MultiLineString<T>) -> Geometry<T> {
        Geometry::MultiLineString(x)
    }
}
impl<T: CoordinateType> From<MultiPolygon<T>> for Geometry<T> {
    fn from(x: MultiPolygon<T>) -> Geometry<T> {
        Geometry::MultiPolygon(x)
    }
}

impl<T: CoordinateType> From<Rect<T>> for Geometry<T> {
    fn from(x: Rect<T>) -> Geometry<T> {
        Geometry::Rect(x)
    }
}

impl<T: CoordinateType> From<Triangle<T>> for Geometry<T> {
    fn from(x: Triangle<T>) -> Geometry<T> {
        Geometry::Triangle(x)
    }
}

impl<T: CoordinateType> Geometry<T> {
    /// If this Geometry is a Point, then return that, else None.
    ///
    /// # Examples
    ///
    /// ```
    /// use geo_types::*;
    /// let g = Geometry::Point(Point::new(0., 0.));
    /// let p2: Point<f32> = g.into_point().unwrap();
    /// assert_eq!(p2, Point::new(0., 0.,));
    /// ```
    #[deprecated(
        note = "Will be removed in an upcoming version. Switch to std::convert::TryInto<Point>"
    )]
    pub fn into_point(self) -> Option<Point<T>> {
        if let Geometry::Point(x) = self {
            Some(x)
        } else {
            None
        }
    }

    /// If this Geometry is a LineString, then return that LineString, else None.
    #[deprecated(
        note = "Will be removed in an upcoming version. Switch to std::convert::TryInto<LineString>"
    )]
    pub fn into_line_string(self) -> Option<LineString<T>> {
        if let Geometry::LineString(x) = self {
            Some(x)
        } else {
            None
        }
    }

    /// If this Geometry is a Line, then return that Line, else None.
    #[deprecated(
        note = "Will be removed in an upcoming version. Switch to std::convert::TryInto<Line>"
    )]
    pub fn into_line(self) -> Option<Line<T>> {
        if let Geometry::Line(x) = self {
            Some(x)
        } else {
            None
        }
    }

    /// If this Geometry is a Polygon, then return that, else None.
    #[deprecated(
        note = "Will be removed in an upcoming version. Switch to std::convert::TryInto<Polygon>"
    )]
    pub fn into_polygon(self) -> Option<Polygon<T>> {
        if let Geometry::Polygon(x) = self {
            Some(x)
        } else {
            None
        }
    }

    /// If this Geometry is a MultiPoint, then return that, else None.
    #[deprecated(
        note = "Will be removed in an upcoming version. Switch to std::convert::TryInto<MultiPoint>"
    )]
    pub fn into_multi_point(self) -> Option<MultiPoint<T>> {
        if let Geometry::MultiPoint(x) = self {
            Some(x)
        } else {
            None
        }
    }

    /// If this Geometry is a MultiLineString, then return that, else None.
    #[deprecated(
        note = "Will be removed in an upcoming version. Switch to std::convert::TryInto<MultiLineString>"
    )]
    pub fn into_multi_line_string(self) -> Option<MultiLineString<T>> {
        if let Geometry::MultiLineString(x) = self {
            Some(x)
        } else {
            None
        }
    }

    /// If this Geometry is a MultiPolygon, then return that, else None.
    #[deprecated(
        note = "Will be removed in an upcoming version. Switch to std::convert::TryInto<MultiPolygon>"
    )]
    pub fn into_multi_polygon(self) -> Option<MultiPolygon<T>> {
        if let Geometry::MultiPolygon(x) = self {
            Some(x)
        } else {
            None
        }
    }

    /// Return the number of coordinates in the `Geometry`.
    pub fn num_coords(&self) -> usize {
        match self {
            Geometry::Point(g) => g.num_coords(),
            Geometry::Line(g) => g.num_coords(),
            Geometry::LineString(g) => g.num_coords(),
            Geometry::Polygon(g) => g.num_coords(),
            Geometry::MultiPoint(g) => g.num_coords(),
            Geometry::MultiLineString(g) => g.num_coords(),
            Geometry::MultiPolygon(g) => g.num_coords(),
            Geometry::GeometryCollection(g) => g.num_coords(),
            Geometry::Rect(g) => g.num_coords(),
            Geometry::Triangle(g) => g.num_coords(),
        }
    }
}

#[derive(Debug)]
pub struct FailedToConvertError;

impl fmt::Display for FailedToConvertError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Could not convert from enum member to concrete type")
    }
}

impl Error for FailedToConvertError {
    fn description(&self) -> &str {
        "Could not convert from enum member to concrete type"
    }
}

impl<T> TryFrom<Geometry<T>> for Point<T>
where
    T: Float,
{
    type Error = FailedToConvertError;

    fn try_from(geom: Geometry<T>) -> Result<Point<T>, Self::Error> {
        match geom {
            Geometry::Point(p) => Ok(p),
            _ => Err(FailedToConvertError),
        }
    }
}

impl<T> TryFrom<Geometry<T>> for Line<T>
where
    T: Float,
{
    type Error = FailedToConvertError;

    fn try_from(geom: Geometry<T>) -> Result<Line<T>, Self::Error> {
        match geom {
            Geometry::Line(l) => Ok(l),
            _ => Err(FailedToConvertError),
        }
    }
}

impl<T> TryFrom<Geometry<T>> for LineString<T>
where
    T: Float,
{
    type Error = FailedToConvertError;

    fn try_from(geom: Geometry<T>) -> Result<LineString<T>, Self::Error> {
        match geom {
            Geometry::LineString(ls) => Ok(ls),
            _ => Err(FailedToConvertError),
        }
    }
}

impl<T> TryFrom<Geometry<T>> for Polygon<T>
where
    T: Float,
{
    type Error = FailedToConvertError;

    fn try_from(geom: Geometry<T>) -> Result<Polygon<T>, Self::Error> {
        match geom {
            Geometry::Polygon(ls) => Ok(ls),
            _ => Err(FailedToConvertError),
        }
    }
}

impl<T> TryFrom<Geometry<T>> for MultiPoint<T>
where
    T: Float,
{
    type Error = FailedToConvertError;

    fn try_from(geom: Geometry<T>) -> Result<MultiPoint<T>, Self::Error> {
        match geom {
            Geometry::MultiPoint(mp) => Ok(mp),
            _ => Err(FailedToConvertError),
        }
    }
}

impl<T> TryFrom<Geometry<T>> for MultiLineString<T>
where
    T: Float,
{
    type Error = FailedToConvertError;

    fn try_from(geom: Geometry<T>) -> Result<MultiLineString<T>, Self::Error> {
        match geom {
            Geometry::MultiLineString(mls) => Ok(mls),
            _ => Err(FailedToConvertError),
        }
    }
}

impl<T> TryFrom<Geometry<T>> for MultiPolygon<T>
where
    T: Float,
{
    type Error = FailedToConvertError;

    fn try_from(geom: Geometry<T>) -> Result<MultiPolygon<T>, Self::Error> {
        match geom {
            Geometry::MultiPolygon(mp) => Ok(mp),
            _ => Err(FailedToConvertError),
        }
    }
}
