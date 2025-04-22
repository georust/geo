use crate::{
    Coord, CoordNum, Line, LineString, MultiLineString, MultiPolygon, Polygon, Rect, Triangle,
};
use core::slice;
use std::fmt::Debug;
use std::iter;

/// Iterate over lines of a geometry.
pub trait LinesIter<'a> {
    type Scalar: CoordNum;
    type Iter: Iterator<Item = Line<Self::Scalar>>;

    /// Iterate over all exterior and (if any) interior lines of a geometry.
    ///
    /// # Examples
    ///
    /// ```
    /// use geo::line_string;
    /// use geo::lines_iter::LinesIter;
    /// use geo::{coord, Line};
    ///
    /// let ls = line_string![
    ///     (x: 1., y: 2.),
    ///     (x: 23., y: 82.),
    ///     (x: -1., y: 0.),
    /// ];
    ///
    /// let mut iter = ls.lines_iter();
    /// assert_eq!(
    ///     Some(Line::new(
    ///         coord! { x: 1., y: 2. },
    ///         coord! { x: 23., y: 82. }
    ///     )),
    ///     iter.next()
    /// );
    /// assert_eq!(
    ///     Some(Line::new(
    ///         coord! { x: 23., y: 82. },
    ///         coord! { x: -1., y: 0. }
    ///     )),
    ///     iter.next()
    /// );
    /// assert_eq!(None, iter.next());
    /// ```
    fn lines_iter(&'a self) -> Self::Iter;
}

impl<'a, T: CoordNum + 'a> LinesIter<'a> for Line<T> {
    type Scalar = T;
    type Iter = iter::Copied<iter::Once<&'a Line<Self::Scalar>>>;

    fn lines_iter(&'a self) -> Self::Iter {
        iter::once(self).copied()
    }
}

impl<'a, T: CoordNum + 'a> LinesIter<'a> for LineString<T> {
    type Scalar = T;
    type Iter = LineStringIter<'a, Self::Scalar>;

    fn lines_iter(&'a self) -> Self::Iter {
        LineStringIter::new(self)
    }
}

/// Iterator over lines in a [LineString].
#[derive(Debug)]
pub struct LineStringIter<'a, T: CoordNum>(slice::Windows<'a, Coord<T>>);

impl<'a, T: CoordNum> LineStringIter<'a, T> {
    fn new(line_string: &'a LineString<T>) -> Self {
        Self(line_string.0.windows(2))
    }
}

impl<T: CoordNum> Iterator for LineStringIter<'_, T> {
    type Item = Line<T>;

    fn next(&mut self) -> Option<Self::Item> {
        // Can't use LineString::lines() because it returns an `impl Trait`
        // and there is no way to name that type in `LinesIter::Iter` until [RFC 2071] is stabilized.
        //
        // [RFC 2071]: https://rust-lang.github.io/rfcs/2071-impl-trait-existential-types.html
        self.0.next().map(|w| {
            // SAFETY: slice::windows(2) is guaranteed to yield a slice with exactly 2 elements
            unsafe { Line::new(*w.get_unchecked(0), *w.get_unchecked(1)) }
        })
    }
}

type MultiLineStringIter<'a, T> =
    iter::Flatten<MapLinesIter<'a, slice::Iter<'a, LineString<T>>, LineString<T>>>;

impl<'a, T: CoordNum + 'a> LinesIter<'a> for MultiLineString<T> {
    type Scalar = T;
    type Iter = MultiLineStringIter<'a, Self::Scalar>;

    fn lines_iter(&'a self) -> Self::Iter {
        MapLinesIter(self.0.iter()).flatten()
    }
}

type PolygonIter<'a, T> = iter::Chain<
    LineStringIter<'a, T>,
    iter::Flatten<MapLinesIter<'a, slice::Iter<'a, LineString<T>>, LineString<T>>>,
>;

impl<'a, T: CoordNum + 'a> LinesIter<'a> for Polygon<T> {
    type Scalar = T;
    type Iter = PolygonIter<'a, Self::Scalar>;

    fn lines_iter(&'a self) -> Self::Iter {
        self.exterior()
            .lines_iter()
            .chain(MapLinesIter(self.interiors().iter()).flatten())
    }
}

type MultiPolygonIter<'a, T> =
    iter::Flatten<MapLinesIter<'a, slice::Iter<'a, Polygon<T>>, Polygon<T>>>;

impl<'a, T: CoordNum + 'a> LinesIter<'a> for MultiPolygon<T> {
    type Scalar = T;
    type Iter = MultiPolygonIter<'a, Self::Scalar>;

    fn lines_iter(&'a self) -> Self::Iter {
        MapLinesIter(self.0.iter()).flatten()
    }
}

impl<'a, T: CoordNum + 'a> LinesIter<'a> for Rect<T> {
    type Scalar = T;
    type Iter = <[Line<Self::Scalar>; 4] as IntoIterator>::IntoIter;

    fn lines_iter(&'a self) -> Self::Iter {
        self.to_lines().into_iter()
    }
}

impl<'a, T: CoordNum + 'a> LinesIter<'a> for Triangle<T> {
    type Scalar = T;
    type Iter = <[Line<Self::Scalar>; 3] as IntoIterator>::IntoIter;

    fn lines_iter(&'a self) -> Self::Iter {
        self.to_lines().into_iter()
    }
}

/// Utility to transform `Iterator<LinesIter>` into `Iterator<Iterator<Line>>`.
#[derive(Debug)]
pub struct MapLinesIter<'a, Iter1: Iterator<Item = &'a Iter2>, Iter2: 'a + LinesIter<'a>>(Iter1);

impl<'a, Iter1: Iterator<Item = &'a Iter2>, Iter2: LinesIter<'a>> Iterator
    for MapLinesIter<'a, Iter1, Iter2>
{
    type Item = Iter2::Iter;

    fn next(&mut self) -> Option<Self::Item> {
        self.0.next().map(|g| g.lines_iter())
    }
}

#[cfg(test)]
mod test {

    use super::LinesIter;
    use crate::{
        coord, line_string, polygon, Line, LineString, MultiLineString, MultiPolygon, Rect,
        Triangle,
    };

    #[test]
    fn test_line() {
        let line = Line::new(coord! { x: 0., y: 0. }, coord! { x: 5., y: 10. });
        let want = vec![Line::new(coord! { x: 0., y: 0. }, coord! { x: 5., y: 10. })];
        assert_eq!(want, line.lines_iter().collect::<Vec<_>>());
    }

    #[test]
    fn test_empty_line_string() {
        let ls: LineString = line_string![];
        assert_eq!(Vec::<Line>::new(), ls.lines_iter().collect::<Vec<_>>());
    }

    #[test]
    fn test_open_line_string() {
        let ls = line_string![(x: 0., y: 0.), (x: 1., y: 1.), (x:2., y: 2.)];
        let want = vec![
            Line::new(coord! { x: 0., y: 0. }, coord! { x: 1., y: 1. }),
            Line::new(coord! { x: 1., y: 1. }, coord! { x: 2., y: 2. }),
        ];
        assert_eq!(want, ls.lines_iter().collect::<Vec<_>>());
    }

    #[test]
    fn test_closed_line_string() {
        let mut ls = line_string![(x: 0., y: 0.), (x: 1., y: 1.), (x:2., y: 2.)];
        ls.close();
        let want = vec![
            Line::new(coord! { x: 0., y: 0. }, coord! { x: 1., y: 1. }),
            Line::new(coord! { x: 1., y: 1. }, coord! { x: 2., y: 2. }),
            Line::new(coord! { x: 2., y: 2. }, coord! { x: 0., y: 0. }),
        ];
        assert_eq!(want, ls.lines_iter().collect::<Vec<_>>());
    }

    #[test]
    fn test_multi_line_string() {
        let mls = MultiLineString::new(vec![
            line_string![],
            line_string![(x: 0., y: 0.), (x: 1., y: 1.)],
            line_string![(x: 0., y: 0.), (x: 1., y: 1.), (x:2., y: 2.)],
        ]);
        let want = vec![
            Line::new(coord! { x: 0., y: 0. }, coord! { x: 1., y: 1. }),
            Line::new(coord! { x: 0., y: 0. }, coord! { x: 1., y: 1. }),
            Line::new(coord! { x: 1., y: 1. }, coord! { x: 2., y: 2. }),
        ];
        assert_eq!(want, mls.lines_iter().collect::<Vec<_>>());
    }

    #[test]
    fn test_polygon() {
        let p = polygon!(
            exterior: [(x: 0., y: 0.), (x: 0., y: 10.), (x: 10., y: 10.), (x: 10., y: 0.)],
            interiors: [
                [(x: 1., y: 1.), (x: 1., y: 2.), (x: 2., y: 2.), (x: 2., y: 1.)],
                [(x: 3., y: 3.), (x: 5., y: 3.), (x: 5., y: 5.), (x: 3., y: 5.)],
            ],
        );
        let want = vec![
            // exterior ring
            Line::new(coord! { x: 0., y: 0. }, coord! { x: 0., y: 10. }),
            Line::new(coord! { x: 0., y: 10. }, coord! { x: 10., y: 10. }),
            Line::new(coord! { x: 10., y: 10. }, coord! { x: 10., y: 0. }),
            Line::new(coord! { x: 10., y: 0. }, coord! { x: 0., y: 0. }),
            // first interior ring
            Line::new(coord! { x: 1., y: 1. }, coord! { x: 1., y: 2. }),
            Line::new(coord! { x: 1., y: 2. }, coord! { x: 2., y: 2. }),
            Line::new(coord! { x: 2., y: 2. }, coord! { x: 2., y: 1. }),
            Line::new(coord! { x: 2., y: 1. }, coord! { x: 1., y: 1. }),
            // second interior ring
            Line::new(coord! { x: 3., y: 3. }, coord! { x: 5., y: 3. }),
            Line::new(coord! { x: 5., y: 3. }, coord! { x: 5., y: 5. }),
            Line::new(coord! { x: 5., y: 5. }, coord! { x: 3., y: 5. }),
            Line::new(coord! { x: 3., y: 5. }, coord! { x: 3., y: 3. }),
        ];
        assert_eq!(want, p.lines_iter().collect::<Vec<_>>());
    }

    #[test]
    fn test_multi_polygon() {
        let mp = MultiPolygon::new(vec![
            polygon!(
                exterior: [(x: 0., y: 0.), (x: 0., y: 10.), (x: 10., y: 10.), (x: 10., y: 0.)],
                interiors: [[(x: 1., y: 1.), (x: 1., y: 2.), (x: 2., y: 2.), (x: 2., y: 1.)]],
            ),
            polygon!(
                exterior: [(x: 3., y: 3.), (x: 5., y: 3.), (x: 5., y: 5.), (x: 3., y: 5.)],
                interiors: [],
            ),
        ]);
        let want = vec![
            // first polygon - exterior ring
            Line::new(coord! { x: 0., y: 0. }, coord! { x: 0., y: 10. }),
            Line::new(coord! { x: 0., y: 10. }, coord! { x: 10., y: 10. }),
            Line::new(coord! { x: 10., y: 10. }, coord! { x: 10., y: 0. }),
            Line::new(coord! { x: 10., y: 0. }, coord! { x: 0., y: 0. }),
            // first polygon - interior ring
            Line::new(coord! { x: 1., y: 1. }, coord! { x: 1., y: 2. }),
            Line::new(coord! { x: 1., y: 2. }, coord! { x: 2., y: 2. }),
            Line::new(coord! { x: 2., y: 2. }, coord! { x: 2., y: 1. }),
            Line::new(coord! { x: 2., y: 1. }, coord! { x: 1., y: 1. }),
            // second polygon - exterior ring
            Line::new(coord! { x: 3., y: 3. }, coord! { x: 5., y: 3. }),
            Line::new(coord! { x: 5., y: 3. }, coord! { x: 5., y: 5. }),
            Line::new(coord! { x: 5., y: 5. }, coord! { x: 3., y: 5. }),
            Line::new(coord! { x: 3., y: 5. }, coord! { x: 3., y: 3. }),
        ];
        assert_eq!(want, mp.lines_iter().collect::<Vec<_>>());
    }

    #[test]
    fn test_rect() {
        let rect = Rect::new(coord! { x: 0., y: 0. }, coord! { x: 1., y: 2. });
        let want = rect.to_polygon().lines_iter().collect::<Vec<_>>();
        assert_eq!(want, rect.lines_iter().collect::<Vec<_>>());
    }

    #[test]
    fn test_triangle() {
        let triangle = Triangle::new(
            coord! { x: 0., y: 0. },
            coord! { x: 1., y: 2. },
            coord! { x: 2., y: 3. },
        );
        let want = triangle.to_polygon().lines_iter().collect::<Vec<_>>();
        assert_eq!(want, triangle.lines_iter().collect::<Vec<_>>());
    }
}
