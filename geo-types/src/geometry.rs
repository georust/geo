use crate::{
    CoordinateType, GeometryCollection, Line, LineString, MultiLineString, MultiPoint,
    MultiPolygon, Point, Polygon, Rect, Triangle,
};
use num_traits::Float;
use std::borrow::{Borrow, Cow};
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

pub enum GeometryIsh<'a, T: CoordinateType> {
    Owned(Geometry<T>),
    Borrowed(GeometryRef<'a, T>),
}

// TODO: prefer From instead of Into

impl<'a, T: CoordinateType> Into<GeometryIsh<'a, T>> for &'a Geometry<T> {
    fn into(self) -> GeometryIsh<'a, T> {
        match self {
            Geometry::Point(g) => GeometryIsh::Borrowed(GeometryRef::Point(g)),
            Geometry::Line(g) => GeometryIsh::Borrowed(GeometryRef::Line(g)),
            Geometry::LineString(g) => GeometryIsh::Borrowed(GeometryRef::LineString(g)),
            Geometry::Polygon(g) => GeometryIsh::Borrowed(GeometryRef::Polygon(g)),
            Geometry::MultiPoint(g) => GeometryIsh::Borrowed(GeometryRef::MultiPoint(g)),
            Geometry::MultiLineString(g) => GeometryIsh::Borrowed(GeometryRef::MultiLineString(g)),
            Geometry::MultiPolygon(g) => GeometryIsh::Borrowed(GeometryRef::MultiPolygon(g)),
            Geometry::GeometryCollection(g) => {
                GeometryIsh::Borrowed(GeometryRef::GeometryCollection(g))
            }
            Geometry::Rect(g) => GeometryIsh::Borrowed(GeometryRef::Rect(g)),
            Geometry::Triangle(g) => GeometryIsh::Borrowed(GeometryRef::Triangle(g)),
        }
    }
}

impl<'a, T: CoordinateType> Into<GeometryIsh<'a, T>> for &'a Point<T> {
    fn into(self) -> GeometryIsh<'a, T> {
        GeometryIsh::Borrowed(GeometryRef::Point(self))
    }
}

impl<'a, T: CoordinateType> Into<GeometryIsh<'a, T>> for &'a LineString<T> {
    fn into(self) -> GeometryIsh<'a, T> {
        GeometryIsh::Borrowed(GeometryRef::LineString(self))
    }
}

impl<'a, T: CoordinateType> Into<GeometryIsh<'a, T>> for &'a Line<T> {
    fn into(self) -> GeometryIsh<'a, T> {
        GeometryIsh::Borrowed(GeometryRef::Line(self))
    }
}

impl<'a, T: CoordinateType> Into<GeometryIsh<'a, T>> for &'a Polygon<T> {
    fn into(self) -> GeometryIsh<'a, T> {
        GeometryIsh::Borrowed(GeometryRef::Polygon(self))
    }
}

impl<'a, T: CoordinateType> Into<GeometryIsh<'a, T>> for &'a MultiPoint<T> {
    fn into(self) -> GeometryIsh<'a, T> {
        GeometryIsh::Borrowed(GeometryRef::MultiPoint(self))
    }
}

impl<'a, T: CoordinateType> Into<GeometryIsh<'a, T>> for &'a MultiLineString<T> {
    fn into(self) -> GeometryIsh<'a, T> {
        GeometryIsh::Borrowed(GeometryRef::MultiLineString(self))
    }
}

impl<'a, T: CoordinateType> Into<GeometryIsh<'a, T>> for &'a MultiPolygon<T> {
    fn into(self) -> GeometryIsh<'a, T> {
        GeometryIsh::Borrowed(GeometryRef::MultiPolygon(self))
    }
}

impl<'a, T: CoordinateType> Into<GeometryIsh<'a, T>> for &'a GeometryCollection<T> {
    fn into(self) -> GeometryIsh<'a, T> {
        GeometryIsh::Borrowed(GeometryRef::GeometryCollection(self))
    }
}

impl<'a, T: CoordinateType> Into<GeometryIsh<'a, T>> for &'a Rect<T> {
    fn into(self) -> GeometryIsh<'a, T> {
        GeometryIsh::Borrowed(GeometryRef::Rect(self))
    }
}

impl<'a, T: CoordinateType> Into<GeometryIsh<'a, T>> for &'a Triangle<T> {
    fn into(self) -> GeometryIsh<'a, T> {
        GeometryIsh::Borrowed(GeometryRef::Triangle(self))
    }
}

// impl<'a, T: CoordinateType> Borrow<GeometryRef<'a, T>> for Geometry<T> {
//     fn borrow(&self) -> &GeometryRef<'a, T> {
//        unimplemented!()
//     }
// }

// impl<'a, T: CoordinateType> ToOwned for GeometryRef<'a, T> {
//     type Owned = Geometry<T>;

//     fn to_owned(&self) -> Geometry<T> {
//        match self {

//        }
//     }
// }

// TODO impl deref? or borrow? or as_ref?

#[derive(PartialEq, Debug, Hash)]
pub enum GeometryRef<'a, T>
where
    T: CoordinateType,
{
    Point(&'a Point<T>),
    Line(&'a Line<T>),
    LineString(&'a LineString<T>),
    Polygon(&'a Polygon<T>),
    MultiPoint(&'a MultiPoint<T>),
    MultiLineString(&'a MultiLineString<T>),
    MultiPolygon(&'a MultiPolygon<T>),
    GeometryCollection(&'a GeometryCollection<T>),
    Rect(&'a Rect<T>),
    Triangle(&'a Triangle<T>),
}

impl<'a, T: 'a + CoordinateType> From<&'a Point<T>> for GeometryRef<'a, T> {
    fn from(x: &'a Point<T>) -> GeometryRef<'a, T> {
        GeometryRef::Point(x)
    }
}

impl<'a, T: 'a + CoordinateType> From<&'a Line<T>> for GeometryRef<'a, T> {
    fn from(x: &'a Line<T>) -> GeometryRef<'a, T> {
        GeometryRef::Line(x)
    }
}

impl<'a, T: 'a + CoordinateType> From<&'a LineString<T>> for GeometryRef<'a, T> {
    fn from(x: &'a LineString<T>) -> GeometryRef<'a, T> {
        GeometryRef::LineString(x)
    }
}

impl<'a, T: 'a + CoordinateType> From<&'a Polygon<T>> for GeometryRef<'a, T> {
    fn from(x: &'a Polygon<T>) -> GeometryRef<'a, T> {
        GeometryRef::Polygon(x)
    }
}

impl<'a, T: 'a + CoordinateType> From<&'a MultiPoint<T>> for GeometryRef<'a, T> {
    fn from(x: &'a MultiPoint<T>) -> GeometryRef<'a, T> {
        GeometryRef::MultiPoint(x)
    }
}

impl<'a, T: 'a + CoordinateType> From<&'a MultiLineString<T>> for GeometryRef<'a, T> {
    fn from(x: &'a MultiLineString<T>) -> GeometryRef<'a, T> {
        GeometryRef::MultiLineString(x)
    }
}

impl<'a, T: 'a + CoordinateType> From<&'a MultiPolygon<T>> for GeometryRef<'a, T> {
    fn from(x: &'a MultiPolygon<T>) -> GeometryRef<'a, T> {
        GeometryRef::MultiPolygon(x)
    }
}

impl<'a, T: 'a + CoordinateType> From<&'a GeometryCollection<T>> for GeometryRef<'a, T> {
    fn from(x: &'a GeometryCollection<T>) -> GeometryRef<'a, T> {
        GeometryRef::GeometryCollection(x)
    }
}

impl<'a, T: 'a + CoordinateType> From<&'a Rect<T>> for GeometryRef<'a, T> {
    fn from(x: &'a Rect<T>) -> GeometryRef<'a, T> {
        GeometryRef::Rect(x)
    }
}

impl<'a, T: 'a + CoordinateType> From<&'a Triangle<T>> for GeometryRef<'a, T> {
    fn from(x: &'a Triangle<T>) -> GeometryRef<'a, T> {
        GeometryRef::Triangle(x)
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
