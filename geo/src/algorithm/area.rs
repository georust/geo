use num_traits::{Num, Float, NumCast};
use {CoordinateType, Line, LineString, MultiPolygon, Polygon, Rect, Triangle};

use algorithm::winding_order::twice_signed_ring_area;

/// Calculation of the area.

pub trait Area<T, Output = T>
where
    T: CoordinateType,
    Output: Copy + Num + NumCast,
{
    /// Signed area of a geometry.
    ///
    /// # Examples
    ///
    /// ```
    /// use geo::{Coordinate, Point, LineString, Polygon};
    /// use geo::algorithm::area::Area;
    ///
    /// let mut polygon = Polygon::new(LineString::from(vec![
    ///     (0., 0.),
    ///     (5., 0.),
    ///     (5., 6.),
    ///     (0., 6.),
    ///     (0., 0.)
    /// ]), vec![]);
    ///
    /// let area: f32 = polygon.area();
    /// assert_eq!(area, 30.);
    ///
    /// polygon.exterior.0.reverse();
    ///
    /// let area: f32 = polygon.area();
    /// assert_eq!(area, -30.);
    /// ```
    fn area(&self) -> Output;
}

// Calculation of simple (no interior holes) Polygon area
pub(crate) fn get_linestring_area<T, Output>(linestring: &LineString<T>) -> Output
where
    T: Float,
    Output: Copy + Num + NumCast,
{
    twice_signed_ring_area::<T, Output>(linestring) / (Output::one() + Output::one())
}

impl<T, Output> Area<T, Output> for Line<T>
where
    T: CoordinateType,
    Output: Copy + Num + NumCast,
{
    fn area(&self) -> Output {
        Output::zero()
    }
}

impl<T, Output> Area<T, Output> for Polygon<T>
where
    T: Float,
    Output: Copy + Num + NumCast,
{
    fn area(&self) -> Output {
        self.interiors
            .iter()
            .fold(get_linestring_area(&self.exterior), |total, next| {
                total - get_linestring_area(next)
            })
    }
}

impl<T, Output> Area<T, Output> for MultiPolygon<T>
where
    T: Float,
    Output: Copy + Num + NumCast,
{
    fn area(&self) -> Output {
        self.0
            .iter()
            .fold(Output::zero(), |total, next| total + next.area())
    }
}

impl<T, Output> Area<T, Output> for Rect<T>
where
    T: CoordinateType,
    Output: Copy + Num + NumCast,
{
    fn area(&self) -> Output {
        let max_x = Output::from(self.max.x).unwrap();
        let max_y = Output::from(self.max.y).unwrap();
        let min_x = Output::from(self.min.x).unwrap();
        let min_y = Output::from(self.min.y).unwrap();
        (max_x - min_x) * (max_y - min_y)
    }
}

impl<T, Output> Area<T, Output> for Triangle<T>
where
    T: Float + NumCast,
    Output: Copy + Num + NumCast,
{
    fn area(&self) -> Output {
        (Line::new(self.0, self.1).determinant::<Output>()
            + Line::new(self.1, self.2).determinant::<Output>()
            + Line::new(self.2, self.0).determinant::<Output>())
            / (Output::one() + Output::one())
    }
}

#[cfg(test)]
mod test {
    use algorithm::area::Area;
    use {Coordinate, Line, LineString, MultiPolygon, Polygon, Rect, Triangle};

    // Area of the polygon
    #[test]
    fn area_empty_polygon_test() {
        let poly = Polygon::<f64>::new(LineString(Vec::new()), Vec::new());
        assert_relative_eq!(poly.area(), 0.);
    }

    #[test]
    fn area_one_point_polygon_test() {
        let poly = Polygon::new(LineString::from(vec![(1., 0.)]), Vec::new());
        assert_relative_eq!(poly.area(), 0.);
    }
    #[test]
    fn area_polygon_test() {
        let linestring = LineString::from(vec![(0., 0.), (5., 0.), (5., 6.), (0., 6.), (0., 0.)]);
        let poly = Polygon::new(linestring, Vec::new());
        assert_relative_eq!(poly.area(), 30.);
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
        assert_eq!(Area::<i32, i32>::area(&rect2), 100);
    }
    #[test]
    fn area_polygon_inner_test() {
        let outer = LineString::from(vec![(0., 0.), (10., 0.), (10., 10.), (0., 10.), (0., 0.)]);
        let inner0 = LineString::from(vec![(1., 1.), (2., 1.), (2., 2.), (1., 2.), (1., 1.)]);
        let inner1 = LineString::from(vec![(5., 5.), (6., 5.), (6., 6.), (5., 6.), (5., 5.)]);
        let poly = Polygon::new(outer, vec![inner0, inner1]);
        assert_relative_eq!(poly.area(), 98.);
    }
    #[test]
    fn area_multipolygon_test() {
        let poly0 = Polygon::new(
            LineString::from(vec![(0., 0.), (10., 0.), (10., 10.), (0., 10.), (0., 0.)]),
            Vec::new(),
        );
        let poly1 = Polygon::new(
            LineString::from(vec![(1., 1.), (2., 1.), (2., 2.), (1., 2.), (1., 1.)]),
            Vec::new(),
        );
        let poly2 = Polygon::new(
            LineString::from(vec![(5., 5.), (6., 5.), (6., 6.), (5., 6.), (5., 5.)]),
            Vec::new(),
        );
        let mpoly = MultiPolygon(vec![poly0, poly1, poly2]);
        assert_eq!(Area::<f32, f32>::area(&mpoly), 102.);
        assert_relative_eq!(mpoly.area(), 102.);
    }
    #[test]
    fn area_line_test() {
        let line1 = Line::new(Coordinate { x: 0.0, y: 0.0 }, Coordinate { x: 1.0, y: 1.0 });
        assert_eq!(Area::<f32, f32>::area(&line1), 0.);
    }

    #[test]
    fn area_triangle_test() {
        let triangle = Triangle(
            Coordinate { x: 0.0, y: 0.0 },
            Coordinate { x: 1.0, y: 0.0 },
            Coordinate { x: 0.0, y: 1.0 },
        );
        assert_eq!(Area::<f32, f32>::area(&triangle), 0.5);

        let triangle = Triangle(
            Coordinate { x: 0.0, y: 0.0 },
            Coordinate { x: 0.0, y: 1.0 },
            Coordinate { x: 1.0, y: 0.0 },
        );
        assert_eq!(Area::<f32, f32>::area(&triangle), -0.5);
    }
}
