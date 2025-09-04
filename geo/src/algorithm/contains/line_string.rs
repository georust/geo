use super::{Contains, impl_contains_from_relate, impl_contains_geometry_for};
use crate::Orientation;
use crate::algorithm::kernels::Kernel;
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

        // orient the other segment
        let mut line = if line.start.x > line.end.x {
            Line::new(line.end, line.start)
        } else {
            *line
        };

        // improve performance by filtering out irrelevant segments
        let candidates: Vec<_> = self
            .lines()
            // keep only collinear segments
            .filter(|segment| is_collinear(&line, segment))
            // flip such that start < end for all segments
            .map(|segment| {
                if greater_than(&segment.end, &segment.start) {
                    segment
                } else {
                    Line::new(segment.end, segment.start)
                }
            })
            // filter out non-intersecting segments
            // faster method using knowledge that segments are collinear to line
            .filter(|segment| {
                greater_than(&line.end, &segment.start) && greater_than(&segment.end, &line.start)
            })
            .collect();

        let mut changed = true;
        while changed {
            changed = false;
            for candidate in candidates.iter() {
                if greater_than(&candidate.start, &line.start) {
                    // cannot trim
                    continue;
                }
                if !greater_than(&line.end, &candidate.end) {
                    // is covered, can terminate early
                    return true;
                }

                if greater_than(&candidate.end, &line.start) {
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

// faster than lex_cmp since we kmow GeoNum has total ordering
#[inline]
fn greater_than<T: GeoNum>(p: &Coord<T>, q: &Coord<T>) -> bool {
    p.x > q.x || (p.x == q.x && p.y > q.y)
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
    fn test_exact_identical() {
        let ln: Line<f64> = wkt! {LINE(0 0, 1 1)}.convert();
        let ls1: LineString<f64> = ln.clone().into();
        let ls2: LineString<f64> = ln.clone().into();

        // matches current Relate.is_contains() behavior
        assert_eq!(
            ln.relate(&ls1).is_contains(), // true
            ln.contains(&ls1)              // true
        );

        assert_eq!(
            ls1.relate(&ln).is_contains(), // true
            ls1.contains(&ln)              // true
        );

        assert_eq!(
            ls1.relate(&ls2).is_contains(), // true
            ls1.contains(&ls2)              // true
        );

        // but this isn't really correct
        assert!(ln.relate(&ls1).is_contains());
        assert!(ln.contains(&ls1));

        assert!(ls1.relate(&ln).is_contains());
        assert!(ls1.contains(&ln));

        assert!(ls1.relate(&ls2).is_contains());
        assert!(ls1.contains(&ls2));
    }

    #[test]
    fn test_vertical() {
        let ln: Line<f64> = wkt! {LINE(0 1, 0 2)}.convert();
        let ls: LineString<f64> = wkt! {LINESTRING(0 0, 0 4)}.convert();

        assert!(ls.contains(&ln));
    }

    #[test]
    fn test_horizontal() {
        let ln: Line<f64> = wkt! {LINE(1 0, 2 0)}.convert();
        let ls: LineString<f64> = wkt! {LINESTRING(0 0, 4 0)}.convert();

        assert!(ls.contains(&ln));
    }
}
