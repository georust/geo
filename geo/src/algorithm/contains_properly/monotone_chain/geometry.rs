use crate::geometry::*;
use crate::monotone_chain::geometry::*;
use crate::{ContainsProperly, GeoFloat, GeoNum};

impl<'a, G, T> ContainsProperly<G> for MonotoneChainGeometry<'a, T>
where
    T: GeoNum,
    MonotoneChainLineString<'a, T>: ContainsProperly<G>,
    MonotoneChainMultiLineString<'a, T>: ContainsProperly<G>,
    MonotoneChainPolygon<'a, T>: ContainsProperly<G>,
    MonotoneChainMultiPolygon<'a, T>: ContainsProperly<G>,
{
    fn contains_properly(&self, rhs: &G) -> bool {
        match self {
            MonotoneChainGeometry::LineString(g) => g.contains_properly(rhs),
            MonotoneChainGeometry::MultiLineString(g) => g.contains_properly(rhs),
            MonotoneChainGeometry::Polygon(g) => g.contains_properly(rhs),
            MonotoneChainGeometry::MultiPolygon(g) => g.contains_properly(rhs),
        }
    }
}
macro_rules! impl_contains_properly_target_monotone_geometry {
    ([$($for:ty),*]) => {
        $(
            impl<'a, T> ContainsProperly<MonotoneChainGeometry<'a, T>> for $for
            where
                T: GeoFloat,
                $for:  ContainsProperly<MonotoneChainLineString<'a, T>> +
                    ContainsProperly<MonotoneChainMultiLineString<'a, T>> +
                    ContainsProperly<MonotoneChainPolygon<'a, T>> +
                    ContainsProperly<MonotoneChainMultiPolygon<'a, T>>
            {
                fn contains_properly(&self, rhs: &MonotoneChainGeometry<'a, T>) -> bool {
                    match rhs {
                        MonotoneChainGeometry::LineString(rhs) => self.contains_properly(rhs),
                        MonotoneChainGeometry::MultiLineString(rhs) => self.contains_properly(rhs),
                        MonotoneChainGeometry::Polygon(rhs) => self.contains_properly(rhs),
                        MonotoneChainGeometry::MultiPolygon(rhs) => self.contains_properly(rhs),
                    }
                }
            }
        )*
    };
}

impl_contains_properly_target_monotone_geometry!([
    Coord<T>,
    Point<T>,
    MultiPoint<T>,

    Line<T>,
    LineString<T>,
    MultiLineString<T>,

    Polygon<T>,
    MultiPolygon<T>,
    Rect<T>,
    Triangle<T>,

    // Geometry<T>,
    GeometryCollection<T>,

    MonotoneChainGeometry<'a, T>
]);
