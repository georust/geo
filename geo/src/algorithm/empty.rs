use crate::{
    CoordinateType, Geometry, GeometryCollection, Line, LineString, MultiLineString, MultiPoint,
    MultiPolygon, Point, Polygon, Rect, Triangle,
};

/// Some geometries, like a `MultiPoint`, can have zero coordinates - we call these `empty`.
///
/// Types like `Point` and `Rect`, which have at least one coordinate by construction, can
/// never be considered empty.
///
/// ```
/// use geo_types::{Point, Coordinate, LineString};
/// use geo::algorithm::empty::Empty;
///
/// let line_string = LineString(vec![
///     Coordinate { x: 0., y: 0. },
///     Coordinate { x: 10., y: 0. },
/// ]);
/// assert!(!line_string.is_empty());
///
/// let empty_line_string: LineString<f64> = LineString(vec![]);
/// assert!(empty_line_string.is_empty());
///
///
/// let point = Point::new(0.0, 0.0);
/// assert!(!point.is_empty());
/// ```
pub trait Empty {
    fn is_empty(&self) -> bool;
}

impl<C: CoordinateType> Empty for Geometry<C> {
    fn is_empty(&self) -> bool {
        match self {
            Geometry::Point(g) => g.is_empty(),
            Geometry::Line(g) => g.is_empty(),
            Geometry::LineString(g) => g.is_empty(),
            Geometry::Polygon(g) => g.is_empty(),
            Geometry::MultiPoint(g) => g.is_empty(),
            Geometry::MultiLineString(g) => g.is_empty(),
            Geometry::MultiPolygon(g) => g.is_empty(),
            Geometry::GeometryCollection(g) => g.is_empty(),
            Geometry::Rect(g) => g.is_empty(),
            Geometry::Triangle(g) => g.is_empty(),
        }
    }
}

impl<C: CoordinateType> Empty for Point<C> {
    fn is_empty(&self) -> bool {
        false
    }
}

impl<C: CoordinateType> Empty for Line<C> {
    fn is_empty(&self) -> bool {
        false
    }
}

impl<C: CoordinateType> Empty for LineString<C> {
    fn is_empty(&self) -> bool {
        self.0.is_empty()
    }
}

impl<C: CoordinateType> Empty for Polygon<C> {
    fn is_empty(&self) -> bool {
        self.exterior().is_empty()
    }
}

impl<C: CoordinateType> Empty for MultiPoint<C> {
    fn is_empty(&self) -> bool {
        self.0.is_empty()
    }
}

impl<C: CoordinateType> Empty for MultiLineString<C> {
    fn is_empty(&self) -> bool {
        if self.0.is_empty() {
            true
        } else {
            self.0.iter().all(LineString::is_empty)
        }
    }
}

impl<C: CoordinateType> Empty for MultiPolygon<C> {
    fn is_empty(&self) -> bool {
        if self.0.is_empty() {
            true
        } else {
            self.0.iter().all(Polygon::is_empty)
        }
    }
}

impl<C: CoordinateType> Empty for GeometryCollection<C> {
    fn is_empty(&self) -> bool {
        if self.0.is_empty() {
            true
        } else {
            self.0.iter().all(Geometry::is_empty)
        }
    }
}

impl<C: CoordinateType> Empty for Rect<C> {
    fn is_empty(&self) -> bool {
        false
    }
}

impl<C: CoordinateType> Empty for Triangle<C> {
    fn is_empty(&self) -> bool {
        false
    }
}
