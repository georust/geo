use std::fmt::Debug;

use crate::{
    Coordinate, CoordinateType, Geometry, GeometryCollection, Line, LineString, MultiLineString,
    MultiPoint, MultiPolygon, Point, Polygon, Rect, Triangle,
};

use std::{fmt, iter, marker, slice};

type CoordinateChainOnce<T> = iter::Chain<iter::Once<Coordinate<T>>, iter::Once<Coordinate<T>>>;

/// Iterate over geometry coordinates.
pub trait CoordsIter<'a> {
    type Iter: Iterator<Item = Coordinate<Self::Scalar>>;
    type Scalar: CoordinateType;

    /// Iterate over all exterior and (if any) interior coordinates of a geometry.
    ///
    /// # Examples
    ///
    /// ```
    /// use geo::coords_iter::CoordsIter;
    ///
    /// let multi_point = geo::MultiPoint(vec![
    ///     geo::point!(x: -10., y: 0.),
    ///     geo::point!(x: 20., y: 20.),
    ///     geo::point!(x: 30., y: 40.),
    /// ]);
    ///
    /// let mut iter = multi_point.coords_iter();
    /// assert_eq!(Some(geo::Coordinate { x: -10., y: 0. }), iter.next());
    /// assert_eq!(Some(geo::Coordinate { x: 20., y: 20. }), iter.next());
    /// assert_eq!(Some(geo::Coordinate { x: 30., y: 40. }), iter.next());
    /// assert_eq!(None, iter.next());
    /// ```
    fn coords_iter(&'a self) -> Self::Iter;

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
    fn coords_count(&'a self) -> usize;
}

// ┌──────────────────────────┐
// │ Implementation for Point │
// └──────────────────────────┘

impl<'a, T: CoordinateType> CoordsIter<'a> for Point<T> {
    type Iter = iter::Once<Coordinate<T>>;
    type Scalar = T;

    fn coords_iter(&'a self) -> Self::Iter {
        iter::once(self.0)
    }

    /// Return the number of coordinates in the `Point`.
    fn coords_count(&'a self) -> usize {
        1
    }
}

// ┌─────────────────────────┐
// │ Implementation for Line │
// └─────────────────────────┘

impl<'a, T: CoordinateType> CoordsIter<'a> for Line<T> {
    type Iter = iter::Chain<iter::Once<Coordinate<T>>, iter::Once<Coordinate<T>>>;
    type Scalar = T;

    fn coords_iter(&'a self) -> Self::Iter {
        iter::once(self.start).chain(iter::once(self.end))
    }

    /// Return the number of coordinates in the `Line`.
    fn coords_count(&'a self) -> usize {
        2
    }
}

// ┌───────────────────────────────┐
// │ Implementation for LineString │
// └───────────────────────────────┘

impl<'a, T: CoordinateType + 'a> CoordsIter<'a> for LineString<T> {
    type Iter = iter::Copied<slice::Iter<'a, Coordinate<T>>>;
    type Scalar = T;

    fn coords_iter(&'a self) -> Self::Iter {
        self.0.iter().copied()
    }

    /// Return the number of coordinates in the `LineString`.
    fn coords_count(&'a self) -> usize {
        self.0.len()
    }
}

// ┌────────────────────────────┐
// │ Implementation for Polygon │
// └────────────────────────────┘
type PolygonChainIter<'a, T> = iter::Chain<
    <LineString<T> as CoordsIter<'a>>::Iter,
    iter::Flatten<MapCoordsIter<'a, T, slice::Iter<'a, LineString<T>>, LineString<T>>>,
>;

impl<'a, T: CoordinateType + 'a> CoordsIter<'a> for Polygon<T> {
    type Iter = PolygonChainIter<'a, T>;
    type Scalar = T;

    fn coords_iter(&'a self) -> Self::Iter {
        self.exterior()
            .coords_iter()
            .chain(MapCoordsIter(self.interiors().iter(), marker::PhantomData).flatten())
    }

    /// Return the number of coordinates in the `Polygon`.
    fn coords_count(&'a self) -> usize {
        self.exterior().coords_count()
            + self
                .interiors()
                .iter()
                .map(|i| i.coords_count())
                .sum::<usize>()
    }
}

// ┌───────────────────────────────┐
// │ Implementation for MultiPoint │
// └───────────────────────────────┘

impl<'a, T: CoordinateType + 'a> CoordsIter<'a> for MultiPoint<T> {
    type Iter = iter::Flatten<MapCoordsIter<'a, T, slice::Iter<'a, Point<T>>, Point<T>>>;
    type Scalar = T;

    fn coords_iter(&'a self) -> Self::Iter {
        MapCoordsIter(self.0.iter(), marker::PhantomData).flatten()
    }

    /// Return the number of coordinates in the `MultiPoint`.
    fn coords_count(&'a self) -> usize {
        self.0.len()
    }
}

// ┌────────────────────────────────────┐
// │ Implementation for MultiLineString │
// └────────────────────────────────────┘

impl<'a, T: CoordinateType + 'a> CoordsIter<'a> for MultiLineString<T> {
    type Iter = iter::Flatten<MapCoordsIter<'a, T, slice::Iter<'a, LineString<T>>, LineString<T>>>;
    type Scalar = T;

    fn coords_iter(&'a self) -> Self::Iter {
        MapCoordsIter(self.0.iter(), marker::PhantomData).flatten()
    }

    /// Return the number of coordinates in the `MultiLineString`.
    fn coords_count(&'a self) -> usize {
        self.0
            .iter()
            .map(|line_string| line_string.coords_count())
            .sum()
    }
}

// ┌─────────────────────────────────┐
// │ Implementation for MultiPolygon │
// └─────────────────────────────────┘

impl<'a, T: CoordinateType + 'a> CoordsIter<'a> for MultiPolygon<T> {
    type Iter = iter::Flatten<MapCoordsIter<'a, T, slice::Iter<'a, Polygon<T>>, Polygon<T>>>;
    type Scalar = T;

    fn coords_iter(&'a self) -> Self::Iter {
        MapCoordsIter(self.0.iter(), marker::PhantomData).flatten()
    }

    /// Return the number of coordinates in the `MultiPolygon`.
    fn coords_count(&'a self) -> usize {
        self.0.iter().map(|polygon| polygon.coords_count()).sum()
    }
}

// ┌───────────────────────────────────────┐
// │ Implementation for GeometryCollection │
// └───────────────────────────────────────┘

impl<'a, T: CoordinateType + 'a> CoordsIter<'a> for GeometryCollection<T> {
    type Iter = Box<dyn Iterator<Item = Coordinate<T>> + 'a>;
    type Scalar = T;

    fn coords_iter(&'a self) -> Self::Iter {
        Box::new(self.0.iter().flat_map(|geometry| geometry.coords_iter()))
    }

    /// Return the number of coordinates in the `GeometryCollection`.
    fn coords_count(&'a self) -> usize {
        self.0.iter().map(|geometry| geometry.coords_count()).sum()
    }
}

// ┌─────────────────────────┐
// │ Implementation for Rect │
// └─────────────────────────┘

type RectChainIter<T> = iter::Chain<
    iter::Chain<CoordinateChainOnce<T>, iter::Once<Coordinate<T>>>,
    iter::Once<Coordinate<T>>,
>;

impl<'a, T: CoordinateType + 'a> CoordsIter<'a> for Rect<T> {
    type Iter = RectChainIter<T>;
    type Scalar = T;

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

    /// Return the number of coordinates in the `Rect`.
    ///
    /// Note: Although a `Rect` is represented by two coordinates, it is
    /// spatially represented by four, so this method returns `4`.
    fn coords_count(&'a self) -> usize {
        4
    }
}

// ┌─────────────────────────────┐
// │ Implementation for Triangle │
// └─────────────────────────────┘

impl<'a, T: CoordinateType + 'a> CoordsIter<'a> for Triangle<T> {
    type Iter = iter::Chain<CoordinateChainOnce<T>, iter::Once<Coordinate<T>>>;
    type Scalar = T;

    fn coords_iter(&'a self) -> Self::Iter {
        iter::once(self.0)
            .chain(iter::once(self.1))
            .chain(iter::once(self.2))
    }

    /// Return the number of coordinates in the `Triangle`.
    fn coords_count(&'a self) -> usize {
        3
    }
}

// ┌─────────────────────────────┐
// │ Implementation for Geometry │
// └─────────────────────────────┘

impl<'a, T: CoordinateType + 'a> CoordsIter<'a> for Geometry<T> {
    type Iter = GeometryCoordsIter<'a, T>;
    type Scalar = T;

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

    /// Return the number of coordinates in the `Geometry`.
    fn coords_count(&'a self) -> usize {
        match self {
            Geometry::Point(g) => g.coords_count(),
            Geometry::Line(g) => g.coords_count(),
            Geometry::LineString(g) => g.coords_count(),
            Geometry::Polygon(g) => g.coords_count(),
            Geometry::MultiPoint(g) => g.coords_count(),
            Geometry::MultiLineString(g) => g.coords_count(),
            Geometry::MultiPolygon(g) => g.coords_count(),
            Geometry::GeometryCollection(g) => g.coords_count(),
            Geometry::Rect(g) => g.coords_count(),
            Geometry::Triangle(g) => g.coords_count(),
        }
    }
}

// ┌───────────┐
// │ Utilities │
// └───────────┘

// Utility to transform Iterator<CoordsIter> into Iterator<Iterator<Coordinate>>
#[doc(hidden)]
#[derive(Debug)]
pub struct MapCoordsIter<
    'a,
    T: 'a + CoordinateType,
    Iter1: Iterator<Item = &'a Iter2>,
    Iter2: 'a + CoordsIter<'a>,
>(Iter1, marker::PhantomData<T>);

impl<'a, T: 'a + CoordinateType, Iter1: Iterator<Item = &'a Iter2>, Iter2: CoordsIter<'a>>
    Iterator for MapCoordsIter<'a, T, Iter1, Iter2>
{
    type Item = Iter2::Iter;

    fn next(&mut self) -> Option<Self::Item> {
        self.0.next().map(|g| g.coords_iter())
    }
}

// Utility to transform Geometry into Iterator<Coordinate>
#[doc(hidden)]
pub enum GeometryCoordsIter<'a, T: CoordinateType + 'a> {
    Point(<Point<T> as CoordsIter<'a>>::Iter),
    Line(<Line<T> as CoordsIter<'a>>::Iter),
    LineString(<LineString<T> as CoordsIter<'a>>::Iter),
    Polygon(<Polygon<T> as CoordsIter<'a>>::Iter),
    MultiPoint(<MultiPoint<T> as CoordsIter<'a>>::Iter),
    MultiLineString(<MultiLineString<T> as CoordsIter<'a>>::Iter),
    MultiPolygon(<MultiPolygon<T> as CoordsIter<'a>>::Iter),
    GeometryCollection(<GeometryCollection<T> as CoordsIter<'a>>::Iter),
    Rect(<Rect<T> as CoordsIter<'a>>::Iter),
    Triangle(<Triangle<T> as CoordsIter<'a>>::Iter),
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

impl<'a, T: CoordinateType + Debug> fmt::Debug for GeometryCoordsIter<'a, T> {
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

#[cfg(test)]
mod test {
    use super::CoordsIter;
    use crate::{
        line_string, point, polygon, Coordinate, Geometry, GeometryCollection, Line, LineString,
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
        let line = Line::new(Coordinate { x: 1., y: 2. }, Coordinate { x: 2., y: 3. });

        let coords = line.coords_iter().collect::<Vec<_>>();

        assert_eq!(
            vec![Coordinate { x: 1., y: 2. }, Coordinate { x: 2., y: 3. },],
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

        let actual_coords = MultiPoint(vec![point.clone(), point.clone()])
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

        let actual_coords = MultiLineString(vec![line_string.clone(), line_string.clone()])
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

        let actual_coords = MultiPolygon(vec![polygon.clone(), polygon.clone()])
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

        let actual_coords = GeometryCollection(vec![
            Geometry::LineString(line_string),
            Geometry::Polygon(polygon),
        ])
        .coords_iter()
        .collect::<Vec<_>>();

        assert_eq!(expected_coords, actual_coords);
    }

    fn create_point() -> (Point<f64>, Vec<Coordinate<f64>>) {
        (point!(x: 1., y: 2.), vec![Coordinate { x: 1., y: 2. }])
    }

    fn create_triangle() -> (Triangle<f64>, Vec<Coordinate<f64>>) {
        (
            Triangle(
                Coordinate { x: 1., y: 2. },
                Coordinate { x: 3., y: 4. },
                Coordinate { x: 5., y: 6. },
            ),
            vec![
                Coordinate { x: 1., y: 2. },
                Coordinate { x: 3., y: 4. },
                Coordinate { x: 5., y: 6. },
            ],
        )
    }

    fn create_rect() -> (Rect<f64>, Vec<Coordinate<f64>>) {
        (
            Rect::new(Coordinate { x: 1., y: 2. }, Coordinate { x: 3., y: 4. }),
            vec![
                Coordinate { x: 1., y: 2. },
                Coordinate { x: 1., y: 4. },
                Coordinate { x: 3., y: 4. },
                Coordinate { x: 3., y: 2. },
            ],
        )
    }

    fn create_line_string() -> (LineString<f64>, Vec<Coordinate<f64>>) {
        (
            line_string![
                (x: 1., y: 2.),
                (x: 2., y: 3.),
            ],
            vec![Coordinate { x: 1., y: 2. }, Coordinate { x: 2., y: 3. }],
        )
    }

    fn create_polygon() -> (Polygon<f64>, Vec<Coordinate<f64>>) {
        (
            polygon!(
                exterior: [(x: 0., y: 0.), (x: 5., y: 10.), (x: 10., y: 0.), (x: 0., y: 0.)],
                interiors: [[(x: 1., y: 1.), (x: 9., y: 1.), (x: 5., y: 9.), (x: 1., y: 1.)]],
            ),
            vec![
                Coordinate { x: 0.0, y: 0.0 },
                Coordinate { x: 5.0, y: 10.0 },
                Coordinate { x: 10.0, y: 0.0 },
                Coordinate { x: 0.0, y: 0.0 },
                Coordinate { x: 1.0, y: 1.0 },
                Coordinate { x: 9.0, y: 1.0 },
                Coordinate { x: 5.0, y: 9.0 },
                Coordinate { x: 1.0, y: 1.0 }
            ],
        )
    }
}
