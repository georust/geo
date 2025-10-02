use crate::geometry::*;
use super::{ContainsProperly};
use crate::{GeoFloat};
use crate::geometry_delegate_impl;
use super::impl_contains_properly_geometry_for;


impl<T,G> ContainsProperly<G> for Geometry<T>
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
    // G: BoundingRect<T>,

{
    geometry_delegate_impl! {
        fn contains_properly(&self, rhs: &G) -> bool;
    }
}


impl_contains_properly_geometry_for!(Point<T>);
impl_contains_properly_geometry_for!(MultiPoint<T>);

impl_contains_properly_geometry_for!(Line<T>);
impl_contains_properly_geometry_for!(LineString<T>);
impl_contains_properly_geometry_for!(MultiLineString<T>);

impl_contains_properly_geometry_for!(Polygon<T>);
impl_contains_properly_geometry_for!(MultiPolygon<T>);

impl_contains_properly_geometry_for!(GeometryCollection<T>);
impl_contains_properly_geometry_for!(Rect<T>);
impl_contains_properly_geometry_for!(Triangle<T>);



// impl<T> ContainsProperly<Geometry<T>> for Geometry<T>
// where
//     T: GeoFloat,
// {
//     fn contains_properly(&self, other: &Geometry<T>) -> bool {
//         match other {
//             Geometry::Point(geom) => self.contains_properly(geom),
//             Geometry::Line(geom) => self.contains_properly(geom),
//             Geometry::LineString(geom) => self.contains_properly(geom),
//             Geometry::Polygon(geom) => self.contains_properly(geom),
//             Geometry::MultiPoint(geom) => self.contains_properly(geom),
//             Geometry::MultiLineString(geom) => self.contains_properly(geom),
//             Geometry::MultiPolygon(geom) => self.contains_properly(geom),
//             Geometry::GeometryCollection(geom) => self.contains_properly(geom),
//             Geometry::Rect(geom) => self.contains_properly(geom),
//             Geometry::Triangle(geom) => self.contains_properly(geom),
//         }
//     }
// }
