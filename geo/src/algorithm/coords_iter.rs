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
    type Iter = iter::Chain<
        <geo_types::LineString<T> as CoordsIter<'a, T>>::Iter,
        LineStringsToCoordsIter<'a, T>,
    >;

    fn coords_iter(&'a self) -> Self::Iter {
        self.exterior()
            .coords_iter()
            .chain(
                LineStringsToCoordsIter(
                    LineStringsToCoordSliceIter(self.interiors().iter()).flatten()
                )
            )
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
        LineStringsToCoordsIter(
            LineStringsToCoordSliceIter(self.0.iter()).flatten()
        )
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
    type Iter = Box<dyn Iterator<Item = Coordinate<T>> + 'a>;

    fn coords_iter(&'a self) -> Self::Iter {
        Box::new(self.0.iter().flat_map(|m| m.coords_iter()))
    }
}

// ┌─────────────────────────┐
// │ Implementation for Rect │
// └─────────────────────────┘

impl<'a, T: CoordinateType + 'a> CoordsIter<'a, T> for Rect<T> {
    type Iter = Box<dyn Iterator<Item = Coordinate<T>> + 'a>;

    fn coords_iter(&'a self) -> Self::Iter {
        Box::new(
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
            })),
        )
    }
}

// ┌─────────────────────────────┐
// │ Implementation for Triangle │
// └─────────────────────────────┘

impl<'a, T: CoordinateType + 'a> CoordsIter<'a, T> for Triangle<T> {
    type Iter = Box<dyn Iterator<Item = Coordinate<T>> + 'a>;

    fn coords_iter(&'a self) -> Self::Iter {
        Box::new(
            iter::once(self.0)
                .chain(iter::once(self.1))
                .chain(iter::once(self.2)),
        )
    }
}

// ┌─────────────────────────────┐
// │ Implementation for Geometry │
// └─────────────────────────────┘

impl<'a, T: CoordinateType + 'a> CoordsIter<'a, T> for Geometry<T> {
    type Iter = Box<dyn Iterator<Item = Coordinate<T>> + 'a>;

    fn coords_iter(&'a self) -> Self::Iter {
        match self {
            Geometry::Point(g) => Box::new(g.coords_iter()),
            Geometry::Line(g) => Box::new(g.coords_iter()),
            Geometry::LineString(g) => Box::new(g.coords_iter()),
            Geometry::Polygon(g) => Box::new(g.coords_iter()),
            Geometry::MultiPoint(g) => Box::new(g.coords_iter()),
            Geometry::MultiLineString(g) => Box::new(g.coords_iter()),
            Geometry::MultiPolygon(g) => g.coords_iter(),
            Geometry::GeometryCollection(g) => g.coords_iter(),
            Geometry::Rect(g) => g.coords_iter(),
            Geometry::Triangle(g) => g.coords_iter(),
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
pub struct LineStringsToCoordsIter<'a, T: CoordinateType>(iter::Flatten<LineStringsToCoordSliceIter<'a, T>>);

impl<'a, T: CoordinateType> Iterator for LineStringsToCoordsIter<'a, T> {
    type Item = Coordinate<T>;

    fn next(&mut self) -> Option<Self::Item> {
        self.0.next().copied()
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.0.size_hint()
    }
}
