use super::Covers;
use crate::GeoFloat;
use crate::geometry::*;
use crate::geometry_delegate_impl;

impl<T> Covers<Point<T>> for Geometry<T>
where
    T: GeoFloat,
{
    geometry_delegate_impl! {
        fn covers(&self, point: &Point<T>) -> bool;
    }
}

impl<T> Covers<Line<T>> for Geometry<T>
where
    T: GeoFloat,
{
    geometry_delegate_impl! {
        fn covers(&self, line: &Line<T>) -> bool;
    }
}

impl<T> Covers<LineString<T>> for Geometry<T>
where
    T: GeoFloat,
{
    geometry_delegate_impl! {
        fn covers(&self, line_string: &LineString<T>) -> bool;
    }
}

impl<T> Covers<Polygon<T>> for Geometry<T>
where
    T: GeoFloat,
{
    geometry_delegate_impl! {
        fn covers(&self, polygon: &Polygon<T>) -> bool;
    }
}

impl<T> Covers<MultiPoint<T>> for Geometry<T>
where
    T: GeoFloat,
{
    geometry_delegate_impl! {
        fn covers(&self, multi_point: &MultiPoint<T>) -> bool;
    }
}

impl<T> Covers<MultiLineString<T>> for Geometry<T>
where
    T: GeoFloat,
{
    geometry_delegate_impl! {
        fn covers(&self, multi_line_string: &MultiLineString<T>) -> bool;
    }
}

impl<T> Covers<MultiPolygon<T>> for Geometry<T>
where
    T: GeoFloat,
{
    geometry_delegate_impl! {
        fn covers(&self, multi_line_string: &MultiPolygon<T>) -> bool;
    }
}

impl<T> Covers<GeometryCollection<T>> for Geometry<T>
where
    T: GeoFloat,
{
    geometry_delegate_impl! {
        fn covers(&self, geometry_collection: &GeometryCollection<T>) -> bool;
    }
}

impl<T> Covers<Rect<T>> for Geometry<T>
where
    T: GeoFloat,
{
    geometry_delegate_impl! {
        fn covers(&self, rect: &Rect<T>) -> bool;
    }
}

impl<T> Covers<Triangle<T>> for Geometry<T>
where
    T: GeoFloat,
{
    geometry_delegate_impl! {
        fn covers(&self, triangle: &Triangle<T>) -> bool;
    }
}

impl<T> Covers<Geometry<T>> for Geometry<T>
where
    T: GeoFloat,
{
    geometry_delegate_impl! {
        fn covers(&self, other: &Geometry<T>) -> bool;
    }
}
