use crate::{Geometry, CoordinateType, Line, LineString, MultiPolygon, Polygon, Rect, Triangle, GeometryRef, Point};
use num_traits::Float;
use std::borrow::Cow;

use crate::algorithm::winding_order::twice_signed_ring_area;

/// Signed planar area of a geometry.
///
/// # Examples
///
/// ```
/// use geo::polygon;
/// use geo::algorithm::area::Area;
///
/// let mut polygon = polygon![
///     (x: 0., y: 0.),
///     (x: 5., y: 0.),
///     (x: 5., y: 6.),
///     (x: 0., y: 6.),
///     (x: 0., y: 0.),
/// ];
///
/// assert_eq!(polygon.area(), 30.);
///
/// polygon.exterior_mut(|line_string| {
///     line_string.0.reverse();
/// });
///
/// assert_eq!(polygon.area(), -30.);
/// ```
pub trait Area<T>
where
    T: CoordinateType,
{
    fn area(&self) -> T;
}

// Calculation of simple (no interior holes) Polygon area
pub(crate) fn get_linestring_area<T>(linestring: &LineString<T>) -> T
where
    T: Float,
{
    twice_signed_ring_area(linestring) / (T::one() + T::one())
}

// impl<T> Area<T> for Line<T>
// where
//     T: CoordinateType,
// {
//     fn area(&self) -> T {
//         T::zero()
//     }
// }

// impl<T> Area<T> for Polygon<T>
// where
//     T: Float,
// {
//     fn area(&self) -> T {
//         self.interiors()
//             .iter()
//             .fold(get_linestring_area(self.exterior()), |total, next| {
//                 total - get_linestring_area(next)
//             })
//     }
// }

// impl<T> Area<T> for MultiPolygon<T>
// where
//     T: Float,
// {
//     fn area(&self) -> T {
//         self.0
//             .iter()
//             .fold(T::zero(), |total, next| total + next.area())
//     }
// }

// impl<T> Area<T> for Rect<T>
// where
//     T: CoordinateType,
// {
//     fn area(&self) -> T {
//         self.width() * self.height()
//     }
// }

// impl<T> Area<T> for Triangle<T>
// where
//     T: Float,
// {
//     fn area(&self) -> T {
//         self.to_lines()
//             .iter()
//             .fold(T::zero(), |total, line| total + line.determinant())
//             / (T::one() + T::one())
//     }
// }

///////////////////////////////////////////////

// impl<I, T> Area<T> for &I
// where
//     I: Into<Geometry<T>>,
//     T: CoordinateType,
// {
//     fn area(&self) -> T {
//         let geometry: Geometry<T> = self.into();
//         match geometry {
//             Geometry::Point(g) => unimplemented!(),
//             Geometry::Line(g) => unimplemented!(),
//             Geometry::LineString(g) => unimplemented!(),
//             Geometry::Polygon(g) => unimplemented!(),
//             Geometry::MultiPoint(g) => unimplemented!(),
//             Geometry::MultiLineString(g) => unimplemented!(),
//             Geometry::MultiPolygon(g) => unimplemented!(),
//             Geometry::GeometryCollection(g) => unimplemented!(),
//             Geometry::Rect(g) => unimplemented!(),
//             Geometry::Triangle(g) => unimplemented!(),
//         }
//     }
// }

// impl<I, J, T> Area<T> for I
// where
//     I: ToOwned<Owned = J>,
//     J: Into<Geometry<T>> + std::borrow::Borrow<I>,
//     T: CoordinateType,
// {
//     fn area(&self) -> T {
//         let owned: J = self.to_owned().to_owned();
//         let geometry: Geometry<T> = owned.into();
//         match geometry {
//             Geometry::Point(g) => unimplemented!(),
//             Geometry::Line(g) => unimplemented!(),
//             Geometry::LineString(g) => unimplemented!(),
//             Geometry::Polygon(g) => unimplemented!(),
//             Geometry::MultiPoint(g) => unimplemented!(),
//             Geometry::MultiLineString(g) => unimplemented!(),
//             Geometry::MultiPolygon(g) => unimplemented!(),
//             Geometry::GeometryCollection(g) => unimplemented!(),
//             Geometry::Rect(g) => unimplemented!(),
//             Geometry::Triangle(g) => unimplemented!(),
//         }
//     }
// }

impl<I , T> Area<T> for I
where
    for<'a> &'a I: Into<Cow<'a, Geometry<T>>>,
    T: CoordinateType,
{
    fn area(&self) -> T {
        let geometry_ref: Cow<Geometry<T>> = self.into();
        match *geometry_ref {
            Geometry::Point(g) => unimplemented!(),
            Geometry::Line(g) => unimplemented!(),
            Geometry::LineString(g) => unimplemented!(),
            Geometry::Polygon(g) => unimplemented!(),
            Geometry::MultiPoint(g) => unimplemented!(),
            Geometry::MultiLineString(g) => unimplemented!(),
            Geometry::MultiPolygon(g) => unimplemented!(),
            Geometry::GeometryCollection(g) => unimplemented!(),
            Geometry::Rect(g) => unimplemented!(),
            Geometry::Triangle(g) => unimplemented!(),
        }
    }
}

///////////////////////////////////////////////

struct NewPoint<T: Float>(Point<T>);

impl<'a, T: Float> Into<Cow<'a, Geometry<T>>> for &'_ NewPoint<T> {
    fn into(self) -> Cow<'a, Geometry<T>> {
        Cow::Owned(Geometry::Point(self.0))
    }
}

// ///////////////////////////////////////////////

fn foo() {
    let n = NewPoint(geo_types::point!(x: 1.0, y: 1.0));
    let a = n.area();
}

///////////////////////////////////////////////

#[cfg(test)]
mod test {
    use crate::algorithm::area::Area;
    use crate::{line_string, polygon, Coordinate, Line, MultiPolygon, Polygon, Rect, Triangle};

    // Area of the polygon
    #[test]
    fn area_empty_polygon_test() {
        let poly: Polygon<f32> = polygon![];
        assert_relative_eq!(poly.area(), 0.);
    }

    #[test]
    fn area_one_point_polygon_test() {
        let poly = polygon![(x: 1., y: 0.)];
        assert_relative_eq!(poly.area(), 0.);
    }
    #[test]
    fn area_polygon_test() {
        let polygon = polygon![
            (x: 0., y: 0.),
            (x: 5., y: 0.),
            (x: 5., y: 6.),
            (x: 0., y: 6.),
            (x: 0., y: 0.)
        ];
        assert_relative_eq!(polygon.area(), 30.);
    }
    #[test]
    fn rectangle_test() {
        let rect1: Rect<f32> =
            Rect::new(Coordinate { x: 10., y: 30. }, Coordinate { x: 20., y: 40. });
        assert_relative_eq!(rect1.area(), 100.);

        let rect2: Rect<i32> = Rect::new(Coordinate { x: 10, y: 30 }, Coordinate { x: 20, y: 40 });
        assert_eq!(rect2.area(), 100);
    }
    #[test]
    fn area_polygon_inner_test() {
        let poly = polygon![
            exterior: [
                (x: 0., y: 0.),
                (x: 10., y: 0.),
                (x: 10., y: 10.),
                (x: 0., y: 10.),
                (x: 0., y: 0.)
            ],
            interiors: [
                [
                    (x: 1., y: 1.),
                    (x: 2., y: 1.),
                    (x: 2., y: 2.),
                    (x: 1., y: 2.),
                    (x: 1., y: 1.),
                ],
                [
                    (x: 5., y: 5.),
                    (x: 6., y: 5.),
                    (x: 6., y: 6.),
                    (x: 5., y: 6.),
                    (x: 5., y: 5.)
                ],
            ],
        ];
        assert_relative_eq!(poly.area(), 98.);
    }
    #[test]
    fn area_multipolygon_test() {
        let poly0 = polygon![
            (x: 0., y: 0.),
            (x: 10., y: 0.),
            (x: 10., y: 10.),
            (x: 0., y: 10.),
            (x: 0., y: 0.)
        ];
        let poly1 = polygon![
            (x: 1., y: 1.),
            (x: 2., y: 1.),
            (x: 2., y: 2.),
            (x: 1., y: 2.),
            (x: 1., y: 1.)
        ];
        let poly2 = polygon![
            (x: 5., y: 5.),
            (x: 6., y: 5.),
            (x: 6., y: 6.),
            (x: 5., y: 6.),
            (x: 5., y: 5.)
        ];
        let mpoly = MultiPolygon(vec![poly0, poly1, poly2]);
        assert_relative_eq!(mpoly.area(), 102.);
        assert_relative_eq!(mpoly.area(), 102.);
    }
    #[test]
    fn area_line_test() {
        let line1 = Line::new(Coordinate { x: 0.0, y: 0.0 }, Coordinate { x: 1.0, y: 1.0 });
        assert_relative_eq!(line1.area(), 0.);
    }

    #[test]
    fn area_triangle_test() {
        let triangle = Triangle(
            Coordinate { x: 0.0, y: 0.0 },
            Coordinate { x: 1.0, y: 0.0 },
            Coordinate { x: 0.0, y: 1.0 },
        );
        assert_relative_eq!(triangle.area(), 0.5);

        let triangle = Triangle(
            Coordinate { x: 0.0, y: 0.0 },
            Coordinate { x: 0.0, y: 1.0 },
            Coordinate { x: 1.0, y: 0.0 },
        );
        assert_relative_eq!(triangle.area(), -0.5);
    }
}
