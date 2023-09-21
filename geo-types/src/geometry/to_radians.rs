use super::*;

/// Converts the coordinates of a geometry to radians
///
/// # Example
/// ```
/// use geo_types::Point;
///
/// let p = Point::new(180.0, 341.5);
/// let (x, y): (f32, f32) = p.to_radians().x_y();
/// assert_eq!(x.round(), 3.0);
/// assert_eq!(y.round(), 6.0);
pub trait ToRadians {
    fn to_radians(self) -> Self;
}

impl ToRadians for Coord {
    fn to_radians(self) -> Self {
        Self {
            x: self.x.to_radians(),
            y: self.y.to_radians(),
        }
    }
}

impl ToRadians for Point {
    fn to_radians(self) -> Self {
        let (x, y) = self.x_y();
        let x = x.to_radians();
        let y = y.to_radians();
        Self::new(x, y)
    }
}

impl ToRadians for MultiPoint {
    fn to_radians(self) -> Self {
        self.into_iter().map(|point| point.to_radians()).collect()
    }
}

impl ToRadians for Line {
    fn to_radians(self) -> Self {
        Self {
            start: self.start.to_radians(),
            end: self.end.to_radians(),
        }
    }
}

impl ToRadians for LineString {
    fn to_radians(mut self) -> Self {
        self.0
            .iter_mut()
            .for_each(|coord| *coord = coord.to_radians());
        self
    }
}

impl ToRadians for MultiLineString {
    fn to_radians(self) -> Self {
        self.into_iter()
            .map(|line_string| line_string.to_radians())
            .collect()
    }
}

impl ToRadians for Polygon {
    fn to_radians(self) -> Self {
        let (exterior, interiors) = self.into_inner();
        let exterior = exterior.to_radians();
        let interiors = interiors
            .into_iter()
            .map(|interior| interior.to_radians())
            .collect();
        Self::new(exterior, interiors)
    }
}

impl ToRadians for MultiPolygon {
    fn to_radians(self) -> Self {
        self.into_iter()
            .map(|polygon| polygon.to_radians())
            .collect()
    }
}

impl ToRadians for Rect {
    fn to_radians(mut self) -> Self {
        self.set_min(self.min().to_radians());
        self.set_max(self.max().to_radians());
        self
    }
}

impl ToRadians for Triangle {
    fn to_radians(mut self) -> Self {
        self.0 = self.0.to_radians();
        self.1 = self.1.to_radians();
        self.2 = self.2.to_radians();
        self
    }
}

impl ToRadians for Geometry {
    fn to_radians(self) -> Self {
        match self {
            Geometry::Point(geometry) => Geometry::Point(geometry.to_radians()),
            Geometry::Line(geometry) => Geometry::Line(geometry.to_radians()),
            Geometry::LineString(geometry) => Geometry::LineString(geometry.to_radians()),
            Geometry::Polygon(geometry) => Geometry::Polygon(geometry.to_radians()),
            Geometry::MultiPoint(geometry) => Geometry::MultiPoint(geometry.to_radians()),
            Geometry::MultiLineString(geometry) => Geometry::MultiLineString(geometry.to_radians()),
            Geometry::MultiPolygon(geometry) => Geometry::MultiPolygon(geometry.to_radians()),
            Geometry::Rect(geometry) => Geometry::Rect(geometry.to_radians()),
            Geometry::Triangle(geometry) => Geometry::Triangle(geometry.to_radians()),
            Geometry::GeometryCollection(geometry) => {
                Geometry::GeometryCollection(geometry.to_radians())
            }
        }
    }
}

impl ToRadians for GeometryCollection {
    fn to_radians(self) -> Self {
        self.into_iter()
            .map(|geometry| geometry.to_radians())
            .collect()
    }
}
