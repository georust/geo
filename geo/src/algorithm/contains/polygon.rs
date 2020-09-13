use super::Contains;
use crate::*;
use crate::kernels::*;
use crate::intersects::Intersects;

// ┌─────────────────────────────┐
// │ Implementations for Polygon │
// └─────────────────────────────┘

impl<T> Contains<Coordinate<T>> for Polygon<T>
where
    T: HasKernel,
{
    fn contains(&self, coord: &Coordinate<T>) -> bool {
        match utils::coord_pos_relative_to_ring(*coord, &self.exterior()) {
            utils::CoordPos::OnBoundary | utils::CoordPos::Outside => false,
            _ => self.interiors().iter().all(|ls| {
                utils::coord_pos_relative_to_ring(*coord, ls) == utils::CoordPos::Outside
            }),
        }
    }
}

impl<T> Contains<Point<T>> for Polygon<T>
where
    T: HasKernel,
{
    fn contains(&self, p: &Point<T>) -> bool {
        self.contains(&p.0)
    }
}

impl<T> Contains<Line<T>> for Polygon<T>
where
    T: HasKernel,
{
    fn contains(&self, line: &Line<T>) -> bool {
        // both endpoints are contained in the polygon and the line
        // does NOT intersect the exterior or any of the interior boundaries
        self.contains(&line.start_point())
            && self.contains(&line.end_point())
            && !self.exterior().intersects(line)
            && !self.interiors().iter().any(|inner| inner.intersects(line))
    }
}

impl<T> Contains<Polygon<T>> for Polygon<T>
where
    T: HasKernel,
{
    fn contains(&self, poly: &Polygon<T>) -> bool {
        // decompose poly's exterior ring into Lines, and check each for containment
        poly.exterior().lines().all(|line| self.contains(&line))
    }
}

impl<T> Contains<LineString<T>> for Polygon<T>
where
    T: HasKernel,
{
    fn contains(&self, linestring: &LineString<T>) -> bool {
        // All LineString points must be inside the Polygon
        if linestring.points_iter().all(|point| self.contains(&point)) {
            // The Polygon interior is allowed to intersect with the LineString
            // but the Polygon's rings are not
            !self
                .interiors()
                .iter()
                .any(|ring| ring.intersects(linestring))
        } else {
            false
        }
    }
}

// ┌──────────────────────────────────┐
// │ Implementations for MultiPolygon │
// └──────────────────────────────────┘

impl<T> Contains<Coordinate<T>> for MultiPolygon<T>
where
    T: HasKernel,
{
    fn contains(&self, coord: &Coordinate<T>) -> bool {
        self.0.iter().any(|poly| poly.contains(coord))
    }
}

impl<T> Contains<Point<T>> for MultiPolygon<T>
where
    T: HasKernel,
{
    fn contains(&self, p: &Point<T>) -> bool {
        self.contains(&p.0)
    }
}
