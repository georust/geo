use crate::monotone_chain::geometry::*;
use crate::{ContainsProperly, GeoNum};
use geo_types::geometry::*;

impl_contains_properly_target_monotone!(MonotoneChainMultiLineString<'a, T>, [
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
    GeometryCollection<T>
]);

impl_contains_properly_for_monotone!(MonotoneChainMultiLineString<'a, T>, [
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

    Geometry<T>,
    GeometryCollection<T>,

    MonotoneChainLineString<'a, T>,
    MonotoneChainMultiLineString<'a, T>,
    MonotoneChainPolygon<'a, T>,
    MonotoneChainMultiPolygon<'a, T>
]);
