use geo_types::line_string::PointsIter;
use geo_types::{CoordinateType, LineString, MultiPolygon, Point, Polygon};
use std::iter::Rev;

pub trait Orient<T> {
    /// Orients a Polygon's exterior and interior rings according to convention
    ///
    /// By default, the exterior ring of a Polygon is oriented counter-clockwise, and any interior
    /// rings are oriented clockwise.
    ///
    /// # Examples
    ///
    /// ```
    /// use geo_types::{Point, LineString, Polygon};
    /// use geo_winding_order::{Orient, Direction};
    /// // a diamond shape, oriented clockwise outside
    /// let points_ext = vec![(1.0, 0.0), (0.0, 1.0), (1.0, 2.0), (2.0, 1.0), (1.0, 0.0)];
    /// // counter-clockwise interior
    /// let points_int = vec![(1.0, 0.5), (1.5, 1.0), (1.0, 1.5), (0.5, 1.0), (1.0, 0.5)];
    /// let poly = Polygon::new(LineString::from(points_ext), vec![LineString::from(points_int)]);
    /// // a diamond shape, oriented counter-clockwise outside,
    /// let oriented_ext = vec![(1.0, 0.0), (2.0, 1.0), (1.0, 2.0), (0.0, 1.0), (1.0, 0.0)];
    /// let oriented_ext_ls = LineString::from(oriented_ext);
    /// // clockwise interior
    /// let oriented_int = vec![(1.0, 0.5), (0.5, 1.0), (1.0, 1.5), (1.5, 1.0), (1.0, 0.5)];
    /// let oriented_int_ls = LineString::from(oriented_int);
    /// // build corrected Polygon
    /// let oriented = poly.orient(Direction::Default);
    /// assert_eq!(oriented.exterior().0, oriented_ext_ls.0);
    /// assert_eq!(oriented.interiors()[0].0, oriented_int_ls.0);
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
    let interiors = poly
        .interiors()
        .iter()
        .map(|l| {
            l.clone_to_winding_order(match direction {
                Direction::Default => WindingOrder::Clockwise,
                Direction::Reversed => WindingOrder::CounterClockwise,
            })
        })
        .collect();

    let ext_ring = poly.exterior().clone_to_winding_order(match direction {
        Direction::Default => WindingOrder::CounterClockwise,
        Direction::Reversed => WindingOrder::Clockwise,
    });

    Polygon::new(ext_ring, interiors)
}

#[doc(hidden)]
pub fn twice_signed_ring_area<T>(linestring: &LineString<T>) -> T
where
    T: CoordinateType,
{
    if linestring.0.is_empty() || linestring.0.len() == 1 {
        return T::zero();
    }
    let mut tmp = T::zero();
    for line in linestring.lines() {
        tmp = tmp + line.determinant();
    }

    tmp
}

/// Iterates through a list of `Point`s
pub struct Points<'a, T>(EitherIter<Point<T>, PointsIter<'a, T>, Rev<PointsIter<'a, T>>>)
where
    T: CoordinateType + 'a;

impl<'a, T> Iterator for Points<'a, T>
where
    T: CoordinateType,
{
    type Item = Point<T>;

    fn next(&mut self) -> Option<Self::Item> {
        self.0.next()
    }
}

/// How a linestring is wound, clockwise or counter-clockwise
#[derive(PartialEq, Clone, Debug, Eq)]
pub enum WindingOrder {
    Clockwise,
    CounterClockwise,
}

/// Calculate, and work with, the winding order
pub trait Winding<T>
where
    T: CoordinateType,
{
    /// Return the winding order of this object
    fn winding_order(&self) -> Option<WindingOrder>;

    /// True iff this clockwise
    fn is_cw(&self) -> bool {
        self.winding_order() == Some(WindingOrder::Clockwise)
    }

    /// True iff this is wound counterclockwise
    fn is_ccw(&self) -> bool {
        self.winding_order() == Some(WindingOrder::CounterClockwise)
    }

    /// Iterate over the points in a clockwise order
    ///
    /// The object isn't changed, and the points are returned either in order, or in reverse
    /// order, so that the resultant order makes it appear clockwise
    fn points_cw(&self) -> Points<T>;

    /// Iterate over the points in a counter-clockwise order
    ///
    /// The object isn't changed, and the points are returned either in order, or in reverse
    /// order, so that the resultant order makes it appear counter-clockwise
    fn points_ccw(&self) -> Points<T>;

    /// Change this objects's points so they are in clockwise winding order
    fn make_cw_winding(&mut self);

    /// Change this line's points so they are in counterclockwise winding order
    fn make_ccw_winding(&mut self);

    /// Return a clone of this object, but in the specified winding order
    fn clone_to_winding_order(&self, winding_order: WindingOrder) -> Self
    where
        Self: Sized + Clone,
    {
        let mut new: Self = self.clone();
        new.make_winding_order(winding_order);
        new
    }

    /// Change the winding order so that it is in this winding order
    fn make_winding_order(&mut self, winding_order: WindingOrder) {
        match winding_order {
            WindingOrder::Clockwise => self.make_cw_winding(),
            WindingOrder::CounterClockwise => self.make_ccw_winding(),
        }
    }
}

impl<T> Winding<T> for LineString<T>
where
    T: CoordinateType,
{
    /// Returns the winding order of this line
    /// None if the winding order is undefined.
    fn winding_order(&self) -> Option<WindingOrder> {
        let shoelace = twice_signed_ring_area(self);
        if shoelace < T::zero() {
            Some(WindingOrder::Clockwise)
        } else if shoelace > T::zero() {
            Some(WindingOrder::CounterClockwise)
        } else if shoelace == T::zero() {
            None
        } else {
            // make compiler stop complaining
            unreachable!()
        }
    }

    /// Iterate over the points in a clockwise order
    ///
    /// The Linestring isn't changed, and the points are returned either in order, or in reverse
    /// order, so that the resultant order makes it appear clockwise
    fn points_cw(&self) -> Points<T> {
        match self.winding_order() {
            Some(WindingOrder::CounterClockwise) => Points(EitherIter::B(self.points_iter().rev())),
            _ => Points(EitherIter::A(self.points_iter())),
        }
    }

    /// Iterate over the points in a counter-clockwise order
    ///
    /// The Linestring isn't changed, and the points are returned either in order, or in reverse
    /// order, so that the resultant order makes it appear counter-clockwise
    fn points_ccw(&self) -> Points<T> {
        match self.winding_order() {
            Some(WindingOrder::Clockwise) => Points(EitherIter::B(self.points_iter().rev())),
            _ => Points(EitherIter::A(self.points_iter())),
        }
    }

    /// Change this line's points so they are in clockwise winding order
    fn make_cw_winding(&mut self) {
        if let Some(WindingOrder::CounterClockwise) = self.winding_order() {
            self.0.reverse();
        }
    }

    /// Change this line's points so they are in counterclockwise winding order
    fn make_ccw_winding(&mut self) {
        if let Some(WindingOrder::Clockwise) = self.winding_order() {
            self.0.reverse();
        }
    }
}

/// Enumeration that allows for two distinct iterator types that yield the same type.
enum EitherIter<T, I1, I2>
where
    I1: Iterator<Item = T>,
    I2: Iterator<Item = T>,
{
    A(I1),
    B(I2),
}

impl<T, I1, I2> Iterator for EitherIter<T, I1, I2>
where
    I1: Iterator<Item = T>,
    I2: Iterator<Item = T>,
{
    type Item = T;

    fn next(&mut self) -> Option<Self::Item> {
        match self {
            EitherIter::A(iter) => iter.next(),
            EitherIter::B(iter) => iter.next(),
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use geo_types::{LineString, Polygon};

    #[test]
    fn winding_order() {
        // 3 points forming a triangle
        let a = Point::new(0., 0.);
        let b = Point::new(2., 0.);
        let c = Point::new(1., 2.);

        // That triangle, but in clockwise ordering
        let cw_line = LineString::from(vec![a.0, c.0, b.0, a.0]);
        // That triangle, but in counterclockwise ordering
        let ccw_line = LineString::from(vec![a.0, b.0, c.0, a.0]);

        assert_eq!(cw_line.winding_order(), Some(WindingOrder::Clockwise));
        assert_eq!(cw_line.is_cw(), true);
        assert_eq!(cw_line.is_ccw(), false);
        assert_eq!(
            ccw_line.winding_order(),
            Some(WindingOrder::CounterClockwise)
        );
        assert_eq!(ccw_line.is_cw(), false);
        assert_eq!(ccw_line.is_ccw(), true);

        let cw_points1: Vec<_> = cw_line.points_cw().collect();
        assert_eq!(cw_points1.len(), 4);
        assert_eq!(cw_points1[0], a);
        assert_eq!(cw_points1[1], c);
        assert_eq!(cw_points1[2], b);
        assert_eq!(cw_points1[3], a);

        let ccw_points1: Vec<_> = cw_line.points_ccw().collect();
        assert_eq!(ccw_points1.len(), 4);
        assert_eq!(ccw_points1[0], a);
        assert_eq!(ccw_points1[1], b);
        assert_eq!(ccw_points1[2], c);
        assert_eq!(ccw_points1[3], a);

        assert_ne!(cw_points1, ccw_points1);

        let cw_points2: Vec<_> = ccw_line.points_cw().collect();
        let ccw_points2: Vec<_> = ccw_line.points_ccw().collect();

        // cw_line and ccw_line are wound differently, but the ordered winding iterator should have
        // make them similar
        assert_eq!(cw_points2, cw_points2);
        assert_eq!(ccw_points2, ccw_points2);

        // test make_clockwise_winding
        let mut new_line1 = ccw_line.clone();
        new_line1.make_cw_winding();
        assert_eq!(new_line1.winding_order(), Some(WindingOrder::Clockwise));
        assert_eq!(new_line1, cw_line);
        assert_ne!(new_line1, ccw_line);

        // test make_counterclockwise_winding
        let mut new_line2 = cw_line.clone();
        new_line2.make_ccw_winding();
        assert_eq!(
            new_line2.winding_order(),
            Some(WindingOrder::CounterClockwise)
        );
        assert_ne!(new_line2, cw_line);
        assert_eq!(new_line2, ccw_line);
    }

    #[test]
    fn test_polygon_orientation() {
        // a diamond shape, oriented clockwise outside
        let points_ext = vec![(1.0, 0.0), (0.0, 1.0), (1.0, 2.0), (2.0, 1.0), (1.0, 0.0)];
        // counter-clockwise interior
        let points_int = vec![(1.0, 0.5), (1.5, 1.0), (1.0, 1.5), (0.5, 1.0), (1.0, 0.5)];
        let poly1 = Polygon::new(
            LineString::from(points_ext),
            vec![LineString::from(points_int)],
        );
        // a diamond shape, oriented counter-clockwise outside,
        let oriented_ext = vec![(1.0, 0.0), (2.0, 1.0), (1.0, 2.0), (0.0, 1.0), (1.0, 0.0)];
        let oriented_ext_ls = LineString::from(oriented_ext);
        // clockwise interior
        let oriented_int_raw = vec![(1.0, 0.5), (0.5, 1.0), (1.0, 1.5), (1.5, 1.0), (1.0, 0.5)];
        let oriented_int_ls = LineString::from(oriented_int_raw);
        // build corrected Polygon
        let oriented = orient(&poly1, Direction::Default);
        assert_eq!(oriented.exterior().0, oriented_ext_ls.0);
        assert_eq!(oriented.interiors()[0].0, oriented_int_ls.0);
    }
}
