use crate::{
    CoordinateType, Geometry, GeometryCollection, Line, LineString, MultiLineString, MultiPoint,
    MultiPolygon, Point, Polygon, Rect, Triangle,
};
use num_traits::Float;

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
/// assert_eq!(polygon.signed_area(), 30.);
/// assert_eq!(polygon.unsigned_area(), 30.);
///
/// polygon.exterior_mut(|line_string| {
///     line_string.0.reverse();
/// });
///
/// assert_eq!(polygon.signed_area(), -30.);
/// assert_eq!(polygon.unsigned_area(), 30.);
/// ```
pub trait Area<T>
where
    T: CoordinateType,
{
    fn signed_area(&self) -> T;

    fn unsigned_area(&self) -> T;
}

// Calculation of simple (no interior holes) Polygon area
pub(crate) fn get_linestring_area<T>(linestring: &LineString<T>) -> T
where
    T: Float,
{
    twice_signed_ring_area(linestring) / (T::one() + T::one())
}

impl<T> Area<T> for Point<T>
where
    T: CoordinateType,
{
    fn signed_area(&self) -> T {
        T::zero()
    }

    fn unsigned_area(&self) -> T {
        T::zero()
    }
}

impl<T> Area<T> for LineString<T>
where
    T: CoordinateType,
{
    fn signed_area(&self) -> T {
        T::zero()
    }

    fn unsigned_area(&self) -> T {
        T::zero()
    }
}

impl<T> Area<T> for Line<T>
where
    T: CoordinateType,
{
    fn signed_area(&self) -> T {
        T::zero()
    }

    fn unsigned_area(&self) -> T {
        T::zero()
    }
}

impl<T> Area<T> for Polygon<T>
where
    T: Float,
{
    fn signed_area(&self) -> T {
        self.interiors()
            .iter()
            .fold(get_linestring_area(self.exterior()), |total, next| {
                total - get_linestring_area(next)
            })
    }

    fn unsigned_area(&self) -> T {
        self.signed_area().abs()
    }
}

impl<T> Area<T> for MultiPoint<T>
where
    T: CoordinateType,
{
    fn signed_area(&self) -> T {
        T::zero()
    }

    fn unsigned_area(&self) -> T {
        T::zero()
    }
}

impl<T> Area<T> for MultiLineString<T>
where
    T: CoordinateType,
{
    fn signed_area(&self) -> T {
        T::zero()
    }

    fn unsigned_area(&self) -> T {
        T::zero()
    }
}

impl<T> Area<T> for MultiPolygon<T>
where
    T: Float,
{
    fn signed_area(&self) -> T {
        self.0
            .iter()
            .fold(T::zero(), |total, next| total + next.signed_area())
    }

    fn unsigned_area(&self) -> T {
        self.0
            .iter()
            .fold(T::zero(), |total, next| total + next.signed_area().abs())
    }
}

/// Because a `Rect` has no winding order, the area will always be positive.
impl<T> Area<T> for Rect<T>
where
    T: CoordinateType,
{
    fn signed_area(&self) -> T {
        self.width() * self.height()
    }

    fn unsigned_area(&self) -> T {
        self.width() * self.height()
    }
}

impl<T> Area<T> for Triangle<T>
where
    T: Float,
{
    fn signed_area(&self) -> T {
        self.to_lines()
            .iter()
            .fold(T::zero(), |total, line| total + line.determinant())
            / (T::one() + T::one())
    }

    fn unsigned_area(&self) -> T {
        self.signed_area().abs()
    }
}

impl<T> Area<T> for Geometry<T>
where
    T: Float,
{
    fn signed_area(&self) -> T {
        match self {
            Geometry::Point(g) => g.signed_area(),
            Geometry::Line(g) => g.signed_area(),
            Geometry::LineString(g) => g.signed_area(),
            Geometry::Polygon(g) => g.signed_area(),
            Geometry::MultiPoint(g) => g.signed_area(),
            Geometry::MultiLineString(g) => g.signed_area(),
            Geometry::MultiPolygon(g) => g.signed_area(),
            Geometry::GeometryCollection(g) => g.signed_area(),
            Geometry::Rect(g) => g.signed_area(),
            Geometry::Triangle(g) => g.signed_area(),
        }
    }

    fn unsigned_area(&self) -> T {
        match self {
            Geometry::Point(g) => g.unsigned_area(),
            Geometry::Line(g) => g.unsigned_area(),
            Geometry::LineString(g) => g.unsigned_area(),
            Geometry::Polygon(g) => g.unsigned_area(),
            Geometry::MultiPoint(g) => g.unsigned_area(),
            Geometry::MultiLineString(g) => g.unsigned_area(),
            Geometry::MultiPolygon(g) => g.unsigned_area(),
            Geometry::GeometryCollection(g) => g.unsigned_area(),
            Geometry::Rect(g) => g.unsigned_area(),
            Geometry::Triangle(g) => g.unsigned_area(),
        }
    }
}

impl<T> Area<T> for GeometryCollection<T>
where
    T: Float,
{
    fn signed_area(&self) -> T {
        self.0
            .iter()
            .map(|g| g.signed_area())
            .fold(T::zero(), |acc, next| acc + next)
    }

    fn unsigned_area(&self) -> T {
        self.0
            .iter()
            .map(|g| g.unsigned_area())
            .fold(T::zero(), |acc, next| acc + next)
    }
}

#[cfg(test)]
mod test {
    use crate::algorithm::area::Area;
    use crate::{line_string, polygon, Coordinate, Line, MultiPolygon, Polygon, Rect, Triangle};

    // Area of the polygon
    #[test]
    fn area_empty_polygon_test() {
        let poly: Polygon<f32> = polygon![];
        assert_relative_eq!(poly.signed_area(), 0.);
    }

    #[test]
    fn area_one_point_polygon_test() {
        let poly = polygon![(x: 1., y: 0.)];
        assert_relative_eq!(poly.signed_area(), 0.);
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
        assert_relative_eq!(polygon.signed_area(), 30.);
    }
    #[test]
    fn rectangle_test() {
        let rect1: Rect<f32> =
            Rect::new(Coordinate { x: 10., y: 30. }, Coordinate { x: 20., y: 40. });
        assert_relative_eq!(rect1.signed_area(), 100.);

        let rect2: Rect<i32> = Rect::new(Coordinate { x: 10, y: 30 }, Coordinate { x: 20, y: 40 });
        assert_eq!(rect2.signed_area(), 100);
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
        assert_relative_eq!(poly.signed_area(), 98.);
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
        assert_relative_eq!(mpoly.signed_area(), 102.);
        assert_relative_eq!(mpoly.signed_area(), 102.);
    }
    #[test]
    fn area_line_test() {
        let line1 = Line::new(Coordinate { x: 0.0, y: 0.0 }, Coordinate { x: 1.0, y: 1.0 });
        assert_relative_eq!(line1.signed_area(), 0.);
    }

    #[test]
    fn area_triangle_test() {
        let triangle = Triangle(
            Coordinate { x: 0.0, y: 0.0 },
            Coordinate { x: 1.0, y: 0.0 },
            Coordinate { x: 0.0, y: 1.0 },
        );
        assert_relative_eq!(triangle.signed_area(), 0.5);

        let triangle = Triangle(
            Coordinate { x: 0.0, y: 0.0 },
            Coordinate { x: 0.0, y: 1.0 },
            Coordinate { x: 1.0, y: 0.0 },
        );
        assert_relative_eq!(triangle.signed_area(), -0.5);
    }

    #[test]
    fn area_multi_polygon_area_reversed() {
        let polygon_cw: Polygon<f32> = polygon![
            Coordinate { x: 0.0, y: 0.0 },
            Coordinate { x: 0.0, y: 1.0 },
            Coordinate { x: 1.0, y: 1.0 },
            Coordinate { x: 1.0, y: 0.0 },
            Coordinate { x: 0.0, y: 0.0 },
        ];
        let polygon_ccw: Polygon<f32> = polygon![
            Coordinate { x: 0.0, y: 0.0 },
            Coordinate { x: 1.0, y: 0.0 },
            Coordinate { x: 1.0, y: 1.0 },
            Coordinate { x: 0.0, y: 1.0 },
            Coordinate { x: 0.0, y: 0.0 },
        ];
        let polygon_area = polygon_cw.unsigned_area();

        let multi_polygon = MultiPolygon(vec![polygon_cw, polygon_ccw]);

        assert_eq!(polygon_area * 2., multi_polygon.unsigned_area());
    }
}
