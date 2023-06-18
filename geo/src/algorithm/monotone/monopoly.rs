use geo_types::{private_utils::get_bounding_rect, Line};

use crate::{
    sweep::SweepPoint, BoundingRect, Coord, CoordinatePosition, GeoNum, HasKernel, Intersects,
    Kernel, LineString, Orientation, Polygon, Rect,
};

/// Monotone polygon
///
/// A monotone polygon is a polygon that can be decomposed into two
/// monotone chains (along the X-axis). This implies any vertical line
/// intersects the polygon at most twice (or not at all).
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
    /// Note: each chain must be a strictly increasing sequence (in the lexigraphic order),
    /// with the same start and end points.  Further, the top chain must be strictly above
    /// the bottom chain except at the end-points.
    pub(super) fn new(top: LineString<T>, bot: LineString<T>) -> Self {
        // TODO: move these to debug-only asserts
        assert_eq!(top.0.first(), bot.0.first());
        assert_eq!(top.0.last(), bot.0.last());
        assert_ne!(top.0.first(), top.0.last());
        let bounds = get_bounding_rect(top.0.iter().chain(bot.0.iter()).cloned()).unwrap();
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
    /// to the Y-axis at the given x-coordinate.
    pub fn bounding_segment(&self, pt: Coord<T>) -> Option<(Line<T>, Line<T>)> {
        // binary search for the segment that contains the x coordinate.
        let tl_idx = match self.top.0.binary_search_by(|coord| {
            SweepPoint::from(pt)
                .partial_cmp(&SweepPoint::from(*coord))
                .unwrap()
        }) {
            Ok(idx) => {
                if idx == self.top.0.len() - 1 {
                    idx - 1
                } else {
                    idx
                }
            }
            Err(idx) => {
                if idx == 0 || idx == self.top.0.len() {
                    return None;
                } else {
                    idx - 1
                }
            }
        };
        let bl_idx = match self.bot.0.binary_search_by(|coord| {
            SweepPoint::from(pt)
                .partial_cmp(&SweepPoint::from(*coord))
                .unwrap()
        }) {
            Ok(idx) => {
                if idx == self.bot.0.len() - 1 {
                    idx - 1
                } else {
                    idx
                }
            }
            Err(idx) => {
                debug_assert!(idx > 0 && idx < self.bot.0.len());
                idx - 1
            }
        };
        Some((
            Line::new(self.top.0[tl_idx], self.top.0[tl_idx + 1]),
            Line::new(self.bot.0[bl_idx], self.bot.0[bl_idx + 1]),
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
        let (top, bot) = if let Some(t) = self.bounding_segment(*coord) {
            t
        } else {
            return;
        };

        match <T as HasKernel>::Ker::orient2d(top.start, *coord, top.end) {
            Orientation::Clockwise => return,
            Orientation::Collinear => {
                *is_inside = true;
                *boundary_count += 1;
                return;
            }
            _ => {}
        }
        match <T as HasKernel>::Ker::orient2d(bot.start, *coord, bot.end) {
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
