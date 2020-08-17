//! Internal utility functions, types, and data structures.

use crate::contains::Contains;

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

/// Calculate the position of a `Coordinate` relative to a `LineString`
pub fn coord_pos_relative_to_line_string<T>(
    coord: crate::Coordinate<T>,
    linestring: &crate::LineString<T>,
) -> CoordPos
where
    T: num_traits::Float,
{
    // See: http://www.ecse.rpi.edu/Homepages/wrf/Research/Short_Notes/pnpoly.html
    //      http://geospatialpython.com/search
    //         ?updated-min=2011-01-01T00:00:00-06:00&updated-max=2012-01-01T00:00:00-06:00&max-results=19

    // LineString without points
    if linestring.0.is_empty() {
        return CoordPos::Outside;
    }
    // Point is on linestring
    if linestring.contains(&coord) {
        return CoordPos::OnBoundary;
    }

    let mut xints = T::zero();
    let mut crossings = 0;
    for line in linestring.lines() {
        if coord.y > line.start.y.min(line.end.y)
            && coord.y <= line.start.y.max(line.end.y)
            && coord.x <= line.start.x.max(line.end.x)
        {
            if line.start.y != line.end.y {
                xints = (coord.y - line.start.y) * (line.end.x - line.start.x)
                    / (line.end.y - line.start.y)
                    + line.start.x;
            }
            if (line.start.x == line.end.x) || (coord.x <= xints) {
                crossings += 1;
            }
        }
    }
    if crossings % 2 == 1 {
        CoordPos::Inside
    } else {
        CoordPos::Outside
    }
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
