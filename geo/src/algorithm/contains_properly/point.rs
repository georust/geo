use crate::geometry::*;
use super::{ContainsProperly, impl_contains_properly_from_relate};
use crate::{GeoFloat};

impl_contains_properly_from_relate!(Point<T>, [
    Point<T>,MultiPoint<T>, 
    Line<T>, LineString<T>, MultiLineString<T>,
    Polygon<T>,MultiPolygon<T>,
    GeometryCollection<T>,
    Rect<T>,Triangle<T>
    ]);

impl_contains_properly_from_relate!(MultiPoint<T>, [
    Point<T>,MultiPoint<T>, 
    Line<T>, LineString<T>, MultiLineString<T>,
    Polygon<T>,MultiPolygon<T>,
    GeometryCollection<T>,
    Rect<T>,Triangle<T>
    ]);