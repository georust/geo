use super::{impl_contains_from_relate, impl_contains_geometry_for, Contains};
use crate::algorithm::Intersects;
use crate::geometry::*;
use crate::{CoordNum, GeoFloat, GeoNum, HasDimensions};

// ┌────────────────────────────────┐
// │ Implementations for LineString │
// └────────────────────────────────┘

impl<T> Contains<Coord<T>> for LineString<T>
where
    T: GeoNum,
{
    fn contains(&self, coord: &Coord<T>) -> bool {
        if self.0.is_empty() {
            return false;
        }

        if coord == &self.0[0] || coord == self.0.last().unwrap() {
            return self.is_closed();
        }

        self.lines()
            .enumerate()
            .any(|(i, line)| line.contains(coord) || (i > 0 && coord == &line.start))
    }
}

impl<T> Contains<Point<T>> for LineString<T>
where
    T: GeoNum,
{
    fn contains(&self, p: &Point<T>) -> bool {
        self.contains(&p.0)
    }
}

impl<T> Contains<Line<T>> for LineString<T>
where
    T: GeoNum,
{
    fn contains(&self, line: &Line<T>) -> bool {
        if line.start == line.end {
            return self.contains(&line.start);
        }

        // We copy the line as we may truncate the line as
        // we find partial matches.
        let mut line = *line;
        let mut first_cut = None;

        let lines_iter = self.lines();
        let num_lines = lines_iter.len();

        // We need to repeat the logic twice to handle cases
        // where the linestring starts at the middle of the line.
        for (i, segment) in self.lines().chain(lines_iter).enumerate() {
            if i >= num_lines {
                // The first loop was done. If we never cut
                // the line, or at the cut segment again, we
                // can exit now.
                if let Some(upto_i) = first_cut {
                    if i >= num_lines + upto_i {
                        break;
                    }
                } else {
                    break;
                }
            }
            // Look for a segment that intersects at least
            // one of the end points.
            let other = if segment.intersects(&line.start) {
                line.end
            } else if segment.intersects(&line.end) {
                line.start
            } else {
                continue;
            };

            // If the other end point also intersects this
            // segment, then we are done.
            let new_inside = if segment.intersects(&other) {
                return true;
            }
            // otoh, if the line contains one of the ends of
            // the segments, then we truncate the line to
            // the part outside.
            else if line.contains(&segment.start) {
                segment.start
            } else if line.contains(&segment.end) {
                segment.end
            } else {
                continue;
            };

            first_cut = first_cut.or(Some(i));
            if other == line.start {
                line.end = new_inside;
            } else {
                line.start = new_inside;
            }
        }

        false
    }
}

impl<T> Contains<LineString<T>> for LineString<T>
where
    T: GeoNum,
{
    fn contains(&self, rhs: &LineString<T>) -> bool {
        if self.is_empty() || rhs.is_empty() {
            return false;
        }
        rhs.lines().all(|l| self.contains(&l))
    }
}

impl_contains_from_relate!(LineString<T>, [Polygon<T>, MultiPoint<T>, MultiLineString<T>, MultiPolygon<T>, GeometryCollection<T>, Rect<T>, Triangle<T>]);
impl_contains_geometry_for!(LineString<T>);

// ┌─────────────────────────────────────┐
// │ Implementations for MultiLineString │
// └─────────────────────────────────────┘

impl_contains_from_relate!(MultiLineString<T>, [Line<T>, LineString<T>, Polygon<T>, MultiPoint<T>, MultiLineString<T>, MultiPolygon<T>, GeometryCollection<T>, Rect<T>, Triangle<T>]);
impl_contains_geometry_for!(MultiLineString<T>);

impl<T> Contains<Point<T>> for MultiLineString<T>
where
    T: CoordNum,
    LineString<T>: Contains<Point<T>>,
{
    fn contains(&self, rhs: &Point<T>) -> bool {
        self.iter().any(|ls| ls.contains(rhs))
    }
}
