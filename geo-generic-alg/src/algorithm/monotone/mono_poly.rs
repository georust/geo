use geo_types::{private_utils::get_bounding_rect, Line};

use crate::{
    coordinate_position::CoordPos, sweep::SweepPoint, BoundingRect, Coord, CoordinatePosition,
    GeoNum, Intersects, Kernel, LineString, Orientation, Polygon, Rect,
};

/// Monotone polygon
///
/// A monotone polygon is a polygon that can be decomposed into two monotone
/// chains (along the X-axis). This implies any vertical line intersects the
/// polygon at most twice (or not at all).  These polygons support
/// point-in-polygon queries in `O(log n)` time; use the `Intersects<Coord>`
/// trait to query.
///
/// This structure cannot be directly constructed.  Use
/// `crate::algorithm::monotone_subdivision` algorithm to obtain a
/// `Vec<MonoPoly>`.  Consider using `MonotonicPolygons` instead if you are not
/// interested in the individual monotone polygons.
#[derive(Clone, PartialEq)]
pub struct MonoPoly<T: GeoNum> {
    top: LineString<T>,
    bot: LineString<T>,
    bounds: Rect<T>,
}

impl<T: GeoNum> BoundingRect<T> for MonoPoly<T> {
    type Output = Rect<T>;

    fn bounding_rect(&self) -> Self::Output {
        self.bounds
    }
}
impl<T: GeoNum> std::fmt::Debug for MonoPoly<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let top: Vec<SweepPoint<T>> = self.top.0.iter().map(|c| (*c).into()).collect();
        let bot: Vec<SweepPoint<T>> = self.bot.0.iter().map(|c| (*c).into()).collect();
        f.debug_struct("MonoPoly")
            .field("top", &top)
            .field("bot", &bot)
            .finish()
    }
}

impl<T: GeoNum> MonoPoly<T> {
    /// Create a monotone polygon from the top and bottom chains.
    ///
    /// Note: each chain must be a strictly increasing sequence (in the lexicographic
    /// order), with the same start and end points.  Further, the top chain must be
    /// strictly above the bottom chain except at the end-points.  Not all these
    /// conditions are checked, and the algorithm may panic if they are not met.
    pub(super) fn new(top: LineString<T>, bot: LineString<T>) -> Self {
        debug_assert_eq!(top.0.first(), bot.0.first());
        debug_assert_eq!(top.0.last(), bot.0.last());
        debug_assert_ne!(top.0.first(), top.0.last());
        let bounds = get_bounding_rect(top.0.iter().chain(bot.0.iter())).unwrap();
        Self { top, bot, bounds }
    }

    /// Get a reference to the mono poly's top chain.
    #[must_use]
    pub fn top(&self) -> &LineString<T> {
        &self.top
    }

    /// Get a reference to the mono poly's bottom chain.
    #[must_use]
    pub fn bot(&self) -> &LineString<T> {
        &self.bot
    }

    /// Convert self to (top, bottom) pair of chains.
    pub fn into_ls_pair(self) -> (LineString<T>, LineString<T>) {
        (self.top, self.bot)
    }

    /// Get the pair of segments in the chain that intersects the line parallel
    /// to the Y-axis at the given x-coordinate.  Ties are broken by picking the
    /// segment with lower index, i.e. the segment closer to the start of the
    /// chains.
    pub fn bounding_segment(&self, x: T) -> Option<(Line<T>, Line<T>)> {
        // binary search for the segment that contains the x coordinate.
        let tl_idx = self.top.0.partition_point(|c| c.x < x);
        if tl_idx == 0 && self.top.0[0].x != x {
            return None;
        }
        let bl_idx = self.bot.0.partition_point(|c| c.x < x);
        if bl_idx == 0 {
            debug_assert_eq!(tl_idx, 0);
            debug_assert_eq!(self.bot.0[0].x, x);
            return Some((
                Line::new(self.top.0[0], self.top.0[1]),
                Line::new(self.bot.0[0], self.bot.0[1]),
            ));
        } else {
            debug_assert_ne!(tl_idx, 0);
        }

        Some((
            Line::new(self.top.0[tl_idx - 1], self.top.0[tl_idx]),
            Line::new(self.bot.0[bl_idx - 1], self.bot.0[bl_idx]),
        ))
    }

    /// Convert self into a [`Polygon`].
    pub fn into_polygon(self) -> Polygon<T> {
        let mut down = self.bot.0;
        let mut top = self.top.0;

        down.reverse();
        assert_eq!(down.first(), top.last());
        top.extend(down.drain(1..));

        let geom = LineString(top);
        debug_assert!(geom.is_closed());

        Polygon::new(geom, vec![])
    }
}

impl<T: GeoNum> CoordinatePosition for MonoPoly<T> {
    type Scalar = T;

    fn calculate_coordinate_position(
        &self,
        coord: &Coord<Self::Scalar>,
        is_inside: &mut bool,
        boundary_count: &mut usize,
    ) {
        if !self.bounds.intersects(coord) {
            return;
        }
        let (top, bot) = if let Some(t) = self.bounding_segment(coord.x) {
            t
        } else {
            return;
        };

        match T::Ker::orient2d(top.start, *coord, top.end) {
            Orientation::Clockwise => return,
            Orientation::Collinear => {
                *is_inside = true;
                *boundary_count += 1;
                return;
            }
            _ => {}
        }
        match T::Ker::orient2d(bot.start, *coord, bot.end) {
            Orientation::CounterClockwise => (),
            Orientation::Collinear => {
                *is_inside = true;
                *boundary_count += 1;
            }
            _ => {
                *is_inside = true;
            }
        }
    }
}
impl<T: GeoNum> Intersects<Coord<T>> for MonoPoly<T> {
    fn intersects(&self, other: &Coord<T>) -> bool {
        self.coordinate_position(other) != CoordPos::Outside
    }
}
