use crate::{CoordNum, Line, LineString};
use std::iter::{self, FromFn};

pub trait LinesIter<'a> {
    type Scalar: CoordNum;
    type Iter: Iterator<Item = Line<Self::Scalar>>;

    fn lines_iter(&'a self) -> Self::Iter;
}

// ┌────────────────────────────┐
// │ Implementation for Line    │
// └────────────────────────────┘

impl<'a, T: CoordNum + 'a> LinesIter<'a> for Line<T> {
    type Scalar = T;
    type Iter = iter::Copied<iter::Once<&'a Line<T>>>;

    fn lines_iter(&'a self) -> Self::Iter {
        iter::once(self).copied()
    }
}

/*
// ┌──────────────────────────────────┐
// │ Implementation for LineString    │
// └──────────────────────────────────┘

impl<'a, T: CoordNum + 'a> LinesIter<'a, T> for LineString<T> {
    type Iter = iter::Map<iter::Windows<'a, Coordinate<T>>>;

    fn lines_iter(&'a self) -> Self::Iter {
        self.0.windows(2).map(|w| {
            // slice::windows(N) is guaranteed to yield a slice with exactly N elements
            unsafe { Line::new(*w.get_unchecked(0), *w.get_unchecked(1)) }
        })
    }
}

pub struct LinesIterIter<'a, T,

struct LineStringLinesIterator<T: CoordNum, Iter: Iterator<Item=Line<T>>> (Iter);

impl<T: CoordNum, Iter: Iterator<Item=Line<T>>> Iterator for LineStringLinesIterator<T, Iter> {
    type Item = Line<T>;

    fn next(&mut self) -> Option<Self::Item> {
        self.0.next()
    }
}
 */
#[cfg(test)]
mod test {

    use super::LinesIter;
    use crate::{Coordinate, Line};

    #[test]
    fn test_line() {
        let line = Line::new(Coordinate { x: 0., y: 0. }, Coordinate { x: 5., y: 10. });
        let want = vec![Line::new(
            Coordinate { x: 0., y: 0. },
            Coordinate { x: 5., y: 10. },
        )];
        assert_eq!(want, line.lines_iter().collect::<Vec<_>>());
    }
}
