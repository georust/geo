use super::*;

/// Converts the coordinates of a geometry to degrees
///
/// # Example
/// ```
/// use geo_types::Point;
///
/// let p = Point::new(1.234, 2.345);
/// let (x, y): (f32, f32) = p.to_degrees().x_y();
/// assert_eq!(x.round(), 71.0);
/// assert_eq!(y.round(), 134.0);
pub trait ToDegrees {
    fn to_degrees(self) -> Self;
}

impl ToDegrees for Coord {
    fn to_degrees(self) -> Self {
        Self {
            x: self.x.to_degrees(),
            y: self.y.to_degrees(),
        }
    }
}

impl ToDegrees for Point {
    fn to_degrees(self) -> Self {
        let (x, y) = self.x_y();
        let x = x.to_degrees();
        let y = y.to_degrees();
        Self::new(x, y)
    }
}

impl ToDegrees for MultiPoint {
    fn to_degrees(self) -> Self {
        self.into_iter().map(|point| point.to_degrees()).collect()
    }
}

impl ToDegrees for Line {
    fn to_degrees(self) -> Self {
        Self {
            start: self.start.to_degrees(),
            end: self.end.to_degrees(),
        }
    }
}

impl ToDegrees for LineString {
    fn to_degrees(mut self) -> Self {
        self.0
            .iter_mut()
            .for_each(|coord| *coord = coord.to_degrees());
        self
    }
}

impl ToDegrees for MultiLineString {
    fn to_degrees(self) -> Self {
        self.into_iter()
            .map(|line_string| line_string.to_degrees())
            .collect()
    }
}

impl ToDegrees for Polygon {
    fn to_degrees(self) -> Self {
        let (exterior, interiors) = self.into_inner();
        let exterior = exterior.to_degrees();
        let interiors = interiors
            .into_iter()
            .map(|interior| interior.to_degrees())
            .collect();
        Self::new(exterior, interiors)
    }
}

impl ToDegrees for MultiPolygon {
    fn to_degrees(self) -> Self {
        self.into_iter()
            .map(|polygon| polygon.to_degrees())
            .collect()
    }
}

impl ToDegrees for Rect {
    fn to_degrees(mut self) -> Self {
        self.set_min(self.min().to_degrees());
        self.set_max(self.max().to_degrees());
        self
    }
}

impl ToDegrees for Triangle {
    fn to_degrees(mut self) -> Self {
        self.0 = self.0.to_degrees();
        self.1 = self.1.to_degrees();
        self.2 = self.2.to_degrees();
        self
    }
}

impl ToDegrees for Geometry {
    fn to_degrees(self) -> Self {
        match self {
            Geometry::Point(geometry) => Geometry::Point(geometry.to_degrees()),
            Geometry::Line(geometry) => Geometry::Line(geometry.to_degrees()),
            Geometry::LineString(geometry) => Geometry::LineString(geometry.to_degrees()),
            Geometry::Polygon(geometry) => Geometry::Polygon(geometry.to_degrees()),
            Geometry::MultiPoint(geometry) => Geometry::MultiPoint(geometry.to_degrees()),
            Geometry::MultiLineString(geometry) => Geometry::MultiLineString(geometry.to_degrees()),
            Geometry::MultiPolygon(geometry) => Geometry::MultiPolygon(geometry.to_degrees()),
            Geometry::Rect(geometry) => Geometry::Rect(geometry.to_degrees()),
            Geometry::Triangle(geometry) => Geometry::Triangle(geometry.to_degrees()),
            Geometry::GeometryCollection(geometry) => {
                Geometry::GeometryCollection(geometry.to_degrees())
            }
        }
    }
}

impl ToDegrees for GeometryCollection {
    fn to_degrees(self) -> Self {
        self.into_iter()
            .map(|geometry| geometry.to_degrees())
            .collect()
    }
}
