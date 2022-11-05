use super::Contains;
use crate::geometry::*;
use crate::geometry_delegate_impl;
use crate::{GeoFloat, GeoNum};

impl<T> Contains<Coord<T>> for Geometry<T>
where
    T: GeoNum,
{
    fn contains(&self, coord: &Coord<T>) -> bool {
        self.contains(&Point::from(*coord))
    }
}

impl<T> Contains<Point<T>> for Geometry<T>
where
    T: GeoNum,
{
    geometry_delegate_impl! {
        fn contains(&self, point: &Point<T>) -> bool;
    }
}

impl<T> Contains<Line<T>> for Geometry<T>
where
    T: GeoFloat,
{
    geometry_delegate_impl! {
        fn contains(&self, line: &Line<T>) -> bool;
    }
}

impl<T> Contains<LineString<T>> for Geometry<T>
where
    T: GeoFloat,
{
    geometry_delegate_impl! {
        fn contains(&self, line_string: &LineString<T>) -> bool;
    }
}

impl<T> Contains<Polygon<T>> for Geometry<T>
where
    T: GeoFloat,
{
    geometry_delegate_impl! {
        fn contains(&self, polygon: &Polygon<T>) -> bool;
    }
}

impl<T> Contains<MultiPoint<T>> for Geometry<T>
where
    T: GeoFloat,
{
    geometry_delegate_impl! {
        fn contains(&self, multi_point: &MultiPoint<T>) -> bool;
    }
}

impl<T> Contains<MultiLineString<T>> for Geometry<T>
where
    T: GeoFloat,
{
    geometry_delegate_impl! {
        fn contains(&self, multi_line_string: &MultiLineString<T>) -> bool;
    }
}

impl<T> Contains<MultiPolygon<T>> for Geometry<T>
where
    T: GeoFloat,
{
    geometry_delegate_impl! {
        fn contains(&self, multi_line_string: &MultiPolygon<T>) -> bool;
    }
}

impl<T> Contains<GeometryCollection<T>> for Geometry<T>
where
    T: GeoFloat,
{
    geometry_delegate_impl! {
        fn contains(&self, geometry_collection: &GeometryCollection<T>) -> bool;
    }
}

impl<T> Contains<Rect<T>> for Geometry<T>
where
    T: GeoFloat,
{
    geometry_delegate_impl! {
        fn contains(&self, rect: &Rect<T>) -> bool;
    }
}

impl<T> Contains<Triangle<T>> for Geometry<T>
where
    T: GeoFloat,
{
    geometry_delegate_impl! {
        fn contains(&self, triangle: &Triangle<T>) -> bool;
    }
}

impl<T> Contains<Geometry<T>> for Geometry<T>
where
    T: GeoFloat,
{
    fn contains(&self, other: &Geometry<T>) -> bool {
        match other {
            Geometry::Point(geom) => self.contains(geom),
            Geometry::Line(geom) => self.contains(geom),
            Geometry::LineString(geom) => self.contains(geom),
            Geometry::Polygon(geom) => self.contains(geom),
            Geometry::MultiPoint(geom) => self.contains(geom),
            Geometry::MultiLineString(geom) => self.contains(geom),
            Geometry::MultiPolygon(geom) => self.contains(geom),
            Geometry::GeometryCollection(geom) => self.contains(geom),
            Geometry::Rect(geom) => self.contains(geom),
            Geometry::Triangle(geom) => self.contains(geom),
        }
    }
}
