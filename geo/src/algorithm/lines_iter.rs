use crate::{Coordinate, CoordNum, Line, LineString};
use core::slice;
use std::iter;

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


// ┌──────────────────────────────────┐
// │ Implementation for LineString    │
// └──────────────────────────────────┘

impl<'a, T: CoordNum + 'a> LinesIter<'a> for LineString<T> {
    type Scalar = T;
    type Iter = LineStringLinesIter<'a, T>;

    fn lines_iter(&'a self) -> Self::Iter {
        LineStringLinesIter::new(self)
    }
}

pub struct LineStringLinesIter<'a, T: CoordNum>(slice::Windows<'a, Coordinate<T>>);

impl<'a, T: CoordNum> LineStringLinesIter<'a, T> {
    fn new(line_string: &'a LineString<T>) -> Self {
        Self(line_string.0.windows(2))
    }
}

impl<'a, T: CoordNum> Iterator for LineStringLinesIter<'a, T> {
    type Item = Line<T>;

    fn next(&mut self) -> Option<Self::Item> {
        // Can't use LineString::lines() because it returns an `impl Trait`
        // and there is no way to name that type in `LinesIter::Iter` until [RFC 2071] is stabilized.
        //
        // [RFC 2071]: https://rust-lang.github.io/rfcs/2071-impl-trait-existential-types.html
        self.0.next().map(|w| {
            // slice::windows(2) is guaranteed to yield a slice with exactly 2 elements
            unsafe { Line::new(*w.get_unchecked(0), *w.get_unchecked(1)) }
        })
    }
}

#[cfg(test)]
mod test {

    use super::LinesIter;
    use crate::{line_string, Coordinate, Line, LineString};

    #[test]
    fn test_line() {
        let line = Line::new(Coordinate { x: 0., y: 0. }, Coordinate { x: 5., y: 10. });
        let want = vec![Line::new(
            Coordinate { x: 0., y: 0. },
            Coordinate { x: 5., y: 10. },
        )];
        assert_eq!(want, line.lines_iter().collect::<Vec<_>>());
    }

    #[test]
    fn test_empty_line_string() {
        let ls: LineString<f64> = line_string![];
        assert_eq!(Vec::<Line<f64>>::new(), ls.lines_iter().collect::<Vec<_>>());
    }

    #[test]
    fn test_open_line_string() {
        let ls = line_string![(x: 0., y: 0.), (x: 1., y: 1.), (x:2., y: 2.)];
        let want = vec![
            Line::new(Coordinate { x: 0., y: 0. }, Coordinate { x: 1., y: 1. }),
            Line::new(Coordinate { x: 1., y: 1. }, Coordinate { x: 2., y: 2. }),
        ];
        assert_eq!(want, ls.lines_iter().collect::<Vec<_>>());
    }

    #[test]
    fn test_closed_line_string() {
        let mut ls = line_string![(x: 0., y: 0.), (x: 1., y: 1.), (x:2., y: 2.)];
        ls.close();
        let want = vec![
            Line::new(Coordinate { x: 0., y: 0. }, Coordinate { x: 1., y: 1. }),
            Line::new(Coordinate { x: 1., y: 1. }, Coordinate { x: 2., y: 2. }),
            Line::new(Coordinate { x: 2., y: 2. }, Coordinate { x: 0., y: 0. }),
        ];
        assert_eq!(want, ls.lines_iter().collect::<Vec<_>>());
    }
}
