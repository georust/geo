use super::ContainsProperly;
use super::impl_contains_properly_geometry_for;
use crate::GeoFloat;
use crate::geometry::*;
use crate::geometry_delegate_impl;

impl<T, G> ContainsProperly<G> for Geometry<T>
where
    T: GeoFloat,
    Point<T>: ContainsProperly<G>,
    MultiPoint<T>: ContainsProperly<G>,
    Line<T>: ContainsProperly<G>,
    LineString<T>: ContainsProperly<G>,
    MultiLineString<T>: ContainsProperly<G>,
    Triangle<T>: ContainsProperly<G>,
    Rect<T>: ContainsProperly<G>,
    Polygon<T>: ContainsProperly<G>,
    MultiPolygon<T>: ContainsProperly<G>,
    GeometryCollection<T>: ContainsProperly<G>,
{
    geometry_delegate_impl! {
        fn contains_properly(&self, rhs: &G) -> bool;
    }
}

impl_contains_properly_geometry_for!(Point<T>);
impl_contains_properly_geometry_for!(MultiPoint<T>);

// impl_contains_properly_geometry_for!(Line<T>);
impl_contains_properly_geometry_for!(LineString<T>);
impl_contains_properly_geometry_for!(MultiLineString<T>);

impl_contains_properly_geometry_for!(Polygon<T>);
impl_contains_properly_geometry_for!(MultiPolygon<T>);

impl_contains_properly_geometry_for!(GeometryCollection<T>);
impl_contains_properly_geometry_for!(Rect<T>);
impl_contains_properly_geometry_for!(Triangle<T>);
