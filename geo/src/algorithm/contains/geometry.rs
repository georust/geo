use super::Contains;
use crate::kernels::*;
use crate::*;

// ┌──────────────────────────────┐
// │ Implementations for Geometry │
// └──────────────────────────────┘

impl<T> Contains<Coordinate<T>> for Geometry<T>
where
    T: HasKernel,
{
    fn contains(&self, coord: &Coordinate<T>) -> bool {
        match self {
            Geometry::Point(g) => g.contains(coord),
            Geometry::Line(g) => g.contains(coord),
            Geometry::LineString(g) => g.contains(coord),
            Geometry::Polygon(g) => g.contains(coord),
            Geometry::MultiPoint(g) => g.contains(coord),
            Geometry::MultiLineString(g) => g.contains(coord),
            Geometry::MultiPolygon(g) => g.contains(coord),
            Geometry::GeometryCollection(g) => g.contains(coord),
            Geometry::Rect(g) => g.contains(coord),
            Geometry::Triangle(g) => g.contains(coord),
        }
    }
}

impl<T> Contains<Point<T>> for Geometry<T>
where
    T: HasKernel,
{
    fn contains(&self, point: &Point<T>) -> bool {
        self.contains(&point.0)
    }
}

// ┌────────────────────────────────────────┐
// │ Implementations for GeometryCollection │
// └────────────────────────────────────────┘

impl<T> Contains<Coordinate<T>> for GeometryCollection<T>
where
    T: HasKernel,
{
    fn contains(&self, coord: &Coordinate<T>) -> bool {
        self.iter().any(|geometry| geometry.contains(coord))
    }
}

impl<T> Contains<Point<T>> for GeometryCollection<T>
where
    T: HasKernel,
{
    fn contains(&self, point: &Point<T>) -> bool {
        self.contains(&point.0)
    }
}
