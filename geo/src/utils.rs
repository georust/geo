//! Internal utility functions, types, and data structures.

use crate::intersects::Intersects;
use crate::kernels::*;
use geo_types::{Coordinate, CoordinateType, Line};

/// Partition a mutable slice in-place so that it contains all elements for
/// which `predicate(e)` is `true`, followed by all elements for which
/// `predicate(e)` is `false`. Returns sub-slices to all predicated and
/// non-predicated elements, respectively.
///
/// https://github.com/llogiq/partition/blob/master/src/lib.rs
pub fn partition_slice<T, P>(data: &mut [T], predicate: P) -> (&mut [T], &mut [T])
where
    P: Fn(&T) -> bool,
{
    let len = data.len();
    if len == 0 {
        return (&mut [], &mut []);
    }
    let (mut l, mut r) = (0, len - 1);
    loop {
        while l < len && predicate(&data[l]) {
            l += 1;
        }
        while r > 0 && !predicate(&data[r]) {
            r -= 1;
        }
        if l >= r {
            return data.split_at_mut(l);
        }
        data.swap(l, r);
    }
}

/// Enumeration that allows for two distinct iterator types that yield the same type.
pub enum EitherIter<T, I1, I2>
where
    I1: Iterator<Item = T>,
    I2: Iterator<Item = T>,
{
    A(I1),
    B(I2),
}

impl<T, I1, I2> Iterator for EitherIter<T, I1, I2>
where
    I1: Iterator<Item = T>,
    I2: Iterator<Item = T>,
{
    type Item = T;

    fn next(&mut self) -> Option<Self::Item> {
        match self {
            EitherIter::A(iter) => iter.next(),
            EitherIter::B(iter) => iter.next(),
        }
    }
}

// The Rust standard library has `max` for `Ord`, but not for `PartialOrd`
pub fn partial_max<T: PartialOrd>(a: T, b: T) -> T {
    if a > b {
        a
    } else {
        b
    }
}

// The Rust standard library has `min` for `Ord`, but not for `PartialOrd`
pub fn partial_min<T: PartialOrd>(a: T, b: T) -> T {
    if a < b {
        a
    } else {
        b
    }
}

/// The position of a `Coordinate` relative to a `LineString`
#[derive(PartialEq, Clone, Debug)]
pub enum CoordPos {
    OnBoundary,
    Inside,
    Outside,
}

/// Calculate the position of a `Coordinate` relative to a
/// closed `LineString`.
pub fn coord_pos_relative_to_ring<T>(
    coord: crate::Coordinate<T>,
    linestring: &crate::LineString<T>,
) -> CoordPos
where
    T: HasKernel,
{
    // Use the ray-tracing algorithm: count #times a
    // horizontal ray from point (to positive infinity).
    //
    // See: https://en.wikipedia.org/wiki/Point_in_polygon

    debug_assert!(linestring.is_closed());

    // LineString without points
    if linestring.0.is_empty() {
        return CoordPos::Outside;
    }
    if linestring.0.len() == 1 {
        // If LineString has one point, it will not generate
        // any lines.  So, we handle this edge case separately.
        return if coord == linestring.0[0] {
            CoordPos::OnBoundary
        } else {
            CoordPos::Outside
        };
    }

    let mut crossings = 0;
    for line in linestring.lines() {
        // Check if coord lies on the line
        if line.intersects(&coord) {
            return CoordPos::OnBoundary;
        }

        // Ignore if the line is strictly to the left of the coord.
        let max_x = if line.start.x < line.end.x {
            line.end.x
        } else {
            line.start.x
        };
        if max_x < coord.x {
            continue;
        }

        // Ignore if line is horizontal. This includes an
        // edge case where the ray would intersect a
        // horizontal segment of the ring infinitely many
        // times, and is irrelevant for the calculation.
        if line.start.y == line.end.y {
            continue;
        }

        // Ignore if the intersection of the line is
        // possibly at the beginning/end of the line, and
        // the line lies below the ray. This is to
        // prevent a double counting when the ray passes
        // through a vertex of the polygon.
        if (line.start.y == coord.y && line.end.y < coord.y)
            || (line.end.y == coord.y && line.start.y < coord.y)
        {
            continue;
        }

        // Otherwise, check if ray intersects the line
        // segment. Enough to consider ray upto the max_x
        // coordinate of the current segment.
        let ray = Line::new(
            coord,
            Coordinate {
                x: max_x,
                y: coord.y,
            },
        );
        if ray.intersects(&line) {
            crossings += 1;
        }
    }
    if crossings % 2 == 1 {
        CoordPos::Inside
    } else {
        CoordPos::Outside
    }
}

use std::cmp::Ordering;

/// Compare two coordinates lexicographically: first by the
/// x coordinate, and break ties with the y coordinate.
/// Expects none of coordinates to be uncomparable (eg. nan)
#[inline]
pub fn lex_cmp<T: CoordinateType>(p: &Coordinate<T>, q: &Coordinate<T>) -> Ordering {
    p.x.partial_cmp(&q.x)
        .unwrap()
        .then(p.y.partial_cmp(&q.y).unwrap())
}

/// Compute index of the least point in slice. Comparison is
/// done using [`lex_cmp`].
///
/// Should only be called on a non-empty slice with no `nan`
/// coordinates.
pub fn least_index<T: CoordinateType>(pts: &[Coordinate<T>]) -> usize {
    pts.iter()
        .enumerate()
        .min_by(|(_, p), (_, q)| lex_cmp(p, q))
        .unwrap()
        .0
}

/// Compute index of the lexicographically least _and_ the
/// greatest coordinate in one pass.
///
/// Should only be called on a non-empty slice with no `nan`
/// coordinates.
pub fn least_and_greatest_index<T: CoordinateType>(pts: &[Coordinate<T>]) -> (usize, usize) {
    assert_ne!(pts.len(), 0);
    let (min, max) = pts
        .iter()
        .enumerate()
        .fold((None, None), |(min, max), (idx, p)| {
            (
                if let Some((midx, min)) = min {
                    if lex_cmp(p, min) == Ordering::Less {
                        Some((idx, p))
                    } else {
                        Some((midx, min))
                    }
                } else {
                    Some((idx, p))
                },
                if let Some((midx, max)) = max {
                    if lex_cmp(p, max) == Ordering::Greater {
                        Some((idx, p))
                    } else {
                        Some((midx, max))
                    }
                } else {
                    Some((idx, p))
                },
            )
        });
    (min.unwrap().0, max.unwrap().0)
}

#[cfg(test)]
mod test {
    use super::{partial_max, partial_min};

    #[test]
    fn test_partial_max() {
        assert_eq!(5, partial_max(5, 4));
        assert_eq!(5, partial_max(5, 5));
    }

    #[test]
    fn test_partial_min() {
        assert_eq!(4, partial_min(5, 4));
        assert_eq!(4, partial_min(4, 4));
    }
}
