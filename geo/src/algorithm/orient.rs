use ::{CoordinateType, MultiPolygon, Polygon};

use algorithm::winding_order::{Winding, WindingOrder};

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
}

impl<T> Orient<T> for Polygon<T>
where
    T: CoordinateType,
{
    fn orient(&self, direction: Direction) -> Polygon<T> {
        orient(self, direction)
    }
}

impl<T> Orient<T> for MultiPolygon<T>
where
    T: CoordinateType,
{
    fn orient(&self, direction: Direction) -> MultiPolygon<T> {
        MultiPolygon(self.0.iter().map(|poly| poly.orient(direction)).collect())
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

// orient a Polygon according to convention
// by default, the exterior ring will be oriented ccw
// and the interior ring(s) will be oriented clockwise
fn orient<T>(poly: &Polygon<T>, direction: Direction) -> Polygon<T>
where
    T: CoordinateType,
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
    use ::{LineString, Point, Polygon};
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
        let poly1 = Polygon::new(LineString(points_ext), vec![LineString(points_int)]);
        // a diamond shape, oriented counter-clockwise outside,
        let oriented_ext = vec![(1.0, 0.0), (2.0, 1.0), (1.0, 2.0), (0.0, 1.0), (1.0, 0.0)];
        let oriented_ext_ls = LineString(
            oriented_ext
                .iter()
                .map(|e| Point::new(e.0, e.1))
                .collect::<Vec<_>>(),
        );
        // clockwise interior
        let oriented_int_raw = vec![(1.0, 0.5), (0.5, 1.0), (1.0, 1.5), (1.5, 1.0), (1.0, 0.5)];
        let oriented_int_ls = LineString(
            oriented_int_raw
                .iter()
                .map(|e| Point::new(e.0, e.1))
                .collect::<Vec<_>>(),
        );
        // build corrected Polygon
        let oriented = orient(&poly1, Direction::Default);
        assert_eq!(oriented.exterior.0, oriented_ext_ls.0);
        assert_eq!(oriented.interiors[0].0, oriented_int_ls.0);
    }
}
