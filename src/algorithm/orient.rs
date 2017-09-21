use num_traits::Float;
use types::{LineString, Polygon, MultiPolygon};

pub trait Orient<T> {
    /// Orients a Polygon's exterior and interior rings according to convention
    ///
    /// By default, the exterior ring of a Polygon is oriented counter-clockwise, and any interior
    /// rings are oriented clockwise.
    ///
    /// ```
    /// use geo::{Point, LineString, Polygon};
    /// use geo::orient::{Orient, Direction};
    /// // a diamond shape, oriented clockwise outside
    /// let points_ext_raw = vec![(1.0, 0.0), (0.0, 1.0), (1.0, 2.0), (2.0, 1.0), (1.0, 0.0)];
    /// // counter-clockwise interior
    /// let points_int_raw = vec![(1.0, 0.5), (1.5, 1.0), (1.0, 1.5), (0.5, 1.0), (1.0, 0.5)];
    /// let points_ext = points_ext_raw.iter().map(|e| Point::new(e.0, e.1)).collect::<Vec<_>>();
    /// let points_int = points_int_raw.iter().map(|e| Point::new(e.0, e.1)).collect::<Vec<_>>();
    /// let poly = Polygon::new(LineString(points_ext), vec![LineString(points_int)]);
    /// // a diamond shape, oriented counter-clockwise outside,
    /// let oriented_ext = vec![(1.0, 0.0), (2.0, 1.0), (1.0, 2.0), (0.0, 1.0), (1.0, 0.0)];
    /// let oriented_ext_ls = LineString(oriented_ext.iter().map(|e| Point::new(e.0, e.1)).collect::<Vec<_>>());
    /// // clockwise interior
    /// let oriented_int = vec![(1.0, 0.5), (0.5, 1.0), (1.0, 1.5), (1.5, 1.0), (1.0, 0.5)];
    /// let oriented_int_ls = LineString(oriented_int.iter().map(|e| Point::new(e.0, e.1)).collect::<Vec<_>>());
    /// // build corrected Polygon
    /// let oriented = poly.orient(Direction::Default);
    /// assert_eq!(oriented.exterior.0, oriented_ext_ls.0);
    /// assert_eq!(oriented.interiors[0].0, oriented_int_ls.0);
    /// ```
    fn orient(&self, orientation: Direction) -> Self;

    /// Orient a Polygon according to default orientation
    fn orient_default(&self) -> Self
        where Self: Sized
    {
        self.orient(Direction::Default)
    }

    /// Orient a Polygon, but reversed from the default
    fn orient_reversed(&self) -> Self
        where Self: Sized
    {
        self.orient(Direction::Reversed)
    }
}

impl<T> Orient<T> for Polygon<T>
    where T: Float
{
    fn orient(&self, direction: Direction) -> Polygon<T> {
        orient(self, direction)
    }
}

impl<T> Orient<T> for MultiPolygon<T>
    where T: Float
{
    fn orient(&self, direction: Direction) -> MultiPolygon<T> {
        MultiPolygon(self.0
                         .iter()
                         .map(|poly| poly.orient(direction))
                         .collect())
    }
}

/// By default, a properly-oriented Polygon has its outer ring oriented counter-clockwise,
/// and its inner ring(s) oriented clockwise. Selecting `Reversed` will result in a Polygon
/// with a clockwise-oriented exterior ring, and counter-clockwise interior ring(s)
#[derive(Copy, Clone, Debug)]
pub enum Direction {
    /// exterior ring is oriented counter-clockwise, interior rings are oriented clockwise
    Default,
    /// exterior ring is oriented clockwise, interior rings are oriented counter-clockwise
    Reversed,
}

#[derive(PartialEq, Copy, Clone, Debug)]
pub enum WindingOrder {
    Clockwise,
    CounterClockwise,
}

// the signed area of a linear ring
pub fn signed_ring_area<T>(linestring: &LineString<T>) -> T
    where T: Float
{
    if linestring.0.is_empty() || linestring.0.len() == 1 {
        return T::zero();
    }
    let mut tmp = T::zero();
    for line in linestring.lines() {
        tmp = tmp + (line.start.x() * line.end.y() - line.end.x() * line.start.y());
    }
    tmp / (T::one() + T::one())
}

// orient a Polygon according to convention
// by default, the exterior ring will be oriented ccw
// and the interior ring(s) will be oriented clockwise
fn orient<T>(poly: &Polygon<T>, direction: Direction) -> Polygon<T>
    where T: Float
{
    let interiors = poly.interiors.iter().map(|l| l.clone_to_winding_order(
        match direction {
            Direction::Default => WindingOrder::Clockwise,
            Direction::Reversed => WindingOrder::CounterClockwise,
        })).collect();

    let ext_ring = poly.exterior.clone_to_winding_order(match direction {
        Direction::Default => WindingOrder::CounterClockwise,
        Direction::Reversed => WindingOrder::Clockwise,
    });

    Polygon::new(ext_ring, interiors)
}

#[cfg(test)]
mod test {
    use types::{Polygon, LineString, Point};
    use super::*;
    #[test]
    fn test_polygon_orientation() {
        // a diamond shape, oriented clockwise outside
        let points_ext_raw = vec![(1.0, 0.0), (0.0, 1.0), (1.0, 2.0), (2.0, 1.0), (1.0, 0.0)];
        // counter-clockwise interior
        let points_int_raw = vec![(1.0, 0.5), (1.5, 1.0), (1.0, 1.5), (0.5, 1.0), (1.0, 0.5)];
        let points_ext = points_ext_raw
            .iter()
            .map(|e| Point::new(e.0, e.1))
            .collect::<Vec<_>>();
        let points_int = points_int_raw
            .iter()
            .map(|e| Point::new(e.0, e.1))
            .collect::<Vec<_>>();
        let poly1 = Polygon::new(LineString(points_ext.clone()), vec![LineString(points_int.clone())]);
        // a diamond shape, oriented counter-clockwise outside,
        let oriented_ext = vec![(1.0, 0.0), (2.0, 1.0), (1.0, 2.0), (0.0, 1.0), (1.0, 0.0)];
        let oriented_ext_ls = LineString(oriented_ext
                                             .iter()
                                             .map(|e| Point::new(e.0, e.1))
                                             .collect::<Vec<_>>());
        // clockwise interior
        let oriented_int_raw = vec![(1.0, 0.5), (0.5, 1.0), (1.0, 1.5), (1.5, 1.0), (1.0, 0.5)];
        let oriented_int_ls = LineString(oriented_int_raw
                                             .iter()
                                             .map(|e| Point::new(e.0, e.1))
                                             .collect::<Vec<_>>());
        // build corrected Polygon
        let oriented = orient(&poly1, Direction::Default);
        assert_eq!(oriented.exterior.0, oriented_ext_ls.0);
        assert_eq!(oriented.interiors[0].0, oriented_int_ls.0);


        let poly2 = Polygon::new(LineString(points_ext.clone()), vec![LineString(points_int.clone())]);
        // It is already in reverse orientation.
        let poly2_reversed = poly2.orient_reversed();
        assert_eq!(poly2, poly2_reversed);

        let poly2_default = poly2.orient_default();
        assert_eq!(poly2_default.exterior, LineString(vec![Point::new(1., 0.), Point::new(2., 1.), Point::new(1., 2.), Point::new(0., 1.), Point::new(1., 0.)]));

    }
}
