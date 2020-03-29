use crate::{
    CoordinateType, GeometryCollection, Line, LineString, MultiLineString, MultiPoint,
    MultiPolygon, Point, Polygon, Rect, Triangle,
};
use num_traits::Float;
use std::borrow::Cow;
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
#[derive(PartialEq, Clone, Debug, Hash)]
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

#[derive(PartialEq, Debug, Hash)]
pub enum GeometryCow<'a, T>
where
    T: CoordinateType,
{
    Point(Cow<'a, Point<T>>),
    Line(Cow<'a, Line<T>>),
    LineString(Cow<'a, LineString<T>>),
    Polygon(Cow<'a, Polygon<T>>),
    MultiPoint(Cow<'a, MultiPoint<T>>),
    MultiLineString(Cow<'a, MultiLineString<T>>),
    MultiPolygon(Cow<'a, MultiPolygon<T>>),
    GeometryCollection(Cow<'a, GeometryCollection<T>>),
    Rect(Cow<'a, Rect<T>>),
    Triangle(Cow<'a, Triangle<T>>),
}

impl<'a, T: CoordinateType> From<&'a Geometry<T>> for GeometryCow<'a, T> {
    fn from(geometry: &'a Geometry<T>) -> Self {
        match geometry {
            Geometry::Point(g) => GeometryCow::Point(Cow::Borrowed(g)),
            Geometry::Line(g) => GeometryCow::Line(Cow::Borrowed(g)),
            Geometry::LineString(g) => GeometryCow::LineString(Cow::Borrowed(g)),
            Geometry::Polygon(g) => GeometryCow::Polygon(Cow::Borrowed(g)),
            Geometry::MultiPoint(g) => GeometryCow::MultiPoint(Cow::Borrowed(g)),
            Geometry::MultiLineString(g) => GeometryCow::MultiLineString(Cow::Borrowed(g)),
            Geometry::MultiPolygon(g) => GeometryCow::MultiPolygon(Cow::Borrowed(g)),
            Geometry::GeometryCollection(g) => GeometryCow::GeometryCollection(Cow::Borrowed(g)),
            Geometry::Rect(g) => GeometryCow::Rect(Cow::Borrowed(g)),
            Geometry::Triangle(g) => GeometryCow::Triangle(Cow::Borrowed(g)),
        }
    }
}

impl<'a, T: CoordinateType> From<&'a Point<T>> for GeometryCow<'a, T> {
    fn from(point: &'a Point<T>) -> Self {
        GeometryCow::Point(Cow::Borrowed(point))
    }
}

impl<'a, T: CoordinateType> From<&'a LineString<T>> for GeometryCow<'a, T> {
    fn from(line_string: &'a LineString<T>) -> Self {
        GeometryCow::LineString(Cow::Borrowed(line_string))
    }
}

impl<'a, T: CoordinateType> From<&'a Line<T>> for GeometryCow<'a, T> {
    fn from(line: &'a Line<T>) -> Self {
        GeometryCow::Line(Cow::Borrowed(line))
    }
}

impl<'a, T: CoordinateType> From<&'a Polygon<T>> for GeometryCow<'a, T> {
    fn from(polygon: &'a Polygon<T>) -> Self {
        GeometryCow::Polygon(Cow::Borrowed(polygon))
    }
}

impl<'a, T: CoordinateType> From<&'a MultiPoint<T>> for GeometryCow<'a, T> {
    fn from(multi_point: &'a MultiPoint<T>) -> GeometryCow<'a, T> {
        GeometryCow::MultiPoint(Cow::Borrowed(multi_point))
    }
}

impl<'a, T: CoordinateType> From<&'a MultiLineString<T>> for GeometryCow<'a, T> {
    fn from(multi_line_string: &'a MultiLineString<T>) -> Self {
        GeometryCow::MultiLineString(Cow::Borrowed(multi_line_string))
    }
}

impl<'a, T: CoordinateType> From<&'a MultiPolygon<T>> for GeometryCow<'a, T> {
    fn from(multi_polygon: &'a MultiPolygon<T>) -> Self {
        GeometryCow::MultiPolygon(Cow::Borrowed(multi_polygon))
    }
}

impl<'a, T: CoordinateType> From<&'a GeometryCollection<T>> for GeometryCow<'a, T> {
    fn from(geometry_collection: &'a GeometryCollection<T>) -> Self {
        GeometryCow::GeometryCollection(Cow::Borrowed(geometry_collection))
    }
}

impl<'a, T: CoordinateType> From<&'a Rect<T>> for GeometryCow<'a, T> {
    fn from(rect: &'a Rect<T>) -> Self {
        GeometryCow::Rect(Cow::Borrowed(rect))
    }
}

impl<'a, T: CoordinateType> From<&'a Triangle<T>> for GeometryCow<'a, T> {
    fn from(triangle: &'a Triangle<T>) -> Self {
        GeometryCow::Triangle(Cow::Borrowed(triangle))
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
