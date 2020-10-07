use crate::{
    CoordNum, Geometry, GeometryCollection, Line, LineString, MultiLineString, MultiPoint,
    MultiPolygon, Point, Polygon, Rect, Triangle,
};
use std::borrow::Cow;

/// A `GeometryCow` is a "one of" enum, just like [`Geometry`], except it is possible for the inner
/// type of a `GeometryCow` to be a reference rather than owned.
///
/// This is a way to "upgrade" an inner type to something like a `Geometry` without `moving` it.
///
/// As an example, see the [`Relate`] trait which uses `GeometryCow`.
#[derive(PartialEq, Debug, Hash)]
pub(crate) enum GeometryCow<'a, T>
where
    T: CoordNum,
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

impl<'a, T: CoordNum> From<&'a Geometry<T>> for GeometryCow<'a, T> {
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

impl<'a, T: CoordNum> From<&'a Point<T>> for GeometryCow<'a, T> {
    fn from(point: &'a Point<T>) -> Self {
        GeometryCow::Point(Cow::Borrowed(point))
    }
}

impl<'a, T: CoordNum> From<&'a LineString<T>> for GeometryCow<'a, T> {
    fn from(line_string: &'a LineString<T>) -> Self {
        GeometryCow::LineString(Cow::Borrowed(line_string))
    }
}

impl<'a, T: CoordNum> From<&'a Line<T>> for GeometryCow<'a, T> {
    fn from(line: &'a Line<T>) -> Self {
        GeometryCow::Line(Cow::Borrowed(line))
    }
}

impl<'a, T: CoordNum> From<&'a Polygon<T>> for GeometryCow<'a, T> {
    fn from(polygon: &'a Polygon<T>) -> Self {
        GeometryCow::Polygon(Cow::Borrowed(polygon))
    }
}

impl<'a, T: CoordNum> From<&'a MultiPoint<T>> for GeometryCow<'a, T> {
    fn from(multi_point: &'a MultiPoint<T>) -> GeometryCow<'a, T> {
        GeometryCow::MultiPoint(Cow::Borrowed(multi_point))
    }
}

impl<'a, T: CoordNum> From<&'a MultiLineString<T>> for GeometryCow<'a, T> {
    fn from(multi_line_string: &'a MultiLineString<T>) -> Self {
        GeometryCow::MultiLineString(Cow::Borrowed(multi_line_string))
    }
}

impl<'a, T: CoordNum> From<&'a MultiPolygon<T>> for GeometryCow<'a, T> {
    fn from(multi_polygon: &'a MultiPolygon<T>) -> Self {
        GeometryCow::MultiPolygon(Cow::Borrowed(multi_polygon))
    }
}

impl<'a, T: CoordNum> From<&'a GeometryCollection<T>> for GeometryCow<'a, T> {
    fn from(geometry_collection: &'a GeometryCollection<T>) -> Self {
        GeometryCow::GeometryCollection(Cow::Borrowed(geometry_collection))
    }
}

impl<'a, T: CoordNum> From<&'a Rect<T>> for GeometryCow<'a, T> {
    fn from(rect: &'a Rect<T>) -> Self {
        GeometryCow::Rect(Cow::Borrowed(rect))
    }
}

impl<'a, T: CoordNum> From<&'a Triangle<T>> for GeometryCow<'a, T> {
    fn from(triangle: &'a Triangle<T>) -> Self {
        GeometryCow::Triangle(Cow::Borrowed(triangle))
    }
}
