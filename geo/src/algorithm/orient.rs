use crate::{GeoNum, MultiPolygon, Polygon};

use crate::winding_order::{Winding, WindingOrder};

pub trait Orient {
    /// Orients a Polygon's exterior and interior rings according to convention
    ///
    /// By default, the exterior ring of a Polygon is oriented counter-clockwise, and any interior
    /// rings are oriented clockwise.
    ///
    /// # Examples
    ///
    /// ```
    /// use geo::orient::{Direction, Orient};
    /// use geo::polygon;
    ///
    /// // a diamond shape
    /// let polygon = polygon![
    ///     // exterior oriented clockwise
    ///     exterior: [
    ///         (x: 1.0, y: 0.0),
    ///         (x: 0.0, y: 1.0),
    ///         (x: 1.0, y: 2.0),
    ///         (x: 2.0, y: 1.0),
    ///         (x: 1.0, y: 0.0),
    ///     ],
    ///     // interior oriented counter-clockwise
    ///     interiors: [
    ///         [
    ///             (x: 1.0, y: 0.5),
    ///             (x: 1.5, y: 1.0),
    ///             (x: 1.0, y: 1.5),
    ///             (x: 0.5, y: 1.0),
    ///             (x: 1.0, y: 0.5),
    ///         ],
    ///     ],
    /// ];
    ///
    /// let oriented = polygon.orient(Direction::Default);
    ///
    /// // a diamond shape
    /// let expected = polygon![
    ///     // exterior oriented counter-clockwise
    ///     exterior: [
    ///         (x: 1.0, y: 0.0),
    ///         (x: 2.0, y: 1.0),
    ///         (x: 1.0, y: 2.0),
    ///         (x: 0.0, y: 1.0),
    ///         (x: 1.0, y: 0.0),
    ///     ],
    ///     // interior oriented clockwise
    ///     interiors: [
    ///         [
    ///             (x: 1.0, y: 0.5),
    ///             (x: 0.5, y: 1.0),
    ///             (x: 1.0, y: 1.5),
    ///             (x: 1.5, y: 1.0),
    ///             (x: 1.0, y: 0.5),
    ///         ],
    ///     ],
    /// ];
    ///
    /// assert_eq!(expected, oriented);
    /// ```
    fn orient(&self, orientation: Direction) -> Self;
}

impl<T> Orient for Polygon<T>
where
    T: GeoNum,
{
    fn orient(&self, direction: Direction) -> Polygon<T> {
        orient(self, direction)
    }
}

impl<T> Orient for MultiPolygon<T>
where
    T: GeoNum,
{
    fn orient(&self, direction: Direction) -> MultiPolygon<T> {
        MultiPolygon::new(self.iter().map(|poly| poly.orient(direction)).collect())
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
    T: GeoNum,
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

#[cfg(test)]
mod test {
    use super::*;
    use crate::{LineString, Polygon};
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

#[cfg(test)]
mod hegel_props {
    use super::{Direction, Orient};
    use crate::utils::hegel_gens::{disjoint_multi_polygons, polygons_with_holes};
    use crate::{Area, Winding};

    // "By default, the exterior ring of a Polygon is oriented counter-clockwise,
    // and any interior rings are oriented clockwise"; `Reversed` swaps both.
    #[hegel::test]
    fn orient_gives_each_ring_the_documented_winding(tc: hegel::TestCase) {
        let polygon = tc.draw(polygons_with_holes());
        let reverse = tc.draw(hegel::generators::booleans());
        let direction = if reverse {
            Direction::Reversed
        } else {
            Direction::Default
        };
        let oriented = polygon.orient(direction);
        assert_eq!(oriented.exterior().is_cw(), reverse);
        for interior in oriented.interiors() {
            assert_eq!(interior.is_ccw(), reverse);
        }
    }

    #[hegel::test]
    fn orient_is_idempotent(tc: hegel::TestCase) {
        let polygon = tc.draw(polygons_with_holes());
        let oriented = polygon.orient(Direction::Default);
        assert_eq!(oriented.orient(Direction::Default), oriented);
    }

    // Orienting only reverses rings, which leaves the enclosed region — and so
    // its area — untouched.
    #[hegel::test]
    fn orient_preserves_unsigned_area(tc: hegel::TestCase) {
        let polygon = tc.draw(polygons_with_holes());
        assert_relative_eq!(
            polygon.orient(Direction::Reversed).unsigned_area(),
            polygon.unsigned_area(),
            max_relative = 1e-12
        );
    }

    // The `MultiPolygon` impl orients each member independently.
    #[hegel::test]
    fn orienting_a_multi_polygon_orients_each_member(tc: hegel::TestCase) {
        let multi_polygon = tc.draw(disjoint_multi_polygons());
        let expected: Vec<_> = multi_polygon
            .iter()
            .map(|p| p.orient(Direction::Default))
            .collect();
        assert_eq!(multi_polygon.orient(Direction::Default).0, expected);
    }
}
