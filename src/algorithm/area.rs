use num::{Float, ToPrimitive};
use types::Polygon;

/// Calculation of the area.

pub trait Area<RHS = Self>
    where T: Float + ToPrimitive
{
    /// Area of polygon.
    /// See: https://en.wikipedia.org/wiki/Polygon
    ///
    /// ```
    /// use geo::{Coordinate, Point, LineString, Polygon};
    /// use geo::algorithm::area::Area;
    /// let p = |x, y| Point(Coordinate { x: x, y: y });
    /// let v = Vec::new();
    /// let linestring = LineString(vec![p(0., 0.), p(5., 0.), p(5., 6.), p(0., 6.), p(0., 0.)]);
    /// let poly = Polygon(linestring, v);
    /// assert_eq!(poly.area(), 30.);
    /// ```
    fn area(&self) -> T;
}

impl<T> Area<T> for Polygon<T>
    where T: Float + ToPrimitive
{
    fn area(&self) -> T {
        // TODO: consideration of inner polygons;
        let linestring = &self.0;
        if linestring.0.is_empty() || linestring.0.len() == 1 {
            return T::zero();
        }
        let mut tmp = T::zero();
        for (p1, p2) in linestring.0.iter().zip(linestring.0[1..].iter()) {
            tmp += p1.lng() * p2.lat() - p2.lng() * p1.lat();
        }
        tmp / (T::one() + T::one())
    }
}

#[cfg(test)]
mod test {
    use types::{Coordinate, Point, LineString, Polygon};
    use algorithm::area::Area;
    // Area of the polygon
    #[test]
    fn area_empty_polygon_test() {
        let poly = Polygon(LineString(Vec::new()), Vec::new());
        assert_eq!(poly.area(), 0.);
    }

    #[test]
    fn area_one_point_polygon_test() {
        let poly = Polygon(LineString(vec![Point::new(1., 0.)]), Vec::new());
        assert_eq!(poly.area(), 0.);
    }
    #[test]
    fn area_polygon_test() {
        let p = |x, y| Point(Coordinate { x: x, y: y });
        let linestring = LineString(vec![p(0., 0.), p(5., 0.), p(5., 6.), p(0., 6.), p(0., 0.)]);
        let poly = Polygon(linestring, Vec::new());
        assert_eq!(poly.area(), 30.);
    }
}
