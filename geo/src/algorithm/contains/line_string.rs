use super::{Contains, impl_contains_from_relate, impl_contains_geometry_for};
use crate::Orientation;
use crate::algorithm::kernels::Kernel;
use crate::geometry::*;
use crate::intersects::value_in_range;
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

        // orient the other segment
        let mut line = if line.start.x > line.end.x {
            Line::new(line.end, line.start)
        } else {
            *line
        };

        // improve performance by filtering out irrelevant segments
        let candidates_iter = self
            .lines()
            .filter(|segment| x_overlap(&line, segment))
            .filter(|segment| is_collinear(&line, segment));

        let mut changed = true;

        // use y value instead if x values are identical
        if line.start.x != line.end.x {
            let candidates: Vec<_> = candidates_iter
                // flip such that start < end for all segments
                .map(|segment| {
                    if segment.start.x < segment.end.x {
                        segment
                    } else {
                        Line::new(segment.end, segment.start)
                    }
                })
                .collect();
            while changed {
                changed = false;
                for candidate in candidates.iter() {
                    if candidate.start.x > line.start.x {
                        // cannot trim
                        continue;
                    }
                    if line.end.x <= candidate.end.x {
                        // is covered, can terminate early
                        return true;
                    }

                    if candidate.end.x > line.start.x {
                        // trimmed
                        changed = true;
                        line = Line::new(candidate.end, line.end);
                    }

                    // else candidate ends before line, no trim needed
                }
            }

            false
        } else {
            let candidates: Vec<_> = candidates_iter
                // flip such that start < end for all segments
                .map(|segment| {
                    // use greater_than to handle the case where x values are equal (straight up or down)
                    if segment.start.y < segment.end.y {
                        segment
                    } else {
                        Line::new(segment.end, segment.start)
                    }
                })
                .filter(|segment| y_overlap(&line, segment))
                .collect();
            while changed {
                changed = false;
                for candidate in candidates.iter() {
                    if candidate.start.y > line.start.y {
                        // cannot trim
                        continue;
                    }
                    if line.end.y <= candidate.end.y {
                        // is covered, can terminate early
                        return true;
                    }

                    if candidate.end.y > line.start.y {
                        // trimmed
                        changed = true;
                        line = Line::new(candidate.end, line.end);
                    }

                    // else candidate ends before line, no trim needed
                }
            }

            false
        }
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

impl<T> Contains<Coord<T>> for MultiLineString<T>
where
    T: CoordNum,
    LineString<T>: Contains<Coord<T>>,
{
    fn contains(&self, coord: &Coord<T>) -> bool {
        self.iter().any(|ls| ls.contains(coord))
    }
}

impl<T> Contains<Point<T>> for MultiLineString<T>
where
    T: CoordNum,
    LineString<T>: Contains<Point<T>>,
{
    fn contains(&self, rhs: &Point<T>) -> bool {
        self.iter().any(|ls| ls.contains(rhs))
    }
}

#[inline]
fn is_collinear<T>(l1: &Line<T>, l2: &Line<T>) -> bool
where
    T: GeoNum,
{
    T::Ker::orient2d(l1.start, l1.end, l2.start) == Orientation::Collinear
        && T::Ker::orient2d(l1.start, l1.end, l2.end) == Orientation::Collinear
}

#[inline]
fn x_overlap<T: GeoNum>(l1: &Line<T>, l2: &Line<T>) -> bool {
    let (p1, p2) = if l1.start.x < l1.end.x {
        (l1.start.x, l1.end.x)
    } else {
        (l1.end.x, l1.start.x)
    };
    let (q1, q2) = if l2.start.x < l2.end.x {
        (l2.start.x, l2.end.x)
    } else {
        (l2.end.x, l2.start.x)
    };
    value_in_range(p1, q1, q2) || value_in_range(q1, p1, p2)
}

#[inline]
fn y_overlap<T: GeoNum>(l1: &Line<T>, l2: &Line<T>) -> bool {
    // save 2 if statements compared to 4 calls of `value_in_between`

    let (p1, p2) = if l1.start.y < l1.end.y {
        (l1.start.y, l1.end.y)
    } else {
        (l1.end.y, l1.start.y)
    };
    let (q1, q2) = if l2.start.y < l2.end.y {
        (l2.start.y, l2.end.y)
    } else {
        (l2.end.y, l2.start.y)
    };
    value_in_range(p1, q1, q2) || value_in_range(q1, p1, p2)
}

#[cfg(test)]
mod test {
    use crate::{Contains, Relate};
    use crate::{Convert, wkt};
    use crate::{Line, LineString, Validation};

    #[test]
    fn triangles() {
        let ln: Line<f64> = wkt! {LINE(0 0, 10 0)}.convert();
        let ls: LineString<f64> = wkt! {LINESTRING(0 0, 1 1, 2 0, 4 0, 5 1, 6 0, 8 0, 9 1,10 0, 8 0, 7 -1, 6 0, 4 0, 3 -1, 2 0 , 0 0 )}.convert();

        // ln and ls are valid
        assert!(ln.is_valid());
        assert!(ls.is_valid());

        assert_eq!(
            ls.relate(&ln).is_contains(), // true
            ls.contains(&ln)              // true
        );
    }

    #[test]
    fn test_start_end() {
        let ls: LineString<f64> = wkt! {LINESTRING(0 0,0 1, 1 1)}.convert();
        let ln_start: Line<f64> = wkt! {LINE(0 0, 0 0)}.convert();
        let ln_end: Line<f64> = wkt! {LINE(1 1, 1 1)}.convert();

        assert!(!ls.contains(&ln_start));
        assert!(!ls.contains(&ln_end));
    }
}
