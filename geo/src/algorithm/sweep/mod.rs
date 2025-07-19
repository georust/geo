// This module implements a simplified variant of the [Bentley-Ottmann] sweep line algorithm
// for efficiently finding line segment intersections, exposed through the [`Intersections`] iterator.
//
// ## Relationship to Classical Bentley-Ottmann Algorithm
//
// The classical Bentley-Ottmann algorithm uses three key data structures:
// 1. **Event queue**: A priority queue of sweep events (segment start/end points and intersections)
// 2. **Status structure**: A balanced binary search tree tracking active segments sorted by y-coordinate
// 3. **Event handling**: Dynamic insertion of newly discovered intersection events
//
// This implementation simplifies the classical approach in several ways:
//
// ### Simplifications from Classical Algorithm:
//
// 1. **No explicit event queue**: Instead of a priority queue, events are stored in vectors
//    and sorted once. This has the same O(n log n) complexity but a simpler implementation.
//    This is similar to [Chen & Chan, 2003].
//    Further, **Only INSERT (segment start) events are sorted**. We only need the DELETE event
//    (segment end) of the "current" event, which is already available to use by being at the
//    front of the INSERT events. Rather than sort DELETE events separately, they are instead stored
//    alongside their INSERT.
//
// 2. **No binary search tree**: Rather than maintaining a balanced BST of active segments,
//    we use pre-sorted iteration with early termination based on x-coordinate bounds.
//    **This is a key difference from traditional BO**: In traditional BO, the BST allows
//    tracking only the segments "above and below" the active segment - those are the only
//    ones which could intersect. Our simplified sweep checks **all x-overlapping segments**
//    for intersection, which means we'll find intersections regardless of multiple segment
//    intersections at the same point, but affects the runtime complexity.
//
// 3. **No intersection events**: The classical algorithm discovers intersections during the sweep
//    and adds them as new events. This implementation instead relies on iteration, and yields
//    intersections immediately as they are discovered.
//
// ### Algorithm Flow
//
// 1. **Preprocessing**:
//    - Convert line segments to `SweepLineInterval`s with left (INSERT) and right (DELETE) x-coordinates
//    - Sort intervals by their left (INSERT) x-coordinate
//
// 2. **Sweep Execution**:
//    - Process intervals in sorted order (left to right).
//    - For each interval, check all subsequent intervals that start before it ends. These intervals overlap in their x-coordinates and so are candidates for intersection.
//    - Apply `line_intersection` test to confirm actual intersections.
//
// 3. **Output**:
//    - Yield intersection results as they are discovered during the sweep.
//
// ## Runtime Complexity
//
// This approach changes the runtime complexity from the classical BO algorithm's
// O((n + k) log n) to **O((n + m) log n)**, where:
// - n is the number of line segments
// - k is the number of actual intersections
// - m is the number of x-coordinate overlaps (and m >= k)
//
// ## Tradeoffs
//
// **Advantages**:
// - More robust: finds intersections regardless of multiple segment intersections at the same point
// - Simpler implementation with fewer data structures
// - Speed gains from reduced algorithmic complexity
//
// **Disadvantages**:
// - More susceptible to pathological data: if most lines overlap without intersecting,
//   this simplified sweep performs worse than traditional BO
// - Checks more segment pairs than strictly necessary in some cases
//
// [Bentley-Ottmann]: https://en.wikipedia.org/wiki/Bentley%E2%80%93Ottmann_algorithm
// [Chen & Chan, 2003]: https://en.wikipedia.org/wiki/Bentley%E2%80%93Ottmann_algorithm#CITEREFChenChan2003

use crate::line_intersection::{line_intersection, LineIntersection};
use crate::{GeoFloat, Line};

#[cfg(test)]
mod tests;

/// Find all intersections within a collection of [`Line`]s using a simplified Bentley-Ottmann
/// sweep.
///
/// Yields `(Line, Line, LineIntersection)` tuples for each pair of input lines that intersect.
///
/// This is a drop-in replacement for computing [`LineIntersection`] over all pairs of the
/// collection.
///
/// ## Performance Characteristics
///
/// This implementation is most effective when you have many line segments with sparse intersections,
/// but even quite dense intersections are competitive in all but extreme cases.
///
/// As a rule of thumb, if each segment intersects fewer than 10% of the total segments, you can be
/// confident that this algorithm will be competitive with or better than brute force.
/// For a concrete example, given 1,000 line segments, you'd need more than 100,000 intersections
/// between them before brute force would be faster.
///
/// ## Examples
///
/// ```
/// use geo::Line;
/// use geo::algorithm::sweep::Intersections;
/// use std::iter::FromIterator;
/// let input = vec![
///     Line::from([(1., 0.), (0., 1.)]),
///     Line::from([(0., 0.), (1., 1.)]),
/// ];
/// let intersections: Vec<_> = Intersections::from_iter(input).iter().collect();
/// // Check that we get the expected intersection
/// assert_eq!(intersections.len(), 1);
/// ```
///
/// [Bentley-Ottmann]: https://en.wikipedia.org/wiki/Bentley%E2%80%93Ottmann_algorithm
pub struct Intersections<T: GeoFloat> {
    index: SweepLineIndex<T>,
}

impl<T: GeoFloat> Intersections<T> {
    pub fn new(segments: impl IntoIterator<Item = Line<T>>) -> Self {
        let index = SweepLineIndex::new(segments);
        Self { index }
    }

    /// Iterate over all pairs of intersecting segments.
    ///
    /// Uses a simplified Bentley-Ottmann sweep line algorithm running in `O((n + m) log n)` time,
    /// where `n` is the number of line segments and `m` is the number of x-coordinate overlaps.
    pub fn iter(&self) -> impl Iterator<Item = (Line<T>, Line<T>, LineIntersection<T>)> + '_ {
        // The SweepLineIndex produces intersection candidates - those whose x-coordinates overlap,
        // which can be found efficiently and is a prerequisite for intersection.
        self.index.x_overlaps().flat_map(|(segment1, segment2)| {
            line_intersection(segment1, segment2)
                .map(|intersection| (segment1, segment2, intersection))
        })
    }
}

impl<T: GeoFloat> FromIterator<Line<T>> for Intersections<T> {
    fn from_iter<I: IntoIterator<Item = Line<T>>>(iter: I) -> Self {
        Self::new(iter)
    }
}

/// A line segment interval for sweep line algorithms.
///
/// Stores the x-bounds of a line segment so the sweep line can uniformly process
/// segments from left to right (-x to +x), regardless of the original segment orientation.
#[derive(Debug, Clone, Copy)]
struct SweepLineInterval<T: GeoFloat> {
    /// Minimum x value where segment enters sweep
    inserted_x: T,
    /// Maximum x value where segment exits sweep
    deleted_x: T,
    /// Original line segment
    segment: Line<T>,
}

/// The SweepLineIndex tracks events in the sweep line algorithm.
///
/// Events represent the start or end points of line segments.
/// Insert events happen at the left end of a segment, delete events at the right end.
///
/// # Event Relationships
///
/// Each line segment generates two events: an `INSERT` event at its left end and
/// a `DELETE` event at its right end.
///
/// # Implementation Note: Why No Binary Search Tree or Priority Queue?
///
/// The Bentley-Ottmann algorithm as described typically uses three tracking data structures:
///
/// 1. Separate `INSERT` and `DELETE` **events**.
/// 2. A **priority queue** to process these events in coordinate order
/// 3. A **balanced binary search tree** to track active segments sorted by their `y`-coordinate position
///
/// This implementation avoids all three:
///
/// - Instead of a priority queue, **events are stored in a vec and sorted once**, and then the events are processed in order.
///   This has the same asymptotic complexity.
///
/// - Instead of separate `INSERT` and `DELETE` events, we track a single active segment. The access pattern is such that
///   we only ever need the `DELETE` event of that active segment, so it makes sense to store it directly with the INSERT
///   rather than track DELETE events separately.
///
/// - Instead of a binary search tree, to track our active segments, we take advantage of our events being pre-sorted,
///   and leverage iteration, bailing out when we've passed the bounds of our active segment.
struct SweepLineIndex<T: GeoFloat> {
    // ordered by their INSERT (min) x value
    inserted_intervals: Vec<SweepLineInterval<T>>,
}

impl<T: GeoFloat> SweepLineIndex<T> {
    fn new(segments: impl IntoIterator<Item = Line<T>>) -> Self {
        // Add all segments to the sweep line index
        let mut inserted_intervals = Vec::new();

        for segment in segments.into_iter() {
            let (inserted_x, deleted_x) = if segment.start.x < segment.end.x {
                (segment.start.x, segment.end.x)
            } else {
                (segment.end.x, segment.start.x)
            };

            let interval = SweepLineInterval {
                inserted_x,
                deleted_x,
                segment,
            };
            inserted_intervals.push(interval);
        }

        // Sort inserts by x-coordinate
        inserted_intervals.sort_by(|a, b| a.inserted_x.total_cmp(&b.inserted_x));

        Self { inserted_intervals }
    }

    /// Get all pairs of segments whose x-coordinates overlap
    fn x_overlaps(&self) -> impl Iterator<Item = (Line<T>, Line<T>)> + '_ {
        // Process INSERT events in sorted order
        self.inserted_intervals
            .iter()
            .enumerate()
            .flat_map(move |(i, current_interval)| {
                // Check all INSERT events that start after this one
                //
                // This iterator chain implements the core sweep line logic:
                // 1. Start from the next INSERT event (i+1)
                // 2. Yield all segments that start BEFORE the current segment ends
                //    (their INSERT x <= our DELETE x)
                //    As these are the only segments that overlap, they are the only candidates
                //    for intersection with the current segment
                self.inserted_intervals[i + 1..]
                    .iter()
                    .take_while(move |interval| interval.inserted_x <= current_interval.deleted_x)
                    .map(move |overlapping_interval| {
                        // overlap should be true by construction at this point
                        debug_assert!(intervals_overlap(current_interval, overlapping_interval));
                        (current_interval.segment, overlapping_interval.segment)
                    })
            })
    }
}

/// Helper function to check if two intervals overlap on the `x`-axis
///
/// This determines which segments might geometrically intersect during the sweep.
/// We use `total_cmp` for robust floating-point comparisons to handle edge cases
/// involving very close values.
fn intervals_overlap<T: GeoFloat>(s0: &SweepLineInterval<T>, s1: &SweepLineInterval<T>) -> bool {
    s0.inserted_x.total_cmp(&s1.inserted_x).is_le()
        && s0.deleted_x.total_cmp(&s1.inserted_x).is_ge()
}
