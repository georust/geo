use std::fmt::Debug;

use crate::geometry::*;
use crate::{coord, CoordNum};

use std::{fmt, iter, marker, slice};

type CoordinateChainOnce<T> = iter::Chain<iter::Once<Coord<T>>, iter::Once<Coord<T>>>;

/// Iterate over geometry coordinates.
pub trait CoordsIter {
    type Iter<'a>: Iterator<Item = Coord<Self::Scalar>>
    where
        Self: 'a;
    type ExteriorIter<'a>: Iterator<Item = Coord<Self::Scalar>>
    where
        Self: 'a;
    type Scalar: CoordNum;

    /// Iterate over all exterior and (if any) interior coordinates of a geometry.
    ///
    /// # Examples
    ///
    /// ```
    /// use geo::coords_iter::CoordsIter;
    ///
    /// let multi_point = geo::MultiPoint::new(vec![
    ///     geo::point!(x: -10., y: 0.),
    ///     geo::point!(x: 20., y: 20.),
    ///     geo::point!(x: 30., y: 40.),
    /// ]);
    ///
    /// let mut iter = multi_point.coords_iter();
    /// assert_eq!(Some(geo::coord! { x: -10., y: 0. }), iter.next());
    /// assert_eq!(Some(geo::coord! { x: 20., y: 20. }), iter.next());
    /// assert_eq!(Some(geo::coord! { x: 30., y: 40. }), iter.next());
    /// assert_eq!(None, iter.next());
    /// ```
    fn coords_iter(&self) -> Self::Iter<'_>;

    /// Return the number of coordinates in a geometry.
    ///
    /// # Examples
    ///
    /// ```
    /// use geo::coords_iter::CoordsIter;
    /// use geo::line_string;
    ///
    /// let ls = line_string![
    ///     (x: 1., y: 2.),
    ///     (x: 23., y: 82.),
    ///     (x: -1., y: 0.),
    /// ];
    ///
    /// assert_eq!(3, ls.coords_count());
    /// ```
    fn coords_count(&self) -> usize;

    /// Iterate over all exterior coordinates of a geometry.
    ///
    /// # Examples
    ///
    /// ```
    /// use geo::coords_iter::CoordsIter;
    /// use geo::polygon;
    ///
    /// // a diamond shape
    /// let polygon = polygon![
    ///     exterior: [
    ///         (x: 1.0, y: 0.0),
    ///         (x: 2.0, y: 1.0),
    ///         (x: 1.0, y: 2.0),
    ///         (x: 0.0, y: 1.0),
    ///         (x: 1.0, y: 0.0),
    ///     ],
    ///     interiors: [
    ///         [
    ///             (x: 1.0, y: 0.5),
    ///             (x: 0.5, y: 1.0),
    ///             (x: 1.0, y: 1.5),
    ///             (x: 1.5, y: 1.0),
    ///             (x: 1.0, y: 0.5),
    ///         ],
    ///     ],
    /// ];
    ///
    /// let mut iter = polygon.exterior_coords_iter();
    /// assert_eq!(Some(geo::coord! { x: 1., y: 0. }), iter.next());
    /// assert_eq!(Some(geo::coord! { x: 2., y: 1. }), iter.next());
    /// assert_eq!(Some(geo::coord! { x: 1., y: 2. }), iter.next());
    /// assert_eq!(Some(geo::coord! { x: 0., y: 1. }), iter.next());
    /// assert_eq!(Some(geo::coord! { x: 1., y: 0. }), iter.next());
    /// assert_eq!(None, iter.next());
    /// ```
    fn exterior_coords_iter(&self) -> Self::ExteriorIter<'_>;
}

// ┌──────────────────────────┐
// │ Implementation for Point │
// └──────────────────────────┘

impl<T: CoordNum> CoordsIter for Point<T> {
    type Iter<'a> = iter::Once<Coord<T>> where T: 'a;
    type ExteriorIter<'a> = Self::Iter<'a> where T: 'a;
    type Scalar = T;

    fn coords_iter(&self) -> Self::Iter<'_> {
        iter::once(self.0)
    }

    /// Return the number of coordinates in the `Point`.
    fn coords_count(&self) -> usize {
        1
    }

    fn exterior_coords_iter(&self) -> Self::ExteriorIter<'_> {
        self.coords_iter()
    }
}

// ┌─────────────────────────┐
// │ Implementation for Line │
// └─────────────────────────┘

impl<T: CoordNum> CoordsIter for Line<T> {
    type Iter<'a> = iter::Chain<iter::Once<Coord<T>>, iter::Once<Coord<T>>>
    where T: 'a;
    type ExteriorIter<'a> = Self::Iter<'a>
    where T: 'a;
    type Scalar = T;

    fn coords_iter(&self) -> Self::Iter<'_> {
        iter::once(self.start).chain(iter::once(self.end))
    }

    /// Return the number of coordinates in the `Line`.
    fn coords_count(&self) -> usize {
        2
    }

    fn exterior_coords_iter(&self) -> Self::ExteriorIter<'_> {
        self.coords_iter()
    }
}

// ┌───────────────────────────────┐
// │ Implementation for LineString │
// └───────────────────────────────┘

type LineStringIter<'a, T> = iter::Copied<slice::Iter<'a, Coord<T>>>;

impl<T: CoordNum> CoordsIter for LineString<T> {
    type Iter<'a> = LineStringIter<'a, T>
    where T: 'a;
    type ExteriorIter<'a> = Self::Iter<'a>
    where T: 'a;
    type Scalar = T;

    fn coords_iter(&self) -> Self::Iter<'_> {
        self.0.iter().copied()
    }

    /// Return the number of coordinates in the `LineString`.
    fn coords_count(&self) -> usize {
        self.0.len()
    }

    fn exterior_coords_iter(&self) -> Self::ExteriorIter<'_> {
        self.coords_iter()
    }
}

// ┌────────────────────────────┐
// │ Implementation for Polygon │
// └────────────────────────────┘

type PolygonIter<'a, T> = iter::Chain<
    LineStringIter<'a, T>,
    iter::Flatten<MapCoordsIter<'a, T, slice::Iter<'a, LineString<T>>, LineString<T>>>,
>;

impl<T: CoordNum> CoordsIter for Polygon<T> {
    type Iter<'a> = PolygonIter<'a, T>
    where T: 'a;
    type ExteriorIter<'a> = LineStringIter<'a, T>
    where T: 'a;
    type Scalar = T;

    fn coords_iter(&self) -> Self::Iter<'_> {
        self.exterior()
            .coords_iter()
            .chain(MapCoordsIter(self.interiors().iter(), marker::PhantomData).flatten())
    }

    /// Return the number of coordinates in the `Polygon`.
    fn coords_count(&self) -> usize {
        self.exterior().coords_count()
            + self
                .interiors()
                .iter()
                .map(|i| i.coords_count())
                .sum::<usize>()
    }

    fn exterior_coords_iter(&self) -> Self::ExteriorIter<'_> {
        self.exterior().coords_iter()
    }
}

// ┌───────────────────────────────┐
// │ Implementation for MultiPoint │
// └───────────────────────────────┘

impl<T: CoordNum> CoordsIter for MultiPoint<T> {
    type Iter<'a> = iter::Flatten<MapCoordsIter<'a, T, slice::Iter<'a, Point<T>>, Point<T>>>
    where T: 'a;
    type ExteriorIter<'a> = Self::Iter<'a>
    where T: 'a;
    type Scalar = T;

    fn coords_iter(&self) -> Self::Iter<'_> {
        MapCoordsIter(self.0.iter(), marker::PhantomData).flatten()
    }

    /// Return the number of coordinates in the `MultiPoint`.
    fn coords_count(&self) -> usize {
        self.0.len()
    }

    fn exterior_coords_iter(&self) -> Self::ExteriorIter<'_> {
        self.coords_iter()
    }
}

// ┌────────────────────────────────────┐
// │ Implementation for MultiLineString │
// └────────────────────────────────────┘

impl<T: CoordNum> CoordsIter for MultiLineString<T> {
    type Iter<'a> =
        iter::Flatten<MapCoordsIter<'a, T, slice::Iter<'a, LineString<T>>, LineString<T>>>
    where T: 'a;
    type ExteriorIter<'a> = Self::Iter<'a>
    where T: 'a;
    type Scalar = T;

    fn coords_iter(&self) -> Self::Iter<'_> {
        MapCoordsIter(self.0.iter(), marker::PhantomData).flatten()
    }

    /// Return the number of coordinates in the `MultiLineString`.
    fn coords_count(&self) -> usize {
        self.0
            .iter()
            .map(|line_string| line_string.coords_count())
            .sum()
    }

    fn exterior_coords_iter(&self) -> Self::ExteriorIter<'_> {
        self.coords_iter()
    }
}

// ┌─────────────────────────────────┐
// │ Implementation for MultiPolygon │
// └─────────────────────────────────┘

impl<T: CoordNum> CoordsIter for MultiPolygon<T> {
    type Iter<'a> = iter::Flatten<MapCoordsIter<'a, T, slice::Iter<'a, Polygon<T>>, Polygon<T>>>
    where T: 'a;
    type ExteriorIter<'a> =
        iter::Flatten<MapExteriorCoordsIter<'a, T, slice::Iter<'a, Polygon<T>>, Polygon<T>>>
    where T: 'a;
    type Scalar = T;

    fn coords_iter(&self) -> Self::Iter<'_> {
        MapCoordsIter(self.0.iter(), marker::PhantomData).flatten()
    }

    /// Return the number of coordinates in the `MultiPolygon`.
    fn coords_count(&self) -> usize {
        self.0.iter().map(|polygon| polygon.coords_count()).sum()
    }

    fn exterior_coords_iter(&self) -> Self::ExteriorIter<'_> {
        MapExteriorCoordsIter(self.0.iter(), marker::PhantomData).flatten()
    }
}

// ┌───────────────────────────────────────┐
// │ Implementation for GeometryCollection │
// └───────────────────────────────────────┘

impl<T: CoordNum> CoordsIter for GeometryCollection<T> {
    type Iter<'a> = Box<dyn Iterator<Item = Coord<T>> + 'a>
    where T: 'a;
    type ExteriorIter<'a> = Box<dyn Iterator<Item = Coord<T>> + 'a>
    where T: 'a;
    type Scalar = T;

    fn coords_iter(&self) -> Self::Iter<'_> {
        Box::new(self.0.iter().flat_map(|geometry| geometry.coords_iter()))
    }

    /// Return the number of coordinates in the `GeometryCollection`.
    fn coords_count(&self) -> usize {
        self.0.iter().map(|geometry| geometry.coords_count()).sum()
    }

    fn exterior_coords_iter(&self) -> Self::ExteriorIter<'_> {
        Box::new(
            self.0
                .iter()
                .flat_map(|geometry| geometry.exterior_coords_iter()),
        )
    }
}

// ┌─────────────────────────┐
// │ Implementation for Rect │
// └─────────────────────────┘

type RectIter<T> =
    iter::Chain<iter::Chain<CoordinateChainOnce<T>, iter::Once<Coord<T>>>, iter::Once<Coord<T>>>;

impl<T: CoordNum> CoordsIter for Rect<T> {
    type Iter<'a> = RectIter<T>
    where T: 'a;
    type ExteriorIter<'a> = Self::Iter<'a>
    where T: 'a;
    type Scalar = T;

    fn coords_iter(&self) -> Self::Iter<'_> {
        iter::once(coord! {
            x: self.min().x,
            y: self.min().y,
        })
        .chain(iter::once(coord! {
            x: self.min().x,
            y: self.max().y,
        }))
        .chain(iter::once(coord! {
            x: self.max().x,
            y: self.max().y,
        }))
        .chain(iter::once(coord! {
            x: self.max().x,
            y: self.min().y,
        }))
    }

    /// Return the number of coordinates in the `Rect`.
    ///
    /// Note: Although a `Rect` is represented by two coordinates, it is
    /// spatially represented by four, so this method returns `4`.
    fn coords_count(&self) -> usize {
        4
    }

    fn exterior_coords_iter(&self) -> Self::ExteriorIter<'_> {
        self.coords_iter()
    }
}

// ┌─────────────────────────────┐
// │ Implementation for Triangle │
// └─────────────────────────────┘

impl<T: CoordNum> CoordsIter for Triangle<T> {
    type Iter<'a> = iter::Chain<CoordinateChainOnce<T>, iter::Once<Coord<T>>>
    where T: 'a;
    type ExteriorIter<'a> = Self::Iter<'a>
    where T: 'a;
    type Scalar = T;

    fn coords_iter(&self) -> Self::Iter<'_> {
        iter::once(self.0)
            .chain(iter::once(self.1))
            .chain(iter::once(self.2))
    }

    /// Return the number of coordinates in the `Triangle`.
    fn coords_count(&self) -> usize {
        3
    }

    fn exterior_coords_iter(&self) -> Self::ExteriorIter<'_> {
        self.coords_iter()
    }
}

// ┌─────────────────────────────┐
// │ Implementation for Geometry │
// └─────────────────────────────┘

impl<T: CoordNum> CoordsIter for Geometry<T> {
    type Iter<'a> = GeometryCoordsIter<'a, T>
    where T: 'a;
    type ExteriorIter<'a> = GeometryExteriorCoordsIter<'a, T>
    where T: 'a;
    type Scalar = T;

    fn coords_iter(&self) -> Self::Iter<'_> {
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
    crate::geometry_delegate_impl! {
        /// Return the number of coordinates in the `Geometry`.
        fn coords_count(&self) -> usize;
    }

    fn exterior_coords_iter(&self) -> Self::ExteriorIter<'_> {
        match self {
            Geometry::Point(g) => GeometryExteriorCoordsIter::Point(g.exterior_coords_iter()),
            Geometry::Line(g) => GeometryExteriorCoordsIter::Line(g.exterior_coords_iter()),
            Geometry::LineString(g) => {
                GeometryExteriorCoordsIter::LineString(g.exterior_coords_iter())
            }
            Geometry::Polygon(g) => GeometryExteriorCoordsIter::Polygon(g.exterior_coords_iter()),
            Geometry::MultiPoint(g) => {
                GeometryExteriorCoordsIter::MultiPoint(g.exterior_coords_iter())
            }
            Geometry::MultiLineString(g) => {
                GeometryExteriorCoordsIter::MultiLineString(g.exterior_coords_iter())
            }
            Geometry::MultiPolygon(g) => {
                GeometryExteriorCoordsIter::MultiPolygon(g.exterior_coords_iter())
            }
            Geometry::GeometryCollection(g) => {
                GeometryExteriorCoordsIter::GeometryCollection(g.exterior_coords_iter())
            }
            Geometry::Rect(g) => GeometryExteriorCoordsIter::Rect(g.exterior_coords_iter()),
            Geometry::Triangle(g) => GeometryExteriorCoordsIter::Triangle(g.exterior_coords_iter()),
        }
    }
}

// ┌───────────┐
// │ Utilities │
// └───────────┘

// Utility to transform Iterator<CoordsIter> into Iterator<Iterator<Coord>>
#[doc(hidden)]
#[derive(Debug)]
pub struct MapCoordsIter<
    'a,
    T: 'a + CoordNum,
    Iter1: Iterator<Item = &'a Iter2>,
    Iter2: 'a + CoordsIter,
>(Iter1, marker::PhantomData<T>);

impl<'a, T: 'a + CoordNum, Iter1: Iterator<Item = &'a Iter2>, Iter2: CoordsIter> Iterator
    for MapCoordsIter<'a, T, Iter1, Iter2>
{
    type Item = Iter2::Iter<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        self.0.next().map(|g| g.coords_iter())
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.0.size_hint()
    }
}

// Utility to transform Iterator<CoordsIter> into Iterator<Iterator<Coord>>
#[doc(hidden)]
#[derive(Debug)]
pub struct MapExteriorCoordsIter<
    'a,
    T: 'a + CoordNum,
    Iter1: Iterator<Item = &'a Iter2>,
    Iter2: 'a + CoordsIter,
>(Iter1, marker::PhantomData<T>);

impl<'a, T: 'a + CoordNum, Iter1: Iterator<Item = &'a Iter2>, Iter2: CoordsIter> Iterator
    for MapExteriorCoordsIter<'a, T, Iter1, Iter2>
{
    type Item = Iter2::ExteriorIter<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        self.0.next().map(|g| g.exterior_coords_iter())
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.0.size_hint()
    }
}

// Utility to transform Geometry into Iterator<Coord>
#[doc(hidden)]
pub enum GeometryCoordsIter<'a, T: CoordNum + 'a> {
    Point(<Point<T> as CoordsIter>::Iter<'a>),
    Line(<Line<T> as CoordsIter>::Iter<'a>),
    LineString(<LineString<T> as CoordsIter>::Iter<'a>),
    Polygon(<Polygon<T> as CoordsIter>::Iter<'a>),
    MultiPoint(<MultiPoint<T> as CoordsIter>::Iter<'a>),
    MultiLineString(<MultiLineString<T> as CoordsIter>::Iter<'a>),
    MultiPolygon(<MultiPolygon<T> as CoordsIter>::Iter<'a>),
    GeometryCollection(<GeometryCollection<T> as CoordsIter>::Iter<'a>),
    Rect(<Rect<T> as CoordsIter>::Iter<'a>),
    Triangle(<Triangle<T> as CoordsIter>::Iter<'a>),
}

impl<'a, T: CoordNum> Iterator for GeometryCoordsIter<'a, T> {
    type Item = Coord<T>;

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

impl<'a, T: CoordNum + Debug> fmt::Debug for GeometryCoordsIter<'a, T> {
    fn fmt(&self, fmt: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            GeometryCoordsIter::Point(i) => fmt.debug_tuple("Point").field(i).finish(),
            GeometryCoordsIter::Line(i) => fmt.debug_tuple("Line").field(i).finish(),
            GeometryCoordsIter::LineString(i) => fmt.debug_tuple("LineString").field(i).finish(),
            GeometryCoordsIter::Polygon(i) => fmt.debug_tuple("Polygon").field(i).finish(),
            GeometryCoordsIter::MultiPoint(i) => fmt.debug_tuple("MultiPoint").field(i).finish(),
            GeometryCoordsIter::MultiLineString(i) => {
                fmt.debug_tuple("MultiLineString").field(i).finish()
            }
            GeometryCoordsIter::MultiPolygon(i) => {
                fmt.debug_tuple("MultiPolygon").field(i).finish()
            }
            GeometryCoordsIter::GeometryCollection(_) => fmt
                .debug_tuple("GeometryCollection")
                .field(&String::from("..."))
                .finish(),
            GeometryCoordsIter::Rect(i) => fmt.debug_tuple("Rect").field(i).finish(),
            GeometryCoordsIter::Triangle(i) => fmt.debug_tuple("Triangle").field(i).finish(),
        }
    }
}

// Utility to transform Geometry into Iterator<Coord>
#[doc(hidden)]
pub enum GeometryExteriorCoordsIter<'a, T: CoordNum + 'a> {
    Point(<Point<T> as CoordsIter>::ExteriorIter<'a>),
    Line(<Line<T> as CoordsIter>::ExteriorIter<'a>),
    LineString(<LineString<T> as CoordsIter>::ExteriorIter<'a>),
    Polygon(<Polygon<T> as CoordsIter>::ExteriorIter<'a>),
    MultiPoint(<MultiPoint<T> as CoordsIter>::ExteriorIter<'a>),
    MultiLineString(<MultiLineString<T> as CoordsIter>::ExteriorIter<'a>),
    MultiPolygon(<MultiPolygon<T> as CoordsIter>::ExteriorIter<'a>),
    GeometryCollection(<GeometryCollection<T> as CoordsIter>::ExteriorIter<'a>),
    Rect(<Rect<T> as CoordsIter>::ExteriorIter<'a>),
    Triangle(<Triangle<T> as CoordsIter>::ExteriorIter<'a>),
}

impl<'a, T: CoordNum> Iterator for GeometryExteriorCoordsIter<'a, T> {
    type Item = Coord<T>;

    fn next(&mut self) -> Option<Self::Item> {
        match self {
            GeometryExteriorCoordsIter::Point(g) => g.next(),
            GeometryExteriorCoordsIter::Line(g) => g.next(),
            GeometryExteriorCoordsIter::LineString(g) => g.next(),
            GeometryExteriorCoordsIter::Polygon(g) => g.next(),
            GeometryExteriorCoordsIter::MultiPoint(g) => g.next(),
            GeometryExteriorCoordsIter::MultiLineString(g) => g.next(),
            GeometryExteriorCoordsIter::MultiPolygon(g) => g.next(),
            GeometryExteriorCoordsIter::GeometryCollection(g) => g.next(),
            GeometryExteriorCoordsIter::Rect(g) => g.next(),
            GeometryExteriorCoordsIter::Triangle(g) => g.next(),
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        match self {
            GeometryExteriorCoordsIter::Point(g) => g.size_hint(),
            GeometryExteriorCoordsIter::Line(g) => g.size_hint(),
            GeometryExteriorCoordsIter::LineString(g) => g.size_hint(),
            GeometryExteriorCoordsIter::Polygon(g) => g.size_hint(),
            GeometryExteriorCoordsIter::MultiPoint(g) => g.size_hint(),
            GeometryExteriorCoordsIter::MultiLineString(g) => g.size_hint(),
            GeometryExteriorCoordsIter::MultiPolygon(g) => g.size_hint(),
            GeometryExteriorCoordsIter::GeometryCollection(g) => g.size_hint(),
            GeometryExteriorCoordsIter::Rect(g) => g.size_hint(),
            GeometryExteriorCoordsIter::Triangle(g) => g.size_hint(),
        }
    }
}

impl<'a, T: CoordNum + Debug> fmt::Debug for GeometryExteriorCoordsIter<'a, T> {
    fn fmt(&self, fmt: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            GeometryExteriorCoordsIter::Point(i) => fmt.debug_tuple("Point").field(i).finish(),
            GeometryExteriorCoordsIter::Line(i) => fmt.debug_tuple("Line").field(i).finish(),
            GeometryExteriorCoordsIter::LineString(i) => {
                fmt.debug_tuple("LineString").field(i).finish()
            }
            GeometryExteriorCoordsIter::Polygon(i) => fmt.debug_tuple("Polygon").field(i).finish(),
            GeometryExteriorCoordsIter::MultiPoint(i) => {
                fmt.debug_tuple("MultiPoint").field(i).finish()
            }
            GeometryExteriorCoordsIter::MultiLineString(i) => {
                fmt.debug_tuple("MultiLineString").field(i).finish()
            }
            GeometryExteriorCoordsIter::MultiPolygon(i) => {
                fmt.debug_tuple("MultiPolygon").field(i).finish()
            }
            GeometryExteriorCoordsIter::GeometryCollection(_) => fmt
                .debug_tuple("GeometryCollection")
                .field(&String::from("..."))
                .finish(),
            GeometryExteriorCoordsIter::Rect(i) => fmt.debug_tuple("Rect").field(i).finish(),
            GeometryExteriorCoordsIter::Triangle(i) => {
                fmt.debug_tuple("Triangle").field(i).finish()
            }
        }
    }
}

#[cfg(test)]
mod test {
    use super::CoordsIter;
    use crate::{
        coord, line_string, point, polygon, Coord, Geometry, GeometryCollection, Line, LineString,
        MultiLineString, MultiPoint, MultiPolygon, Point, Polygon, Rect, Triangle,
    };

    #[test]
    fn test_point() {
        let (point, expected_coords) = create_point();

        let actual_coords = point.coords_iter().collect::<Vec<_>>();

        assert_eq!(expected_coords, actual_coords);
    }

    #[test]
    fn test_line() {
        let line = Line::new(coord! { x: 1., y: 2. }, coord! { x: 2., y: 3. });

        let coords = line.coords_iter().collect::<Vec<_>>();

        assert_eq!(
            vec![coord! { x: 1., y: 2. }, coord! { x: 2., y: 3. },],
            coords
        );
    }

    #[test]
    fn test_line_string() {
        let (line_string, expected_coords) = create_line_string();

        let actual_coords = line_string.coords_iter().collect::<Vec<_>>();

        assert_eq!(expected_coords, actual_coords);
    }

    #[test]
    fn test_polygon() {
        let (polygon, expected_coords) = create_polygon();

        let actual_coords = polygon.coords_iter().collect::<Vec<_>>();

        assert_eq!(expected_coords, actual_coords);
    }

    #[test]
    fn test_multi_point() {
        let mut expected_coords = vec![];
        let (point, mut coords) = create_point();
        expected_coords.append(&mut coords.clone());
        expected_coords.append(&mut coords);

        let actual_coords = MultiPoint::new(vec![point, point])
            .coords_iter()
            .collect::<Vec<_>>();

        assert_eq!(expected_coords, actual_coords);
    }

    #[test]
    fn test_multi_line_string() {
        let mut expected_coords = vec![];
        let (line_string, mut coords) = create_line_string();
        expected_coords.append(&mut coords.clone());
        expected_coords.append(&mut coords);

        let actual_coords = MultiLineString::new(vec![line_string.clone(), line_string])
            .coords_iter()
            .collect::<Vec<_>>();

        assert_eq!(expected_coords, actual_coords);
    }

    #[test]
    fn test_multi_polygon() {
        let mut expected_coords = vec![];
        let (polygon, mut coords) = create_polygon();
        expected_coords.append(&mut coords.clone());
        expected_coords.append(&mut coords);

        let actual_coords = MultiPolygon::new(vec![polygon.clone(), polygon])
            .coords_iter()
            .collect::<Vec<_>>();

        assert_eq!(expected_coords, actual_coords);
    }

    #[test]
    fn test_geometry() {
        let (line_string, expected_coords) = create_line_string();

        let actual_coords = Geometry::LineString(line_string)
            .coords_iter()
            .collect::<Vec<_>>();

        assert_eq!(expected_coords, actual_coords);
    }

    #[test]
    fn test_rect() {
        let (rect, expected_coords) = create_rect();

        let actual_coords = rect.coords_iter().collect::<Vec<_>>();

        assert_eq!(expected_coords, actual_coords);
    }

    #[test]
    fn test_triangle() {
        let (triangle, expected_coords) = create_triangle();

        let actual_coords = triangle.coords_iter().collect::<Vec<_>>();

        assert_eq!(expected_coords, actual_coords);
    }

    #[test]
    fn test_geometry_collection() {
        let mut expected_coords = vec![];
        let (line_string, mut coords) = create_line_string();
        expected_coords.append(&mut coords);
        let (polygon, mut coords) = create_polygon();
        expected_coords.append(&mut coords);

        let actual_coords = GeometryCollection::new_from(vec![
            Geometry::LineString(line_string),
            Geometry::Polygon(polygon),
        ])
        .coords_iter()
        .collect::<Vec<_>>();

        assert_eq!(expected_coords, actual_coords);
    }

    fn create_point() -> (Point, Vec<Coord>) {
        (point!(x: 1., y: 2.), vec![coord! { x: 1., y: 2. }])
    }

    fn create_triangle() -> (Triangle, Vec<Coord>) {
        (
            Triangle::new(
                coord! { x: 1., y: 2. },
                coord! { x: 3., y: 4. },
                coord! { x: 5., y: 6. },
            ),
            vec![
                coord! { x: 1., y: 2. },
                coord! { x: 3., y: 4. },
                coord! { x: 5., y: 6. },
            ],
        )
    }

    fn create_rect() -> (Rect, Vec<Coord>) {
        (
            Rect::new(coord! { x: 1., y: 2. }, coord! { x: 3., y: 4. }),
            vec![
                coord! { x: 1., y: 2. },
                coord! { x: 1., y: 4. },
                coord! { x: 3., y: 4. },
                coord! { x: 3., y: 2. },
            ],
        )
    }

    fn create_line_string() -> (LineString, Vec<Coord>) {
        (
            line_string![
                (x: 1., y: 2.),
                (x: 2., y: 3.),
            ],
            vec![coord! { x: 1., y: 2. }, coord! { x: 2., y: 3. }],
        )
    }

    fn create_polygon() -> (Polygon<f64>, Vec<Coord>) {
        (
            polygon!(
                exterior: [(x: 0., y: 0.), (x: 5., y: 10.), (x: 10., y: 0.), (x: 0., y: 0.)],
                interiors: [[(x: 1., y: 1.), (x: 9., y: 1.), (x: 5., y: 9.), (x: 1., y: 1.)]],
            ),
            vec![
                coord! { x: 0.0, y: 0.0 },
                coord! { x: 5.0, y: 10.0 },
                coord! { x: 10.0, y: 0.0 },
                coord! { x: 0.0, y: 0.0 },
                coord! { x: 1.0, y: 1.0 },
                coord! { x: 9.0, y: 1.0 },
                coord! { x: 5.0, y: 9.0 },
                coord! { x: 1.0, y: 1.0 },
            ],
        )
    }
}
