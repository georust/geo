use crate::{CoordinateType, Line, LineString, MultiPolygon, Polygon, Rect, Triangle};
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

default impl<T> Area<T> for Line<T>
where
    T: CoordinateType,
{
    fn area(&self) -> T {
        T::zero()
    }
}

impl<T> Area<T> for Polygon<T>
where
    T: Float,
{
    fn area(&self) -> T {
        self.interiors()
            .iter()
            .fold(get_linestring_area(self.exterior()), |total, next| {
                total - get_linestring_area(next)
            })
    }
}

impl<T> Area<T> for MultiPolygon<T>
where
    T: Float,
{
    fn area(&self) -> T {
        self.0
            .iter()
            .fold(T::zero(), |total, next| total + next.area())
    }
}

impl<T> Area<T> for Rect<T>
where
    T: CoordinateType,
{
    fn area(&self) -> T {
        (self.max.x - self.min.x) * (self.max.y - self.min.y)
    }
}

impl<T> Area<T> for Triangle<T>
where
    T: Float,
{
    fn area(&self) -> T {
        (Line::new(self.0, self.1).determinant()
            + Line::new(self.1, self.2).determinant()
            + Line::new(self.2, self.0).determinant())
            / (T::one() + T::one())
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
        let rect1: Rect<f32> = Rect {
            min: Coordinate { x: 10., y: 30. },
            max: Coordinate { x: 20., y: 40. },
        };
        assert_relative_eq!(rect1.area(), 100.);

        let rect2: Rect<i32> = Rect {
            min: Coordinate { x: 10, y: 30 },
            max: Coordinate { x: 20, y: 40 },
        };
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
        assert_eq!(mpoly.area(), 102.);
        assert_relative_eq!(mpoly.area(), 102.);
    }
    #[test]
    fn area_line_test() {
        let line1 = Line::new(Coordinate { x: 0.0, y: 0.0 }, Coordinate { x: 1.0, y: 1.0 });
        assert_eq!(line1.area(), 0.);
    }

    #[test]
    fn area_triangle_test() {
        let triangle = Triangle(
            Coordinate { x: 0.0, y: 0.0 },
            Coordinate { x: 1.0, y: 0.0 },
            Coordinate { x: 0.0, y: 1.0 },
        );
        assert_eq!(triangle.area(), 0.5);

        let triangle = Triangle(
            Coordinate { x: 0.0, y: 0.0 },
            Coordinate { x: 0.0, y: 1.0 },
            Coordinate { x: 1.0, y: 0.0 },
        );
        assert_eq!(triangle.area(), -0.5);
    }
}
