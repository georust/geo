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

        let is_vertical = line.start.x == line.end.x;

        // pre-order the line so that we can use the faster overlap check
        let line = if is_vertical {
            if line.start.y > line.end.y {
                Line::new(line.end, line.start)
            } else {
                *line
            }
        } else {
            if line.start.x > line.end.x {
                Line::new(line.end, line.start)
            } else {
                *line
            }
        };

        let candidates: Vec<(T, T)> = if is_vertical {
            self.lines()
                .filter(|segment| overlap::y_overlap(&line, segment))
                .filter(|segment| is_collinear(&line, segment))
                .map(|segment| {
                    if segment.start.y < segment.end.y {
                        (segment.start.y, segment.end.y)
                    } else {
                        (segment.end.y, segment.start.y)
                    }
                })
                .collect()
        } else {
            self.lines()
                .filter(|segment| overlap::x_overlap(&line, segment))
                .filter(|segment| is_collinear(&line, segment))
                .map(|segment| {
                    if segment.start.x < segment.end.x {
                        (segment.start.x, segment.end.x)
                    } else {
                        (segment.end.x, segment.start.x)
                    }
                })
                .collect()
        };

        let mut changed = true;

        // use y value instead if x values are identical
        let (mut line_start, mut line_end) = if is_vertical {
            (line.start.y, line.end.y)
        } else {
            (line.start.x, line.end.x)
        };

        // interval-based overlap checks
        while changed {
            changed = false;
            for (c_start, c_end) in candidates.iter() {
                // if no overlap, skip
                if *c_end <= line_start || line_end <= *c_start {
                }
                // if candidate covers line, return true
                else if *c_start <= line_start && line_end <= *c_end {
                    return true;
                } else if *c_start <= line_start {
                    // trim start
                    changed = true;
                    line_start = *c_end;
                } else if line_end <= *c_end {
                    // trim end
                    changed = true;
                    line_end = *c_start;
                }
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

/// Suppose we have 2 pairs (p1,p2) and (q1,q2) where p1 < p2 and q1 < q2
///
/// It is sufficient to show that each lower bound is smaller than the others' upper bound for the ranges to overlap  
mod overlap {
    use super::*;

    #[inline]
    /// Since l1 is ordered, we can execute overlap check in 3 comparisons.  
    /// We use exclusive bounds because we only want to keep segments which can trim the line
    pub(super) fn x_overlap<T: GeoNum>(ordered_l1: &Line<T>, l2: &Line<T>) -> bool {
        debug_assert!(ordered_l1.start.x <= ordered_l1.end.x);

        let (p1, p2) = (ordered_l1.start.x, ordered_l1.end.x);
        let (q1, q2) = if l2.start.x < l2.end.x {
            (l2.start.x, l2.end.x)
        } else {
            (l2.end.x, l2.start.x)
        };

        p1 < q2 && q1 < p2
    }

    #[inline]
    /// Since l1 is ordered, we can execute overlap check in 3 comparisons.  
    /// We use exclusive bounds because we only want to keep segments which can trim the line
    pub(super) fn y_overlap<T: GeoNum>(ordered_l1: &Line<T>, l2: &Line<T>) -> bool {
        debug_assert!(ordered_l1.start.y <= ordered_l1.end.y);

        let (p1, p2) = (ordered_l1.start.y, ordered_l1.end.y);
        let (q1, q2) = if l2.start.y < l2.end.y {
            (l2.start.y, l2.end.y)
        } else {
            (l2.end.y, l2.start.y)
        };

        p1 < q2 && q1 < p2
    }
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

    #[test]
    fn test_vertical() {
        let ls1: LineString<f64> = wkt! {LINESTRING(0 0,0 5,0 10)}.convert();
        let ls2: LineString<f64> = wkt! {LINESTRING(0 10,0 5, 0 0)}.convert();

        let ln: Line<f64> = wkt! {LINE(0 0, 0 9)}.convert();

        assert!(ls1.contains(&ln));
        assert!(ls2.contains(&ln));
    }
}
