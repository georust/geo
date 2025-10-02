use crate::geometry::*;
use super::{ContainsProperly, impl_contains_properly_from_relate};
use crate::{GeoFloat};

impl_contains_properly_from_relate!(Triangle<T>, [
    Point<T>,MultiPoint<T>, 
    Line<T>, LineString<T>, MultiLineString<T>,
    Polygon<T>,MultiPolygon<T>,
    GeometryCollection<T>,
    Rect<T>,Triangle<T>
    ]);
