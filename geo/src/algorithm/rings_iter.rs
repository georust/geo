use crate::{
    line_string, CoordNum, Geometry, GeometryCollection, Line, LineString, MultiLineString,
    MultiPoint, MultiPolygon, Point, Polygon, Rect, Triangle,
};
use std::{borrow, iter, slice};

pub struct RingSet<'a, T: CoordNum + 'a> {
    pub exterior: borrow::Cow<'a, LineString<T>>,
    pub interiors: &'a [LineString<T>],
}

pub trait RingsIter<'a> {
    type Scalar: crate::CoordNum + 'a;
    type RingSetIter: Iterator<Item = RingSet<'a, Self::Scalar>>;

    fn rings(&'a self) -> Self::RingSetIter;
}

impl<'a, T: CoordNum + 'a> RingsIter<'a> for Point<T> {
    type Scalar = T;
    type RingSetIter = iter::Empty<RingSet<'a, T>>;

    fn rings(&'a self) -> Self::RingSetIter {
        iter::empty()
    }
}

impl<'a, T: CoordNum + 'a> RingsIter<'a> for Line<T> {
    type Scalar = T;
    type RingSetIter = iter::Empty<RingSet<'a, T>>;

    fn rings(&'a self) -> Self::RingSetIter {
        iter::empty()
    }
}

impl<'a, T: CoordNum + 'a> RingsIter<'a> for LineString<T> {
    type Scalar = T;
    type RingSetIter = iter::Empty<RingSet<'a, T>>;

    fn rings(&'a self) -> Self::RingSetIter {
        iter::empty()
    }
}

impl<'a, T: CoordNum + 'a> RingsIter<'a> for Polygon<T> {
    type Scalar = T;
    type RingSetIter = iter::Once<RingSet<'a, T>>;

    fn rings(&'a self) -> Self::RingSetIter {
        iter::once(RingSet {
            exterior: borrow::Cow::Borrowed(self.exterior()),
            interiors: self.interiors(),
        })
    }
}

impl<'a, T: CoordNum + 'a> RingsIter<'a> for MultiPoint<T> {
    type Scalar = T;
    type RingSetIter = iter::Empty<RingSet<'a, T>>;

    fn rings(&'a self) -> Self::RingSetIter {
        iter::empty()
    }
}

impl<'a, T: CoordNum + 'a> RingsIter<'a> for MultiLineString<T> {
    type Scalar = T;
    type RingSetIter = iter::Empty<RingSet<'a, T>>;

    fn rings(&'a self) -> Self::RingSetIter {
        iter::empty()
    }
}

impl<'a, T: CoordNum + 'a> RingsIter<'a> for MultiPolygon<T> {
    type Scalar = T;
    type RingSetIter = MultiPolygonRingsIter<'a, Self::Scalar>;

    fn rings(&'a self) -> Self::RingSetIter {
        MultiPolygonRingsIter(self.0.iter())
    }
}

#[doc(hidden)]
pub struct MultiPolygonRingsIter<'a, T: CoordNum + 'a>(slice::Iter<'a, Polygon<T>>);

impl<'a, T: CoordNum> Iterator for MultiPolygonRingsIter<'a, T> {
    type Item = RingSet<'a, T>;

    fn next(&mut self) -> Option<Self::Item> {
        self.0.next().map(|n| {
            n.rings().next().unwrap() // TODO
        })
    }
}

impl<'a, T: CoordNum + 'a> RingsIter<'a> for Rect<T> {
    type Scalar = T;
    type RingSetIter = iter::Once<RingSet<'a, T>>;

    fn rings(&'a self) -> Self::RingSetIter {
        iter::once(
            // TODO is this the right order?
            RingSet {
                exterior: borrow::Cow::Owned(line_string![
                    (x: self.min().x, y: self.min().y),
                    (x: self.min().x, y: self.max().y),
                    (x: self.max().x, y: self.max().y),
                    (x: self.max().x, y: self.min().y),
                    (x: self.min().x, y: self.min().y),
                ]),
                interiors: &[],
            },
        )
    }
}

impl<'a, T: CoordNum + 'a> RingsIter<'a> for Triangle<T> {
    type Scalar = T;
    type RingSetIter = iter::Once<RingSet<'a, T>>;

    fn rings(&'a self) -> Self::RingSetIter {
        iter::once(
            RingSet {
                exterior:
                    // TODO is this the right order?
                    borrow::Cow::Owned(
                        line_string![
                            (x: self.0.x, y: self.0.y),
                            (x: self.1.x, y: self.1.y),
                            (x: self.2.x, y: self.2.y),
                            (x: self.0.x, y: self.0.y),
                        ]
                    ),
                interiors: &[],
            }
        )
    }
}

impl<'a, T: CoordNum + 'a> RingsIter<'a> for GeometryCollection<T> {
    type Scalar = T;
    type RingSetIter = Box<dyn Iterator<Item = RingSet<'a, T>> + 'a>;

    fn rings(&'a self) -> Self::RingSetIter {
        Box::new(self.0.iter().flat_map(|g| g.rings()))
    }
}

impl<'a, T: CoordNum + 'a> RingsIter<'a> for Geometry<T> {
    type Scalar = T;
    type RingSetIter = GeometryRingsIter<'a, T>;

    fn rings(&'a self) -> Self::RingSetIter {
        match self {
            Geometry::Point(g) => GeometryRingsIter::Point(g.rings()),
            Geometry::Line(g) => GeometryRingsIter::Line(g.rings()),
            Geometry::LineString(g) => {
                GeometryRingsIter::LineString(g.rings())
            }
            Geometry::Polygon(g) => GeometryRingsIter::Polygon(g.rings()),
            Geometry::MultiPoint(g) => {
                GeometryRingsIter::MultiPoint(g.rings())
            }
            Geometry::MultiLineString(g) => {
                GeometryRingsIter::MultiLineString(g.rings())
            }
            Geometry::MultiPolygon(g) => {
                GeometryRingsIter::MultiPolygon(g.rings())
            }
            Geometry::GeometryCollection(g) => {
                GeometryRingsIter::GeometryCollection(g.rings())
            }
            Geometry::Rect(g) => GeometryRingsIter::Rect(g.rings()),
            Geometry::Triangle(g) => GeometryRingsIter::Triangle(g.rings()),
        }
    }

    /*
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
    } */
}

// Utility to transform Geometry into Iterator<LineString>
#[doc(hidden)]
pub enum GeometryRingsIter<'a, T: CoordNum + 'a> {
    Point(<Point<T> as RingsIter<'a>>::RingSetIter),
    Line(<Line<T> as RingsIter<'a>>::RingSetIter),
    LineString(<LineString<T> as RingsIter<'a>>::RingSetIter),
    Polygon(<Polygon<T> as RingsIter<'a>>::RingSetIter),
    MultiPoint(<MultiPoint<T> as RingsIter<'a>>::RingSetIter),
    MultiLineString(<MultiLineString<T> as RingsIter<'a>>::RingSetIter),
    MultiPolygon(<MultiPolygon<T> as RingsIter<'a>>::RingSetIter),
    GeometryCollection(<GeometryCollection<T> as RingsIter<'a>>::RingSetIter),
    Rect(<Rect<T> as RingsIter<'a>>::RingSetIter),
    Triangle(<Triangle<T> as RingsIter<'a>>::RingSetIter),
}

impl<'a, T: CoordNum> Iterator for GeometryRingsIter<'a, T> {
    type Item = RingSet<'a, T>;

    fn next(&mut self) -> Option<Self::Item> {
        match self {
            GeometryRingsIter::Point(g) => g.next(),
            GeometryRingsIter::Line(g) => g.next(),
            GeometryRingsIter::LineString(g) => g.next(),
            GeometryRingsIter::Polygon(g) => g.next(),
            GeometryRingsIter::MultiPoint(g) => g.next(),
            GeometryRingsIter::MultiLineString(g) => g.next(),
            GeometryRingsIter::MultiPolygon(g) => g.next(),
            GeometryRingsIter::GeometryCollection(g) => g.next(),
            GeometryRingsIter::Rect(g) => g.next(),
            GeometryRingsIter::Triangle(g) => g.next(),
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        match self {
            GeometryRingsIter::Point(g) => g.size_hint(),
            GeometryRingsIter::Line(g) => g.size_hint(),
            GeometryRingsIter::LineString(g) => g.size_hint(),
            GeometryRingsIter::Polygon(g) => g.size_hint(),
            GeometryRingsIter::MultiPoint(g) => g.size_hint(),
            GeometryRingsIter::MultiLineString(g) => g.size_hint(),
            GeometryRingsIter::MultiPolygon(g) => g.size_hint(),
            GeometryRingsIter::GeometryCollection(g) => g.size_hint(),
            GeometryRingsIter::Rect(g) => g.size_hint(),
            GeometryRingsIter::Triangle(g) => g.size_hint(),
        }
    }
}
