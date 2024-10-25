use super::*;
use crate::{line_intersection::line_intersection, Coord, LineIntersection};

/// A segment of a input [`Cross`] type.
///
/// This type is used to convey the part of the input geometry that is
/// intersecting at a given intersection. This is returned by the
/// [`CrossingsIter::intersections`] method.
#[derive(Debug, Clone)]
pub(crate) struct Crossing<C: Cross> {
    /// The input associated with this segment.
    pub cross: C,

    #[allow(unused)]
    /// The geometry of this segment.
    ///
    /// This is a part of the input `crossable` geometry and either
    /// starts or ends at the intersection point last yielded by
    /// [`CrossingsIter`]. If it ends at the point (`at_left` is
    /// `false`), then it is guaranteed to not contain any other
    /// intersection point in its interior.
    pub line: LineOrPoint<C::Scalar>,

    /// Whether this is the first segment of the input line.
    pub first_segment: bool,

    /// Flag that is `true` if the next geom in the sequence overlaps
    /// (i.e. intersects at more than one point) with this. Not
    /// relevant and `false` if this is a point.
    ///
    /// Note that the overlapping segments may not always
    /// _all_ get batched together. They may be reported as
    /// one or more set of overlapping segments in an
    /// arbitrary order.
    pub has_overlap: bool,

    /// Flag that is `true` if the `geom` starts at the intersection
    /// point. Otherwise, it ends at the intersection point.
    pub at_left: bool,

    #[allow(unused)]
    pub(super) segment: IMSegment<C>,
}

impl<C: Cross + Clone> Crossing<C> {
    /// Convert `self` into a `Crossing` to return to user.
    pub(super) fn from_segment(segment: &IMSegment<C>, event_ty: EventType) -> Crossing<C> {
        Crossing {
            cross: segment.cross_cloned(),
            line: segment.geom(),
            first_segment: segment.is_first_segment(),
            has_overlap: segment.is_overlapping(),
            at_left: event_ty == EventType::LineLeft,
            segment: segment.clone(),
        }
    }
}

/// Iterator that yields all crossings.
///
/// Yields all end points, intersections and overlaps of a set of
/// line-segments and points. Construct it by `collect`-ing an
/// iterator of [`Cross`]. The implementation uses the
/// [Bentley-Ottman] algorithm and runs in time O((n + k) log n) time;
/// this is faster than a brute-force search for intersections across
/// all pairs of input segments if k --- the number of intersections
/// --- is small compared to n^2.
///
/// ## Usage
///
/// Construct from an iterator of any type implementing the
/// [`Cross`] trait. Use the [`CrossingsIter::intersections`]
/// method to access all segments that start or end at the last
/// yielded point.
///
/// ```rust,ignore
/// use geo::Line;
/// use geo::sweep::CrossingsIter;
/// use std::iter::FromIterator;
/// let input = vec![
///     Line::from([(1., 0.), (0., 1.)]),
///     Line::from([(0., 0.75), (1., 0.25)]),
///     Line::from([(0., 0.25), (1., 0.75)]),
///     Line::from([(0., 0.), (1., 1.)]),
/// ];
/// let iter = CrossingsIter::<_>::from_iter(input);
/// // 1 intersection point, and 8 end points
/// assert_eq!(iter.count(), 9);
/// ```
///
/// [Bentley-Ottman]: //en.wikipedia.org/wiki/Bentley%E2%80%93Ottmann_algorithm
pub(crate) struct CrossingsIter<C>
where
    C: Cross + Clone,
{
    sweep: Sweep<C>,
    segments: Vec<Crossing<C>>,
}

impl<C> CrossingsIter<C>
where
    C: Cross + Clone,
{
    /// Returns the segments that intersect the last point yielded by
    /// the iterator.
    pub fn intersections_mut(&mut self) -> &mut [Crossing<C>] {
        &mut self.segments
    }

    pub fn intersections(&self) -> &[Crossing<C>] {
        &self.segments
    }

    fn new_ex<T: IntoIterator<Item = C>>(iter: T, is_simple: bool) -> Self {
        let iter = iter.into_iter();
        let size = {
            let (min_size, max_size) = iter.size_hint();
            max_size.unwrap_or(min_size)
        };
        let sweep = Sweep::new(iter, is_simple);
        let segments = Vec::with_capacity(4 * size);
        Self { sweep, segments }
    }
}

impl<C> FromIterator<C> for CrossingsIter<C>
where
    C: Cross + Clone,
{
    fn from_iter<T: IntoIterator<Item = C>>(iter: T) -> Self {
        Self::new_ex(iter, false)
    }
}

impl<C> Iterator for CrossingsIter<C>
where
    C: Cross + Clone,
{
    type Item = Coord<C::Scalar>;

    fn next(&mut self) -> Option<Self::Item> {
        let segments = &mut self.segments;

        segments.clear();
        let mut last_point = self.sweep.peek_point();
        debug!("pt: {last_point:?}");
        while last_point == self.sweep.peek_point() && self.sweep.peek_point().is_some() {
            last_point = self.sweep.next_event(|seg, ty| {
                trace!(
                    "cb: {seg:?} {ty:?} (crossable = {cross:?})",
                    cross = seg.cross_cloned().line()
                );
                segments.push(Crossing::from_segment(seg, ty))
            });
        }

        if segments.is_empty() {
            None
        } else {
            last_point.map(|p| *p)
        }
    }
}

/// Iterator over all intersections of a collection of lines.
///
/// Yields tuples `(C, C, LineIntersection)` for each pair of input
/// crossables that intersect or overlap. This is a drop-in
/// replacement for computing [`LineIntersection`] over all pairs of
/// the collection, but is typically more efficient. The
/// implementation uses the [Bentley-Ottman] algorithm and runs in
/// time O((n + k) log n) time; this is faster than a brute-force
/// search for intersections across all pairs of input segments if k,
/// the number of intersections is small compared to n^2.
///
/// ## Usage
///
/// Construct from an iterator of any type implementing the
/// [`Cross`] trait. The geo-type [`Line`](crate::Line) implements this trait.
/// See the trait documentation for more information on usage with
/// custom types.
///
/// ```rust
/// use geo::Line;
/// use geo::sweep::Intersections;
/// use std::iter::FromIterator;
/// let input = vec![
///     Line::from([(1., 0.), (0., 1.)]),
///     Line::from([(0., 0.75), (1., 0.25)]),
///     Line::from([(0., 0.25), (1., 0.75)]),
///     Line::from([(0., 0.), (1., 1.)]),
/// ];
/// let iter = Intersections::<_>::from_iter(input);
/// // All pairs intersect
/// assert_eq!(iter.count(), 6);
/// ```
///
/// [Bentley-Ottman]: //en.wikipedia.org/wiki/Bentley%E2%80%93Ottmann_algorithm
pub struct Intersections<C: Cross + Clone> {
    inner: CrossingsIter<C>,
    idx: usize,
    jdx: usize,
    is_overlap: bool,
    pt: Option<Coord<C::Scalar>>,
}

impl<C> FromIterator<C> for Intersections<C>
where
    C: Cross + Clone,
{
    fn from_iter<T: IntoIterator<Item = C>>(iter: T) -> Self {
        Self {
            inner: FromIterator::from_iter(iter),
            idx: 0,
            jdx: 0,
            is_overlap: false,
            pt: None,
        }
    }
}

impl<C> Intersections<C>
where
    C: Cross + Clone,
{
    fn intersection(&mut self) -> Option<(C, C, LineIntersection<C::Scalar>)> {
        let (si, sj) = {
            let segments = self.inner.intersections();
            (&segments[self.idx], &segments[self.jdx])
        };
        debug!(
            "comparing intersection: [{iso}]",
            iso = if self.is_overlap { "OVL" } else { "" }
        );
        for i in [si, sj] {
            debug!(
                "\t{geom:?} ({at_left}) [{ovl}] [{first}]",
                geom = i.cross.line(),
                first = if i.first_segment { "FIRST" } else { "" },
                at_left = if i.at_left { "S" } else { "E" },
                ovl = if i.has_overlap { "OVL" } else { "" },
            );
        }
        // Ignore intersections that have already been processed
        let should_compute = if self.is_overlap {
            // For overlap, we only return intersection if both segments are the
            // first, and both are at left.
            debug_assert_eq!(si.at_left, sj.at_left);
            si.at_left && (si.first_segment && sj.first_segment)
        } else {
            (!si.at_left || si.first_segment) && (!sj.at_left || sj.first_segment)
        };

        if should_compute {
            let si = si.cross.clone();
            let sj = sj.cross.clone();

            let int = line_intersection(si.line().line(), sj.line().line())
                .expect("line_intersection returned `None` disagreeing with `CrossingsIter`");

            Some((si, sj, int))
        } else {
            None
        }
    }

    fn step(&mut self) -> bool {
        let seg_len = self.inner.intersections_mut().len();
        if 1 + self.jdx < seg_len {
            self.is_overlap =
                self.is_overlap && self.inner.intersections_mut()[self.jdx].has_overlap;
            self.jdx += 1;
        } else {
            self.idx += 1;
            if 1 + self.idx >= seg_len {
                loop {
                    self.pt = self.inner.next();
                    if self.pt.is_none() {
                        return false;
                    }
                    if self.inner.intersections_mut().len() > 1 {
                        break;
                    }
                }
                self.idx = 0;
            }
            self.is_overlap = self.inner.intersections_mut()[self.idx].has_overlap;
            self.jdx = self.idx + 1;
        }
        true
    }
}

impl<C> Iterator for Intersections<C>
where
    C: Cross + Clone,
{
    type Item = (C, C, LineIntersection<C::Scalar>);

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            if !self.step() {
                return None;
            }
            let it = self.intersection();
            debug!("\t{it:?}", it = it.is_some());
            if let Some(result) = it {
                return Some(result);
            }
        }
    }
}

#[cfg(test)]
pub(super) mod tests {
    use crate::Line;
    use log::info;
    use pretty_env_logger::env_logger;
    use std::{io::Write, rc::Rc};

    use super::*;

    pub(super) fn init_log() {
        let _ = env_logger::builder()
            .format(|buf, record| writeln!(buf, "{} - {}", record.level(), record.args()))
            .try_init();
    }

    #[test]
    fn simple_iter() {
        let input = vec![
            Rc::from(Line::from([(1., 0.), (0., 1.)])),
            Line::from([(0., 0.), (1., 1.)]).into(),
        ];
        let iter: CrossingsIter<_> = input.into_iter().collect();
        assert_eq!(iter.count(), 5);
    }

    #[test]
    fn overlap_intersect() {
        init_log();

        let input = [
            Line::from([(0., 0.), (1., 1.)]),
            [(1., 0.), (0., 1.)].into(),
            [(0., 0.5), (1., 0.5)].into(),
            [(-1., 0.5), (0.5, 0.5)].into(),
            [(0.5, 0.5), (0.5, 0.5)].into(),
            [(0., 0.), (0., 0.)].into(),
        ];
        // Intersections (by_idx):
        // (0, 1), (0, 2), (0, 3), (0, 4), (0, 5),
        // (1, 2), (1, 3), (1, 4),
        // (2, 3)
        let mut verify = 0;
        for (i, l1) in input.iter().enumerate() {
            for (j, l2) in input.iter().enumerate() {
                if j <= i {
                    continue;
                }
                if line_intersection(*l1, *l2).is_some() {
                    let lp_a = LineOrPoint::from(*l1);
                    let lp_b = LineOrPoint::from(*l2);
                    eprintln!("{lp_a:?} intersects {lp_b:?}",);
                    verify += 1;
                }
            }
        }

        let iter: Intersections<_> = input.iter().collect();
        let count = iter
            .inspect(|(a, b, _int)| {
                let lp_a = LineOrPoint::from(**a);
                let lp_b = LineOrPoint::from(**b);
                eprintln!("{lp_a:?} intersects {lp_b:?}",);
            })
            .count();
        assert_eq!(count, verify);
    }

    #[test]
    #[ignore]
    fn check_adhoc_crossings() {
        init_log();

        let input = vec![
            Line::from([(0., 0.), (1., 1.)]),
            [(1., 0.), (0., 1.)].into(),
            [(0., 0.5), (1., 0.5)].into(),
            [(-1., 0.5), (0.5, 0.5)].into(),
            [(0.5, 0.5), (0.5, 0.5)].into(),
            [(0., 0.), (0., 0.)].into(),
        ];

        let mut iter: CrossingsIter<_> = input.into_iter().collect();
        while let Some(pt) = iter.next() {
            info!("pt: {pt:?}");
            iter.intersections().iter().for_each(|i| {
                info!(
                    "\t{geom:?} ({at_left}) {ovl}",
                    geom = i.line,
                    at_left = if i.at_left { "S" } else { "E" },
                    ovl = if i.has_overlap { "[OVL] " } else { "" },
                );
            });
        }
    }
}
