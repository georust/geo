use crate::{BoundingRect, GeoNum, LineString, Polygon, Rect, sweep::SweepPoint};

/// Monotone polygon
/// 
/// A monotone polygon is a polygon that can be decomposed into two
/// monotone chains.
pub struct MonoPoly<T: GeoNum> {
    top: LineString<T>,
    bot: LineString<T>,
}

impl<T: GeoNum> BoundingRect<T> for MonoPoly<T> {
    type Output = Rect<T>;

    fn bounding_rect(&self) -> Self::Output {
        let min_x = self.top.0[0].x;
        let max_x = self.top.0.last().unwrap().x;

        let mut max_y = self.top.0[0].y;
        for coord in self.top.0.iter() {
            if coord.y > max_y {
                max_y = coord.y;
            }
        }
        let mut min_y = max_y;
        for coord in self.bot.0.iter() {
            if coord.y < min_y {
                min_y = coord.y;
            }
        }
        assert!(min_x < max_x);
        assert!(min_y < max_y);
        Rect::new((min_x, min_y), (max_x, max_y))
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
    pub(super) fn new(top: LineString<T>, bot: LineString<T>) -> Self {
        assert_eq!(top.0.first(), bot.0.first());
        assert_eq!(top.0.last(), bot.0.last());
        assert_ne!(top.0.first(), top.0.last());

        for win in top.0.windows(2).chain(bot.0.windows(2)) {
            if SweepPoint::from(win[0]) >= SweepPoint::from(win[1]) {
                eprintln!("ERR: {:?} >= {:?}", win[0], win[1]);
            }
            assert!(SweepPoint::from(win[0]) < SweepPoint::from(win[1]));
        }
        Self { top, bot }
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

    pub fn into_ls_pair(self) -> (LineString<T>, LineString<T>) {
        (self.top, self.bot)
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


