use super::{ impl_contains_geometry_for, Contains};
use crate::algorithm::{CoordsIter, HasDimensions};
use crate::geometry::*;
use crate::{CoordNum, GeoFloat};

// ┌────────────────────────────────┐
// │ Implementations for Coord      │
// └────────────────────────────────┘

impl<T> Contains<Coord<T>> for Coord<T>
where
    T: CoordNum,
{
    fn contains(&self, coord: &Coord<T>) -> bool {
        self == coord
    }
}

impl<T> Contains<Point<T>> for Coord<T>
where
    T: CoordNum,
{
    fn contains(&self, p: &Point<T>) -> bool {
        self.contains(&p.0)
    }
}

impl<T> Contains<Line<T>> for Coord<T>
where
    T: CoordNum,
{
    fn contains(&self, line: &Line<T>) -> bool {
        if line.start == line.end {
            // degenerate line is a point
            line.start == *self
        } else {
            false
        }
    }
}

impl<T> Contains<LineString<T>> for Coord<T>
where
    T: CoordNum,
{
    fn contains(&self, line_string: &LineString<T>) -> bool {
        if line_string.is_empty() {
            return false;
        }
        // only a degenerate LineString could be within a point
        line_string.coords().all(|c| c == self)
    }
}

impl<T> Contains<Polygon<T>> for Coord<T>
where
    T: CoordNum,
{
    fn contains(&self, polygon: &Polygon<T>) -> bool {
        if polygon.is_empty() {
            return false;
        }
        // only a degenerate Polygon could be within a point
        polygon.coords_iter().all(|coord| coord == *self)
    }
}

impl<T> Contains<MultiPoint<T>> for Coord<T>
where
    T: CoordNum,
{
    fn contains(&self, multi_point: &MultiPoint<T>) -> bool {
        if multi_point.is_empty() {
            return false;
        }
        multi_point.iter().all(|point| self.contains(point))
    }
}

impl<T> Contains<MultiLineString<T>> for Coord<T>
where
    T: CoordNum,
{
    fn contains(&self, multi_line_string: &MultiLineString<T>) -> bool {
        if multi_line_string.is_empty() {
            return false;
        }
        // only a degenerate MultiLineString could be within a point
        multi_line_string
            .iter()
            .all(|line_string| self.contains(line_string))
    }
}

impl<T> Contains<MultiPolygon<T>> for Coord<T>
where
    T: CoordNum,
{
    fn contains(&self, multi_polygon: &MultiPolygon<T>) -> bool {
        if multi_polygon.is_empty() {
            return false;
        }
        // only a degenerate MultiPolygon could be within a point
        multi_polygon.iter().all(|polygon| self.contains(polygon))
    }
}

impl<T> Contains<GeometryCollection<T>> for Coord<T>
where
    T: GeoFloat,
{
    fn contains(&self, geometry_collection: &GeometryCollection<T>) -> bool {
        if geometry_collection.is_empty() {
            return false;
        }
        geometry_collection
            .iter()
            .all(|geometry| self.contains(geometry))
    }
}

impl<T> Contains<Rect<T>> for Coord<T>
where
    T: CoordNum,
{
    fn contains(&self, rect: &Rect<T>) -> bool {
        // only a degenerate Rect could be within a point
        rect.min() == rect.max() && rect.min() == *self
    }
}

impl<T> Contains<Triangle<T>> for Coord<T>
where
    T: CoordNum,
{
    fn contains(&self, triangle: &Triangle<T>) -> bool {
        // only a degenerate Triangle could be within a point
        triangle.0 == triangle.1 && triangle.0 == triangle.2 && triangle.0 == *self
    }
}

impl_contains_geometry_for!(Coord<T>);
