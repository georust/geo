use crate::{
    Coordinate, CoordinateType, Geometry, GeometryCollection, Line, LineString, MultiLineString,
    MultiPoint, MultiPolygon, Point, Polygon, Rect, Triangle,
};
use std::{iter, slice};

pub trait CoordsIter<'a, T: CoordinateType + 'a> {
    type Iter: Iterator<Item = Coordinate<T>>;

    fn coords_iter(&'a self) -> Self::Iter;
}

// ┌──────────────────────────┐
// │ Implementation for Point │
// └──────────────────────────┘

impl<'a, T: CoordinateType + 'a> CoordsIter<'a, T> for Point<T> {
    type Iter = iter::Once<Coordinate<T>>;

    fn coords_iter(&'a self) -> Self::Iter {
        iter::once(self.0)
    }
}

// ┌─────────────────────────┐
// │ Implementation for Line │
// └─────────────────────────┘

impl<'a, T: CoordinateType + 'a> CoordsIter<'a, T> for Line<T> {
    type Iter = iter::Chain<iter::Once<Coordinate<T>>, iter::Once<Coordinate<T>>>;

    fn coords_iter(&'a self) -> Self::Iter {
        iter::once(self.start).chain(iter::once(self.end))
    }
}

// ┌───────────────────────────────┐
// │ Implementation for LineString │
// └───────────────────────────────┘

impl<'a, T: CoordinateType + 'a> CoordsIter<'a, T> for LineString<T> {
    type Iter = iter::Copied<slice::Iter<'a, Coordinate<T>>>;

    fn coords_iter(&'a self) -> Self::Iter {
        self.0.iter().copied()
    }
}

// ┌────────────────────────────┐
// │ Implementation for Polygon │
// └────────────────────────────┘

impl<'a, T: CoordinateType + 'a> CoordsIter<'a, T> for Polygon<T> {
    type Iter =
        iter::Chain<<LineString<T> as CoordsIter<'a, T>>::Iter, LineStringsToCoordsIter<'a, T>>;

    fn coords_iter(&'a self) -> Self::Iter {
        self.exterior().coords_iter().chain(LineStringsToCoordsIter(
            LineStringsToCoordSliceIter(self.interiors().iter()).flatten(),
        ))
    }
}

// ┌───────────────────────────────┐
// │ Implementation for MultiPoint │
// └───────────────────────────────┘

impl<'a, T: CoordinateType + 'a> CoordsIter<'a, T> for MultiPoint<T> {
    type Iter = PointsToCoordsIter<'a, T>;

    fn coords_iter(&'a self) -> Self::Iter {
        PointsToCoordsIter(self.0.iter())
    }
}

// ┌────────────────────────────────────┐
// │ Implementation for MultiLineString │
// └────────────────────────────────────┘

impl<'a, T: CoordinateType + 'a> CoordsIter<'a, T> for MultiLineString<T> {
    type Iter = LineStringsToCoordsIter<'a, T>;

    fn coords_iter(&'a self) -> Self::Iter {
        LineStringsToCoordsIter(LineStringsToCoordSliceIter(self.0.iter()).flatten())
    }
}

// ┌─────────────────────────────────┐
// │ Implementation for MultiPolygon │
// └─────────────────────────────────┘

impl<'a, T: CoordinateType + 'a> CoordsIter<'a, T> for MultiPolygon<T> {
    type Iter = Box<dyn Iterator<Item = Coordinate<T>> + 'a>;

    fn coords_iter(&'a self) -> Self::Iter {
        Box::new(self.0.iter().flat_map(|m| m.coords_iter()))
    }
}

// ┌───────────────────────────────────────┐
// │ Implementation for GeometryCollection │
// └───────────────────────────────────────┘

impl<'a, T: CoordinateType + 'a> CoordsIter<'a, T> for GeometryCollection<T> {
    type Iter = iter::Flatten<GeometriesToGeometryCoordsIterIter<'a, T>>;

    fn coords_iter(&'a self) -> Self::Iter {
        GeometriesToGeometryCoordsIterIter(self.0.iter()).flatten()
    }
}

// ┌─────────────────────────┐
// │ Implementation for Rect │
// └─────────────────────────┘

impl<'a, T: CoordinateType + 'a> CoordsIter<'a, T> for Rect<T> {
    type Iter = iter::Chain<
        iter::Chain<
            iter::Chain<iter::Once<Coordinate<T>>, iter::Once<Coordinate<T>>>,
            iter::Once<Coordinate<T>>,
        >,
        iter::Once<Coordinate<T>>,
    >;

    fn coords_iter(&'a self) -> Self::Iter {
        iter::once(Coordinate {
            x: self.min().x,
            y: self.min().y,
        })
        .chain(iter::once(Coordinate {
            x: self.min().x,
            y: self.max().y,
        }))
        .chain(iter::once(Coordinate {
            x: self.max().x,
            y: self.max().y,
        }))
        .chain(iter::once(Coordinate {
            x: self.max().x,
            y: self.min().y,
        }))
    }
}

// ┌─────────────────────────────┐
// │ Implementation for Triangle │
// └─────────────────────────────┘

impl<'a, T: CoordinateType + 'a> CoordsIter<'a, T> for Triangle<T> {
    type Iter = iter::Chain<
        iter::Chain<iter::Once<Coordinate<T>>, iter::Once<Coordinate<T>>>,
        iter::Once<Coordinate<T>>,
    >;

    fn coords_iter(&'a self) -> Self::Iter {
        iter::once(self.0)
            .chain(iter::once(self.1))
            .chain(iter::once(self.2))
    }
}

// ┌─────────────────────────────┐
// │ Implementation for Geometry │
// └─────────────────────────────┘

impl<'a, T: CoordinateType + 'a> CoordsIter<'a, T> for Geometry<T> {
    type Iter = GeometryCoordsIter<'a, T>;

    fn coords_iter(&'a self) -> Self::Iter {
        match self {
            Geometry::Point(g) => GeometryCoordsIter::Point(g.coords_iter()),
            Geometry::Line(g) => GeometryCoordsIter::Line(g.coords_iter()),
            Geometry::LineString(g) => GeometryCoordsIter::LineString(g.coords_iter()),
            Geometry::Polygon(g) => GeometryCoordsIter::Polygon(g.coords_iter()),
            Geometry::MultiPoint(g) => GeometryCoordsIter::MultiPoint(g.coords_iter()),
            Geometry::MultiLineString(g) => GeometryCoordsIter::MultiLineString(g.coords_iter()),
            Geometry::MultiPolygon(g) => GeometryCoordsIter::MultiPolygon(g.coords_iter()),
            Geometry::GeometryCollection(g) => {
                GeometryCoordsIter::GeometryCollection(g.coords_iter())
            }
            Geometry::Rect(g) => GeometryCoordsIter::Rect(g.coords_iter()),
            Geometry::Triangle(g) => GeometryCoordsIter::Triangle(g.coords_iter()),
        }
    }
}

// ┌───────────┐
// │ Utilities │
// └───────────┘

// Utility to transform Iterator<Point> into Iterator<Coordinate>
#[doc(hidden)]
pub struct PointsToCoordsIter<'a, T: CoordinateType>(slice::Iter<'a, Point<T>>);

impl<'a, T: CoordinateType> Iterator for PointsToCoordsIter<'a, T> {
    type Item = Coordinate<T>;

    fn next(&mut self) -> Option<Self::Item> {
        self.0.next().map(|point| point.0)
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.0.size_hint()
    }
}

// Utility to transform Iterator<LineString> into Iterator<&[Coordinate]>
struct LineStringsToCoordSliceIter<'a, T: CoordinateType>(slice::Iter<'a, LineString<T>>);

impl<'a, T: CoordinateType> Iterator for LineStringsToCoordSliceIter<'a, T> {
    type Item = &'a [Coordinate<T>];

    fn next(&mut self) -> Option<Self::Item> {
        self.0.next().map(|n| &n.0[..])
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.0.size_hint()
    }
}

// Utility to transform Iterator<&[Coordinate]> into Iterator<Coordinate>
#[doc(hidden)]
pub struct LineStringsToCoordsIter<'a, T: CoordinateType>(
    iter::Flatten<LineStringsToCoordSliceIter<'a, T>>,
);

impl<'a, T: CoordinateType> Iterator for LineStringsToCoordsIter<'a, T> {
    type Item = Coordinate<T>;

    fn next(&mut self) -> Option<Self::Item> {
        self.0.next().copied()
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.0.size_hint()
    }
}

#[doc(hidden)]
// Utility to transform Iterator<Geometry> into Iterator<Iterator<Coordinate>>
pub struct GeometriesToGeometryCoordsIterIter<'a, T: CoordinateType>(slice::Iter<'a, Geometry<T>>);

impl<'a, T: CoordinateType> Iterator for GeometriesToGeometryCoordsIterIter<'a, T> {
    type Item = GeometryCoordsIter<'a, T>;

    fn next(&mut self) -> Option<Self::Item> {
        self.0.next().map(|n| match n {
            Geometry::Point(n) => GeometryCoordsIter::Point(n.coords_iter()),
            Geometry::Line(n) => GeometryCoordsIter::Line(n.coords_iter()),
            Geometry::LineString(n) => GeometryCoordsIter::LineString(n.coords_iter()),
            Geometry::Polygon(n) => GeometryCoordsIter::Polygon(n.coords_iter()),
            Geometry::MultiPoint(n) => GeometryCoordsIter::MultiPoint(n.coords_iter()),
            Geometry::MultiLineString(n) => GeometryCoordsIter::MultiLineString(n.coords_iter()),
            Geometry::MultiPolygon(n) => GeometryCoordsIter::MultiPolygon(n.coords_iter()),
            Geometry::GeometryCollection(n) => {
                GeometryCoordsIter::GeometryCollection(n.coords_iter())
            }
            Geometry::Rect(n) => GeometryCoordsIter::Rect(n.coords_iter()),
            Geometry::Triangle(n) => GeometryCoordsIter::Triangle(n.coords_iter()),
        })
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.0.size_hint()
    }
}

#[doc(hidden)]
pub enum GeometryCoordsIter<'a, T: CoordinateType> {
    Point(<Point<T> as CoordsIter<'a, T>>::Iter),
    Line(<Line<T> as CoordsIter<'a, T>>::Iter),
    LineString(<LineString<T> as CoordsIter<'a, T>>::Iter),
    Polygon(<Polygon<T> as CoordsIter<'a, T>>::Iter),
    MultiPoint(<MultiPoint<T> as CoordsIter<'a, T>>::Iter),
    MultiLineString(<MultiLineString<T> as CoordsIter<'a, T>>::Iter),
    MultiPolygon(<MultiPolygon<T> as CoordsIter<'a, T>>::Iter),
    GeometryCollection(<GeometryCollection<T> as CoordsIter<'a, T>>::Iter),
    Rect(<Rect<T> as CoordsIter<'a, T>>::Iter),
    Triangle(<Triangle<T> as CoordsIter<'a, T>>::Iter),
}

impl<'a, T: CoordinateType> Iterator for GeometryCoordsIter<'a, T> {
    type Item = Coordinate<T>;

    fn next(&mut self) -> Option<Self::Item> {
        match self {
            GeometryCoordsIter::Point(g) => g.next(),
            GeometryCoordsIter::Line(g) => g.next(),
            GeometryCoordsIter::LineString(g) => g.next(),
            GeometryCoordsIter::Polygon(g) => g.next(),
            GeometryCoordsIter::MultiPoint(g) => g.next(),
            GeometryCoordsIter::MultiLineString(g) => g.next(),
            GeometryCoordsIter::MultiPolygon(g) => g.next(),
            GeometryCoordsIter::GeometryCollection(g) => g.next(),
            GeometryCoordsIter::Rect(g) => g.next(),
            GeometryCoordsIter::Triangle(g) => g.next(),
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        match self {
            GeometryCoordsIter::Point(g) => g.size_hint(),
            GeometryCoordsIter::Line(g) => g.size_hint(),
            GeometryCoordsIter::LineString(g) => g.size_hint(),
            GeometryCoordsIter::Polygon(g) => g.size_hint(),
            GeometryCoordsIter::MultiPoint(g) => g.size_hint(),
            GeometryCoordsIter::MultiLineString(g) => g.size_hint(),
            GeometryCoordsIter::MultiPolygon(g) => g.size_hint(),
            GeometryCoordsIter::GeometryCollection(g) => g.size_hint(),
            GeometryCoordsIter::Rect(g) => g.size_hint(),
            GeometryCoordsIter::Triangle(g) => g.size_hint(),
        }
    }
}
