use crate::{
    CoordNum, Geometry, GeometryCollection, Line, LineString, MultiLineString,
    MultiPoint, MultiPolygon, Point, Polygon, Rect, Triangle, line_string
};
use std::{borrow, iter, slice};

type Ring<'a, T> = borrow::Cow<'a, LineString<T>>;

pub trait RingsIter<'a> {
    type Scalar: crate::CoordNum + 'a;
    type ExteriorRingsIter: Iterator<Item = Ring<'a, Self::Scalar>>;
    type InteriorRingsIter: Iterator<Item = Ring<'a, Self::Scalar>>;

    fn exterior_rings_iter(&'a self) -> Self::ExteriorRingsIter;
    fn interior_rings_iter(&'a self) -> Self::InteriorRingsIter;
}

impl<'a, T: CoordNum + 'a> RingsIter<'a> for Point<T> {
    type Scalar = T;
    type ExteriorRingsIter = iter::Empty<Ring<'a, T>>;
    type InteriorRingsIter = iter::Empty<Ring<'a, T>>;

    fn exterior_rings_iter(&'a self) -> Self::ExteriorRingsIter {
        iter::empty()
    }

    fn interior_rings_iter(&'a self) -> Self::InteriorRingsIter {
        iter::empty()
    }
}

impl<'a, T: CoordNum + 'a> RingsIter<'a> for Line<T> {
    type Scalar = T;
    type ExteriorRingsIter = iter::Empty<Ring<'a, T>>;
    type InteriorRingsIter = iter::Empty<Ring<'a, T>>;

    fn exterior_rings_iter(&'a self) -> Self::ExteriorRingsIter {
        iter::empty()
    }

    fn interior_rings_iter(&'a self) -> Self::InteriorRingsIter {
        iter::empty()
    }
}

impl<'a, T: CoordNum + 'a> RingsIter<'a> for LineString<T> {
    type Scalar = T;
    type ExteriorRingsIter = iter::Empty<Ring<'a, T>>;
    type InteriorRingsIter = iter::Empty<Ring<'a, T>>;

    fn exterior_rings_iter(&'a self) -> Self::ExteriorRingsIter {
        iter::empty()
    }

    fn interior_rings_iter(&'a self) -> Self::InteriorRingsIter {
        iter::empty()
    }
}

impl<'a, T: CoordNum + 'a> RingsIter<'a> for Polygon<T> {
    type Scalar = T;
    type ExteriorRingsIter = iter::Once<Ring<'a, T>>;
    type InteriorRingsIter = PolygonInteriorRingsIter<'a, T>;

    fn exterior_rings_iter(&'a self) -> Self::ExteriorRingsIter {
        iter::once(borrow::Cow::Borrowed(self.exterior()))
    }

    fn interior_rings_iter(&'a self) -> Self::InteriorRingsIter {
        PolygonInteriorRingsIter(self.interiors().iter())
    }
}

#[doc(hidden)]
pub struct PolygonInteriorRingsIter<'a, T: CoordNum + 'a>(slice::Iter<'a, LineString<T>>);

impl<'a, T: CoordNum + 'a> Iterator for PolygonInteriorRingsIter<'a, T> {
    type Item = Ring<'a, T>;

    fn next(&mut self) -> Option<Self::Item> {
        self.0.next().map(borrow::Cow::Borrowed)
    }
}

impl<'a, T: CoordNum + 'a> RingsIter<'a> for MultiPoint<T> {
    type Scalar = T;
    type InteriorRingsIter = iter::Empty<Ring<'a, T>>;
    type ExteriorRingsIter = iter::Empty<Ring<'a, T>>;

    fn exterior_rings_iter(&'a self) -> Self::ExteriorRingsIter {
        iter::empty()
    }

    fn interior_rings_iter(&'a self) -> Self::InteriorRingsIter {
        iter::empty()
    }
}

impl<'a, T: CoordNum + 'a> RingsIter<'a> for MultiLineString<T> {
    type Scalar = T;
    type InteriorRingsIter = iter::Empty<Ring<'a, T>>;
    type ExteriorRingsIter = iter::Empty<Ring<'a, T>>;

    fn exterior_rings_iter(&'a self) -> Self::ExteriorRingsIter {
        iter::empty()
    }

    fn interior_rings_iter(&'a self) -> Self::InteriorRingsIter {
        iter::empty()
    }
}

impl<'a, T: CoordNum + 'a> RingsIter<'a> for MultiPolygon<T> {
    type Scalar = T;
    type ExteriorRingsIter = iter::Empty<Ring<'a, T>>;
    type InteriorRingsIter = iter::Empty<Ring<'a, T>>;

    fn exterior_rings_iter(&'a self) -> Self::ExteriorRingsIter {
        iter::empty()
    }

    fn interior_rings_iter(&'a self) -> Self::InteriorRingsIter {
        iter::empty()
    }
}

impl<'a, T: CoordNum + 'a> RingsIter<'a> for Rect<T> {
    type Scalar = T;
    type ExteriorRingsIter = iter::Once<Ring<'a, T>>;
    type InteriorRingsIter = iter::Empty<Ring<'a, T>>;

    fn exterior_rings_iter(&'a self) -> Self::ExteriorRingsIter {
        iter::once(
            // TODO is this the right order?
            borrow::Cow::Owned(
                line_string![
                    (x: self.min().x, y: self.min().y),
                    (x: self.min().x, y: self.max().y),
                    (x: self.max().x, y: self.max().y),
                    (x: self.max().x, y: self.min().y),
                    (x: self.min().x, y: self.min().y),
                ]
            )
        )
    }

    fn interior_rings_iter(&'a self) -> Self::InteriorRingsIter {
        iter::empty()
    }
}

impl<'a, T: CoordNum + 'a> RingsIter<'a> for Triangle<T> {
    type Scalar = T;
    type ExteriorRingsIter = iter::Once<Ring<'a, T>>;
    type InteriorRingsIter = iter::Empty<Ring<'a, T>>;

    fn exterior_rings_iter(&'a self) -> Self::ExteriorRingsIter {
        iter::once(
            // TODO is this the right order?
            borrow::Cow::Owned(
                line_string![
                    (x: self.0.x, y: self.0.y),
                    (x: self.1.x, y: self.1.y),
                    (x: self.2.x, y: self.2.y),
                    (x: self.0.x, y: self.0.y),
                ]
            )
        )
    }

    fn interior_rings_iter(&'a self) -> Self::InteriorRingsIter {
        iter::empty()
    }
}

impl<'a, T: CoordNum + 'a> RingsIter<'a> for GeometryCollection<T> {
    type Scalar = T;
    type InteriorRingsIter = Box<dyn Iterator<Item = Ring<'a, T>> + 'a>;
    type ExteriorRingsIter = Box<dyn Iterator<Item = Ring<'a, T>> + 'a>;

    fn exterior_rings_iter(&'a self) -> Self::ExteriorRingsIter {
        Box::new(self.0.iter().flat_map(|g| g.exterior_rings_iter()))
    }

    fn interior_rings_iter(&'a self) -> Self::InteriorRingsIter {
        Box::new(self.0.iter().flat_map(|g| g.interior_rings_iter()))
    }
}

impl<'a, T: CoordNum + 'a> RingsIter<'a> for Geometry<T> {
    type Scalar = T;
    type ExteriorRingsIter = GeometryExteriorRingsIter<'a, T>;
    type InteriorRingsIter = GeometryInteriorRingsIter<'a, T>;

    fn exterior_rings_iter(&'a self) -> Self::ExteriorRingsIter {
        match self {
            Geometry::Point(g) => GeometryExteriorRingsIter::Point(g.exterior_rings_iter()),
            Geometry::Line(g) => GeometryExteriorRingsIter::Line(g.exterior_rings_iter()),
            Geometry::LineString(g) => {
                GeometryExteriorRingsIter::LineString(g.exterior_rings_iter())
            }
            Geometry::Polygon(g) => GeometryExteriorRingsIter::Polygon(g.exterior_rings_iter()),
            Geometry::MultiPoint(g) => {
                GeometryExteriorRingsIter::MultiPoint(g.exterior_rings_iter())
            }
            Geometry::MultiLineString(g) => {
                GeometryExteriorRingsIter::MultiLineString(g.exterior_rings_iter())
            }
            Geometry::MultiPolygon(g) => {
                GeometryExteriorRingsIter::MultiPolygon(g.exterior_rings_iter())
            }
            Geometry::GeometryCollection(g) => {
                GeometryExteriorRingsIter::GeometryCollection(g.exterior_rings_iter())
            }
            Geometry::Rect(g) => GeometryExteriorRingsIter::Rect(g.exterior_rings_iter()),
            Geometry::Triangle(g) => GeometryExteriorRingsIter::Triangle(g.exterior_rings_iter()),
        }
    }

    fn interior_rings_iter(&'a self) -> Self::InteriorRingsIter {
        match self {
            Geometry::Point(g) => GeometryInteriorRingsIter::Point(g.interior_rings_iter()),
            Geometry::Line(g) => GeometryInteriorRingsIter::Line(g.interior_rings_iter()),
            Geometry::LineString(g) => {
                GeometryInteriorRingsIter::LineString(g.interior_rings_iter())
            }
            Geometry::Polygon(g) => GeometryInteriorRingsIter::Polygon(g.interior_rings_iter()),
            Geometry::MultiPoint(g) => {
                GeometryInteriorRingsIter::MultiPoint(g.interior_rings_iter())
            }
            Geometry::MultiLineString(g) => {
                GeometryInteriorRingsIter::MultiLineString(g.interior_rings_iter())
            }
            Geometry::MultiPolygon(g) => {
                GeometryInteriorRingsIter::MultiPolygon(g.interior_rings_iter())
            }
            Geometry::GeometryCollection(g) => {
                GeometryInteriorRingsIter::GeometryCollection(g.interior_rings_iter())
            }
            Geometry::Rect(g) => GeometryInteriorRingsIter::Rect(g.interior_rings_iter()),
            Geometry::Triangle(g) => GeometryInteriorRingsIter::Triangle(g.interior_rings_iter()),
        }
    }
}

// Utility to transform Geometry into Iterator<LineString>
#[doc(hidden)]
pub enum GeometryExteriorRingsIter<'a, T: CoordNum + 'a> {
    Point(<Point<T> as RingsIter<'a>>::ExteriorRingsIter),
    Line(<Line<T> as RingsIter<'a>>::ExteriorRingsIter),
    LineString(<LineString<T> as RingsIter<'a>>::ExteriorRingsIter),
    Polygon(<Polygon<T> as RingsIter<'a>>::ExteriorRingsIter),
    MultiPoint(<MultiPoint<T> as RingsIter<'a>>::ExteriorRingsIter),
    MultiLineString(<MultiLineString<T> as RingsIter<'a>>::ExteriorRingsIter),
    MultiPolygon(<MultiPolygon<T> as RingsIter<'a>>::ExteriorRingsIter),
    GeometryCollection(<GeometryCollection<T> as RingsIter<'a>>::ExteriorRingsIter),
    Rect(<Rect<T> as RingsIter<'a>>::ExteriorRingsIter),
    Triangle(<Triangle<T> as RingsIter<'a>>::ExteriorRingsIter),
}

impl<'a, T: CoordNum> Iterator for GeometryExteriorRingsIter<'a, T> {
    type Item = Ring<'a, T>;

    fn next(&mut self) -> Option<Self::Item> {
        match self {
            GeometryExteriorRingsIter::Point(g) => g.next(),
            GeometryExteriorRingsIter::Line(g) => g.next(),
            GeometryExteriorRingsIter::LineString(g) => g.next(),
            GeometryExteriorRingsIter::Polygon(g) => g.next(),
            GeometryExteriorRingsIter::MultiPoint(g) => g.next(),
            GeometryExteriorRingsIter::MultiLineString(g) => g.next(),
            GeometryExteriorRingsIter::MultiPolygon(g) => g.next(),
            GeometryExteriorRingsIter::GeometryCollection(g) => g.next(),
            GeometryExteriorRingsIter::Rect(g) => g.next(),
            GeometryExteriorRingsIter::Triangle(g) => g.next(),
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        match self {
            GeometryExteriorRingsIter::Point(g) => g.size_hint(),
            GeometryExteriorRingsIter::Line(g) => g.size_hint(),
            GeometryExteriorRingsIter::LineString(g) => g.size_hint(),
            GeometryExteriorRingsIter::Polygon(g) => g.size_hint(),
            GeometryExteriorRingsIter::MultiPoint(g) => g.size_hint(),
            GeometryExteriorRingsIter::MultiLineString(g) => g.size_hint(),
            GeometryExteriorRingsIter::MultiPolygon(g) => g.size_hint(),
            GeometryExteriorRingsIter::GeometryCollection(g) => g.size_hint(),
            GeometryExteriorRingsIter::Rect(g) => g.size_hint(),
            GeometryExteriorRingsIter::Triangle(g) => g.size_hint(),
        }
    }
}

// Utility to transform Geometry into Iterator<LineString>
#[doc(hidden)]
pub enum GeometryInteriorRingsIter<'a, T: CoordNum + 'a> {
    Point(<Point<T> as RingsIter<'a>>::InteriorRingsIter),
    Line(<Line<T> as RingsIter<'a>>::InteriorRingsIter),
    LineString(<LineString<T> as RingsIter<'a>>::InteriorRingsIter),
    Polygon(<Polygon<T> as RingsIter<'a>>::InteriorRingsIter),
    MultiPoint(<MultiPoint<T> as RingsIter<'a>>::InteriorRingsIter),
    MultiLineString(<MultiLineString<T> as RingsIter<'a>>::InteriorRingsIter),
    MultiPolygon(<MultiPolygon<T> as RingsIter<'a>>::InteriorRingsIter),
    GeometryCollection(<GeometryCollection<T> as RingsIter<'a>>::InteriorRingsIter),
    Rect(<Rect<T> as RingsIter<'a>>::InteriorRingsIter),
    Triangle(<Triangle<T> as RingsIter<'a>>::InteriorRingsIter),
}

impl<'a, T: CoordNum> Iterator for GeometryInteriorRingsIter<'a, T> {
    type Item = Ring<'a, T>;

    fn next(&mut self) -> Option<Self::Item> {
        match self {
            GeometryInteriorRingsIter::Point(g) => g.next(),
            GeometryInteriorRingsIter::Line(g) => g.next(),
            GeometryInteriorRingsIter::LineString(g) => g.next(),
            GeometryInteriorRingsIter::Polygon(g) => g.next(),
            GeometryInteriorRingsIter::MultiPoint(g) => g.next(),
            GeometryInteriorRingsIter::MultiLineString(g) => g.next(),
            GeometryInteriorRingsIter::MultiPolygon(g) => g.next(),
            GeometryInteriorRingsIter::GeometryCollection(g) => g.next(),
            GeometryInteriorRingsIter::Rect(g) => g.next(),
            GeometryInteriorRingsIter::Triangle(g) => g.next(),
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        match self {
            GeometryInteriorRingsIter::Point(g) => g.size_hint(),
            GeometryInteriorRingsIter::Line(g) => g.size_hint(),
            GeometryInteriorRingsIter::LineString(g) => g.size_hint(),
            GeometryInteriorRingsIter::Polygon(g) => g.size_hint(),
            GeometryInteriorRingsIter::MultiPoint(g) => g.size_hint(),
            GeometryInteriorRingsIter::MultiLineString(g) => g.size_hint(),
            GeometryInteriorRingsIter::MultiPolygon(g) => g.size_hint(),
            GeometryInteriorRingsIter::GeometryCollection(g) => g.size_hint(),
            GeometryInteriorRingsIter::Rect(g) => g.size_hint(),
            GeometryInteriorRingsIter::Triangle(g) => g.size_hint(),
        }
    }
}
