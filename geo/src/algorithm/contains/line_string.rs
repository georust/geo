use super::Contains;
use crate::*;
use crate::kernels::*;


// ┌────────────────────────────────┐
// │ Implementations for LineString │
// └────────────────────────────────┘

impl<T> Contains<Coordinate<T>> for LineString<T>
where
    T: HasKernel,
{
    fn contains(&self, coord: &Coordinate<T>) -> bool {
        self.lines().any(|line| line.contains(coord))
    }
}

impl<T> Contains<Point<T>> for LineString<T>
where
    T: HasKernel,
{
    fn contains(&self, p: &Point<T>) -> bool {
        self.lines().any(|line| line.contains(p))
    }
}

impl<T> Contains<Line<T>> for LineString<T>
where
    T: HasKernel,
{
    fn contains(&self, line: &Line<T>) -> bool {
        let (p0, p1) = line.points();
        let mut look_for: Option<Point<T>> = None;
        for segment in self.lines() {
            if look_for.is_none() {
                // If segment contains an endpoint of line, we mark the other endpoint as the
                // one we are looking for.
                if segment.contains(&p0) {
                    look_for = Some(p1);
                } else if segment.contains(&p1) {
                    look_for = Some(p0);
                }
            }
            if let Some(p) = look_for {
                // If we are looking for an endpoint, we need to either find it, or show that we
                // should continue to look for it
                if segment.contains(&p) {
                    // If the segment contains the endpoint we are looking for we are done
                    return true;
                } else if !line.contains(&segment.end_point()) {
                    // If not, and the end of the segment is not on the line, we should stop
                    // looking
                    look_for = None
                }
            }
        }
        false
    }
}

// ┌─────────────────────────────────────┐
// │ Implementations for MultiLineString │
// └─────────────────────────────────────┘
impl<T> Contains<Coordinate<T>> for MultiLineString<T>
where
    T: HasKernel,
{
    fn contains(&self, coord: &Coordinate<T>) -> bool {
        self.0.iter().any(|line_string| line_string.contains(coord))
    }
}

impl<T> Contains<Point<T>> for MultiLineString<T>
where
    T: HasKernel,
{
    fn contains(&self, point: &Point<T>) -> bool {
        self.contains(&point.0)
    }
}
