use crate::{
    Coordinate, CoordinateType, Geometry, GeometryCollection, Line, LineString, MultiLineString,
    MultiPoint, MultiPolygon, Point, Polygon, Rect, Triangle,
};
use std::{iter, marker, slice};

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
        <LineString<T> as CoordsIter<'a, T>>::Iter,
        iter::Flatten<MapCoordsIter<'a, T, slice::Iter<'a, LineString<T>>, LineString<T>>>,
    >;

    fn coords_iter(&'a self) -> Self::Iter {
        self.exterior()
            .coords_iter()
            .chain(MapCoordsIter(self.interiors().iter(), marker::PhantomData).flatten())
    }
}

// ┌───────────────────────────────┐
// │ Implementation for MultiPoint │
// └───────────────────────────────┘

impl<'a, T: CoordinateType + 'a> CoordsIter<'a, T> for MultiPoint<T> {
    type Iter = iter::Flatten<MapCoordsIter<'a, T, slice::Iter<'a, Point<T>>, Point<T>>>;

    fn coords_iter(&'a self) -> Self::Iter {
        MapCoordsIter(self.0.iter(), marker::PhantomData).flatten()
    }
}

// ┌────────────────────────────────────┐
// │ Implementation for MultiLineString │
// └────────────────────────────────────┘

impl<'a, T: CoordinateType + 'a> CoordsIter<'a, T> for MultiLineString<T> {
    type Iter = iter::Flatten<MapCoordsIter<'a, T, slice::Iter<'a, LineString<T>>, LineString<T>>>;

    fn coords_iter(&'a self) -> Self::Iter {
        MapCoordsIter(self.0.iter(), marker::PhantomData).flatten()
    }
}

// ┌─────────────────────────────────┐
// │ Implementation for MultiPolygon │
// └─────────────────────────────────┘

impl<'a, T: CoordinateType + 'a> CoordsIter<'a, T> for MultiPolygon<T> {
    type Iter = iter::Flatten<MapCoordsIter<'a, T, slice::Iter<'a, Polygon<T>>, Polygon<T>>>;

    fn coords_iter(&'a self) -> Self::Iter {
        MapCoordsIter(self.0.iter(), marker::PhantomData).flatten()
    }
}

// ┌───────────────────────────────────────┐
// │ Implementation for GeometryCollection │
// └───────────────────────────────────────┘

impl<'a, T: CoordinateType + 'a> CoordsIter<'a, T> for GeometryCollection<T> {
    type Iter = iter::Flatten<MapCoordsIter<'a, T, slice::Iter<'a, Geometry<T>>, Geometry<T>>>;

    fn coords_iter(&'a self) -> Self::Iter {
        MapCoordsIter(self.0.iter(), marker::PhantomData).flatten()
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

// Utility to transform Iterator<CoordsIter> into Iterator<CoordsIter::Iter>
#[doc(hidden)]
pub struct MapCoordsIter<
    'a,
    T: 'a + CoordinateType,
    Iter1: Iterator<Item = &'a Iter2>,
    Iter2: 'a + CoordsIter<'a, T>,
>(Iter1, marker::PhantomData<T>);

impl<'a, T: 'a + CoordinateType, Iter1: Iterator<Item = &'a Iter2>, Iter2: CoordsIter<'a, T>>
    Iterator for MapCoordsIter<'a, T, Iter1, Iter2>
{
    type Item = Iter2::Iter;

    fn next(&mut self) -> Option<Self::Item> {
        self.0.next().map(|g| g.coords_iter())
    }
}

// Utility to transform Geometry into Iterator<Coordinate>
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
