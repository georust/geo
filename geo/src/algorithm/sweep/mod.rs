// This module implements the Bentley-Ottmann sweep line algorithm for efficiently finding
// line segment intersections, exposed through the [`Intersections`] iterator.
//
// Algorithm Flow
//
// 1. Collection:
//    - Line segments are collected and stored in `Intersections.segments`
//    - The `compute` method calls `compute_sweep_intersections` to process all segments.
//
// 2. Sweep Setup:
//    - Creates a new `SweepLineIndex` to manage the sweep process
//    - For each segment, creates a `SweepLineInterval` with min/max x-values
//    - Each interval generates two events: `INSERT` at left end, `DELETE` at right end.
//
// 3. Sweep Execution:
//    - `compute_intersections` sorts INSERT and DELETE events separately by `x` coordinate
//    - Creates a lookup array mapping segment indices to their DELETE x-coordinates
//    - Processes INSERT events in sorted order using functional iterator chains
//    - For each INSERT event, examines subsequent INSERT events up to its DELETE x-coordinate
//    - Filters for potentially overlapping intervals (using `intervals_overlap`)
//    - For each potential overlap, `line_intersection` checks for actual geometric intersection
//    - When found, intersections are stored in `intersection_pairs`.

use crate::line_intersection::{line_intersection, LineIntersection};
use crate::{GeoFloat, Line};
use std::cmp::Ordering;

#[cfg(test)]
mod tests;

/// Iterator over all intersections of a collection of [`Line`]s.
///
/// # Performance Note
///
/// This implementation is most useful when there is a need to efficiently find sparse intersections
/// between many line segments. For smaller numbers of segments, a brute-force approach
/// will be around 20 % faster in current tests: if you are reasonably confident that the number of
/// segments is larger than the number of intersections, use this algorithm.
///
/// Yields tuples `(`Line`, `Line`, `LineIntersection`)` for each pair of input
/// lines that intersect or overlap. This is a drop-in
/// replacement for computing [`LineIntersection`] over all pairs of
/// the collection, with the perf caveat noted above.
///
/// The implementation uses the [Bentley-Ottmann] sweep line algorithm,
/// which runs in `O((n + k) log n)` time, where `n` is the number of line segments
/// and `k` is the number of intersections.
///
/// # Examples
///
/// ```
/// use geo::Line;
/// use geo::algorithm::sweep::Intersections;
/// use std::iter::FromIterator;
/// let input = vec![
///     Line::from([(1., 0.), (0., 1.)]),
///     Line::from([(0., 0.), (1., 1.)]),
/// ];
/// let intersections: Vec<_> = Intersections::<_>::from_iter(input).collect();
/// // Check that we get the expected intersection
/// assert_eq!(intersections.len(), 1);
/// ```
///
/// [Bentley-Ottmann]: //en.wikipedia.org/wiki/Bentley%E2%80%93Ottmann_algorithm
pub struct Intersections<T: GeoFloat> {
    segments: Vec<Line<T>>,
    intersection_pairs: Vec<(Line<T>, Line<T>, LineIntersection<T>)>,
    current_index: usize,
}

impl<T: GeoFloat> Intersections<T> {
    fn new() -> Self {
        Self {
            segments: Vec::new(),
            intersection_pairs: Vec::new(),
            current_index: 0,
        }
    }

    /// Add a segment to the collection
    fn add_segment(&mut self, segment: Line<T>) {
        self.segments.push(segment);
    }

    /// Compute all intersections
    fn compute(&mut self) {
        self.compute_sweep_intersections();
    }

    /// Sweep line logic
    ///
    /// This method implements the sweep line algorithm for finding segment
    /// intersections. The algorithm runs in `O((n + k) log n)` time,
    /// where `n` is the number of line segments and `k` is the number of intersections.
    fn compute_sweep_intersections(&mut self) {
        let mut index = SweepLineIndex::new(self.segments.len());

        // Add all segments to the sweep line index
        for (idx, segment) in self.segments.iter().enumerate() {
            let min_x = segment.start.x.min(segment.end.x);
            let max_x = segment.start.x.max(segment.end.x);

            // Store the segment index with the interval
            let interval = SweepLineInterval {
                min: min_x,
                max: max_x,
                item: *segment,
                index: idx,
            };
            index.add(interval);
        }

        // Find all intersections using the sweep algorithm
        index.overlap_action(|s0, s1| {
            let segment1 = s0.item;
            let segment2 = s1.item;

            if let Some(intersection) = line_intersection(segment1, segment2) {
                self.intersection_pairs
                    .push((segment1, segment2, intersection));
            }
        });
    }
}

impl<T: GeoFloat> Iterator for Intersections<T> {
    type Item = (Line<T>, Line<T>, LineIntersection<T>);

    fn next(&mut self) -> Option<Self::Item> {
        if self.current_index < self.intersection_pairs.len() {
            let result = self.intersection_pairs[self.current_index];
            self.current_index += 1;
            Some(result)
        } else {
            None
        }
    }
}

impl<T: GeoFloat> FromIterator<Line<T>> for Intersections<T> {
    fn from_iter<I: IntoIterator<Item = Line<T>>>(iter: I) -> Self {
        let mut intersections = Intersections::new();
        for segment in iter {
            intersections.add_segment(segment);
        }
        intersections.compute();
        intersections
    }
}

/// A line segment represented as an `x`-coordinate interval with its original index.
///
/// This structure stores the `x` bounds of a line segment (min and max `x` values),
/// the original line segment, and its index in the original collection.
///
/// # Identity
///
/// The `index` field uniquely identifies each interval based on its position in the original
/// collection. This allows us to correlate INSERT and DELETE events for the same segment
/// without needing explicit pointers or a HashMap.
#[derive(Debug, Clone, Copy)]
struct SweepLineInterval<T: GeoFloat> {
    min: T,
    max: T,
    item: Line<T>, // Original line segment
    index: usize,  // Index used to match INSERT/DELETE event pairs
}

/// An event in the sweep line algorithm.
///
/// Events represent the start or end points of line segments.
/// Insert events happen at the left end of a segment, delete events at the right end.
///
/// # Event Relationships
///
/// Each line segment generates two events: an `INSERT` event at its left end and
/// a `DELETE` event at its right end. These paired events are correlated using the
/// segment index stored in the `SweepLineInterval`.
///
/// Rather than explicit pointers or references, events are stored in two separate vecs
/// (`insert_events` and `delete_events`) and correlated through their shared segment index.
/// A lookup array maps segment indices to DELETE x-coordinates for access during
/// the sweep process.
///
/// # Implementation Note: Why No Binary Search Tree or Priority Queue?
///
/// The Bentley-Ottmann algorithm as described typically uses two tracking data structures:
///
/// 1. A **priority queue** to process events in coordinate order
/// 2. A **balanced binary search tree** to track active segments sorted by their `y`-coordinate position
///
/// This implementation avoids both:
///
/// - Instead of a priority queue, the **event vecs are sorted once** and process events in order.
///   This has the same asymptotic complexity.
///
/// - Instead of a binary search tree, we use **two separate vecs** for INSERT and DELETE events.
///   When processing an `INSERT` event, we find its corresponding DELETE event by segment index
///   and check all INSERT events that start before that DELETE x-coordinate.
///
/// By maintaining separate vecs for INSERT and DELETE events and using the segment index
/// for correlation, we can determine which segments are active at any point
/// _without_ maintaining an "active segment" tree or using a HashMap.
#[derive(Debug, Clone)]
struct SweepLineEvent<T: GeoFloat> {
    x_value: T,
    interval: SweepLineInterval<T>,
}

impl<T: GeoFloat> SweepLineEvent<T> {
    fn new(x: T, interval: SweepLineInterval<T>) -> Self {
        Self {
            x_value: x,
            interval,
        }
    }
}

impl<T: GeoFloat> PartialEq for SweepLineEvent<T> {
    fn eq(&self, other: &Self) -> bool {
        self.x_value.total_cmp(&other.x_value) == Ordering::Equal
    }
}

impl<T: GeoFloat> Eq for SweepLineEvent<T> {}

impl<T: GeoFloat> PartialOrd for SweepLineEvent<T> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl<T: GeoFloat> Ord for SweepLineEvent<T> {
    fn cmp(&self, other: &Self) -> Ordering {
        // Use total_cmp for more robust floating point comparison
        // This is crucial for grid patterns where many x-values are exactly equal
        self.x_value.total_cmp(&other.x_value)
    }
}

struct SweepLineIndex<T: GeoFloat> {
    insert_events: Vec<SweepLineEvent<T>>,
    delete_events: Vec<SweepLineEvent<T>>,
}

impl<T: GeoFloat> SweepLineIndex<T> {
    fn new(num_segments: usize) -> Self {
        Self {
            insert_events: Vec::with_capacity(num_segments),
            delete_events: Vec::with_capacity(num_segments),
        }
    }

    fn add(&mut self, interval: SweepLineInterval<T>) {
        let x_min = interval.min;
        let x_max = interval.max;

        // Create INSERT event
        let insert_event = SweepLineEvent::new(x_min, interval);
        self.insert_events.push(insert_event);

        // Create DELETE event
        let delete_event = SweepLineEvent::new(x_max, interval);
        self.delete_events.push(delete_event);
    }

    fn overlap_action<F>(&mut self, mut intersection_action: F)
    where
        F: FnMut(&SweepLineInterval<T>, &SweepLineInterval<T>),
    {
        // Sort both by x-coordinate
        self.insert_events.sort();
        self.delete_events.sort();

        // Create a lookup array to map segment indices to their DELETE x-coordinates
        // This avoids O(n) search for each INSERT event
        // This will let us look up the insert event for a given interval index in constant time
        // JTS (and GEOS) don't need this as they use object references and pointers respectively
        // but these approaches are (obviously) a pain in Rust, so we need an intermediate mapping
        // https://github.com/simplegeo/jts/blob/3a4c9a9c3a7d3274e16bdd2f341df7d3e113d81b/src/com/vividsolutions/jts/index/sweepline/SweepLineIndex.java#L57-L59
        // We know the number of segments equals the number of INSERT events
        let num_segments = self.insert_events.len();
        let mut delete_x_by_index = vec![None; num_segments];

        self.delete_events
            .iter()
            .filter(|event| event.interval.index < num_segments)
            .for_each(|event| {
                delete_x_by_index[event.interval.index] = Some(event.x_value);
            });

        // Process INSERT events in sorted order
        self.insert_events
            .iter()
            .enumerate()
            .for_each(|(i, current_event)| {
                let current_interval = &current_event.interval;
                let current_segment_index = current_interval.index;

                // Get the DELETE x-coordinate from our lookup array
                // Every segment should have both INSERT and DELETE events
                let delete_x = delete_x_by_index[current_segment_index]
                    .expect("Every segment should have a DELETE event");

                // Check all INSERT events that start after this one
                // This iterator chain implements the core sweep line logic:
                // 1. Start from the next INSERT event (i+1)
                // 2. Only examine segments that start BEFORE the current segment ends
                //    (their INSERT x <= our DELETE x), as these are the only ones that
                //    could potentially intersect with the current segment
                // 3. Skip self-intersections by filtering out segments with the same index
                // 4. Check if the x-intervals actually overlap using intervals_overlap
                // 5. For each overlapping pair, carry out the actual geometric intersection test
                self.insert_events[i + 1..]
                    .iter()
                    .take_while(|other_event| other_event.interval.min <= delete_x)
                    .filter(|other_event| {
                        intervals_overlap(current_interval, &other_event.interval)
                    })
                    .for_each(|other_event| {
                        intersection_action(current_interval, &other_event.interval);
                    });
            });
    }
}

/// Helper function to check if two intervals overlap on the `x`-axis
///
/// This determines which segments might geometrically intersect during the sweep.
/// We use `total_cmp` for robust floating-point comparisons to handle edge cases
/// involving very close values.
fn intervals_overlap<T: GeoFloat>(s0: &SweepLineInterval<T>, s1: &SweepLineInterval<T>) -> bool {
    // Check if s0's maximum `x` is greater than or equal to s1's minimum `x`.
    // Implicitly we already know s0.min <= s1.min since we're processing
    // events in sorted order in the sweep algorithm
    debug_assert!(s0.min <= s1.min);
    s0.max.total_cmp(&s1.min).is_ge()
}
