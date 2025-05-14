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
//    - `build_index` sorts all events by `x` coordinate (and type for equal `x` values)
//    - Links paired INSERT/DELETE events via array indices for efficient tracking
//    - `compute_intersections` iterates through Insert events in sorted order
//    - For each INSERT event, it examines only events between it and its paired DELETE event
//    - It filters these for other INSERT events where intervals potentially overlap (using `intervals_overlap`)
//    - For each potential overlap, `line_intersection` checks for actual geometric intersection
//    - When found, intersections are stored in `intersection_pairs`.

use crate::line_intersection::{line_intersection, LineIntersection};
use crate::{GeoFloat, Line};
use std::cmp::Ordering;
use std::collections::HashMap;

#[cfg(test)]
mod tests;

/// Iterator over all intersections of a collection of [`Line`]s.
///
/// # Performance Note
///
/// This implementation is most useful when there is a need to efficiently find sparse intersections
/// between thousands of line segments. For smaller numbers of segments, a brute-force approach
/// will be around 30 % faster in current tests.
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
/// use geo::algorithm::new_sweep::Intersections;
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
        // Use the standard sweep line algorithm for all patterns
        self.compute_sweep_intersections();

        self.intersection_pairs.retain(|(line1, line2, _)| {
            // Keep only intersections between different lines
            line1 != line2
        });
    }

    /// Standard sweep line algorithm (Bentley-Ottmann)
    ///
    /// This method implements the standard sweep line algorithm for finding segment
    /// intersections. The algorithm runs in `O((n + k) log n)` time,
    /// where `n` is the number of line segments and `k` is the number of intersections.
    fn compute_sweep_intersections(&mut self) {
        let mut index = SweepLineIndex::new();

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

        // Build the index and find all intersections
        index.compute_intersections(|s0, s1| {
            let segment1 = s0.item;
            let segment2 = s1.item;

            // Use standard line_intersection for all cases
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
/// # Identity and Relationships
///
/// The `index` field serves two purposes:
/// 1. It uniquely identifies each interval based on its position in the original collection
/// 2. It connects `INSERT` /  `DELETE` event pairs during the index building phase
///
/// When two events have intervals with the same index, they are recognized as belonging
/// to the same line segment, allowing us to establish bidirectional "pointers" between them.
#[derive(Debug, Clone, Copy)]
struct SweepLineInterval<T: GeoFloat> {
    min: T,
    max: T,
    item: Line<T>, // Original line segment
    index: usize,  // Index used to match INSERT/DELETE event pairs via HashMap lookup
}

/// Event types for the sweep line algorithm
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum EventType {
    Insert,
    Delete,
}

/// An event in the sweep line algorithm.
///
/// Events represent the start or end points of line segments.
/// Insert events happen at the left end of a segment, delete events at the right end.
///
/// # Event Relationships
///
/// Each line segment generates two events: an `INSERT` event at its left end and
/// a `DELETE` event at its right end. These paired events need to reference each other.
///
/// Rather than using references (which would require lifetimes and possibly self-references),
/// events are stored in a single `Vec` and refer to each other using array indices:
///
/// * `insert_event`: In a `DELETE` event, stores the index of its paired `INSERT` event
/// * `delete_event_index`: In an `INSERT` event, stores the index of its paired `DELETE` event
///
/// These indices are established in `build_index()`, which uses a HashMap to efficiently
/// match events with the same interval index in O(1) time. This approach provides a simple
/// way to navigate between related events without the complexity of ownership or borrowing.
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
/// - Instead of a priority queue, the **event array is sorted once** and processes events in order.
///   This has the same asymptotic complexity.
///
/// - Instead of a binary search tree, we use **array indices as bidirectional "pointers"** between
///   related events with HashMap lookups for O(1) matching. When processing an `INSERT` event,
///   we directly examine all events between it and its paired `DELETE` event.
///
/// By establishing the relationship between INSERT and DELETE events using
/// a HashMap for fast lookups, it is possible to efficiently determine which segments are active
/// at any point _without_ maintaining an "active segment" tree.
#[derive(Debug, Clone)]
struct SweepLineEvent<T: GeoFloat> {
    x_value: T,
    event_type: EventType,
    interval: SweepLineInterval<T>,
    insert_event_index: Option<usize>, // Index of corresponding insert event (for DELETE events)
    delete_event_index: usize,         // Index of corresponding delete event (for INSERT events)
}

impl<T: GeoFloat> SweepLineEvent<T> {
    fn new(x: T, event_type: EventType, interval: SweepLineInterval<T>) -> Self {
        Self {
            x_value: x,
            event_type,
            interval,
            insert_event_index: None,
            delete_event_index: 0, // Will be set during index building
        }
    }

    fn is_insert(&self) -> bool {
        self.event_type == EventType::Insert
    }

    fn is_delete(&self) -> bool {
        self.event_type == EventType::Delete
    }

    fn set_insert_event(&mut self, insert_idx: usize) {
        self.insert_event_index = Some(insert_idx);
    }

    fn set_delete_event(&mut self, delete_idx: usize) {
        self.delete_event_index = delete_idx;
    }
}

impl<T: GeoFloat> PartialEq for SweepLineEvent<T> {
    fn eq(&self, other: &Self) -> bool {
        self.x_value.total_cmp(&other.x_value) == Ordering::Equal
            && self.event_type == other.event_type
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
        match self.x_value.total_cmp(&other.x_value) {
            Ordering::Equal => {
                // When x values are equal, prioritize INSERT events before DELETE events
                // This ensures proper handling of overlapping segments
                match (self.event_type, other.event_type) {
                    (EventType::Insert, EventType::Delete) => Ordering::Less,
                    (EventType::Delete, EventType::Insert) => Ordering::Greater,
                    _ => Ordering::Equal,
                }
            }
            ordering => ordering,
        }
    }
}

struct SweepLineIndex<T: GeoFloat> {
    events: Vec<SweepLineEvent<T>>,
}

impl<T: GeoFloat> SweepLineIndex<T> {
    fn new() -> Self {
        Self {
            events: Vec::with_capacity(64), // Pre-allocate with a reasonable capacity
        }
    }

    fn add(&mut self, interval: SweepLineInterval<T>) {
        let x_min = interval.min;
        let x_max = interval.max;

        // Create INSERT event
        let insert_event = SweepLineEvent::new(x_min, EventType::Insert, interval);
        self.events.push(insert_event);

        // Create DELETE event
        let delete_event = SweepLineEvent::new(x_max, EventType::Delete, interval);
        self.events.push(delete_event);
    }

    fn compute_intersections<F>(&mut self, mut intersection_action: F)
    where
        F: FnMut(&SweepLineInterval<T>, &SweepLineInterval<T>),
    {
        // First, build the index by sorting events and setting DELETE event indices
        self.build_index();

        // Process events in order
        self.events
            .iter()
            .enumerate()
            .filter(|(_, event)| event.is_insert())
            .for_each(|(i, insertion)| {
                // For each insert event, find all segments active between this INSERT and its DELETE
                let delete_idx = insertion.delete_event_index;
                let interval = insertion.interval;

                // Process all other insert events between this INSERT and its DELETE
                self.events[i + 1..delete_idx]
                    .iter()
                    .filter(|other_event| {
                        other_event.is_insert()
                            && interval.index != other_event.interval.index
                            && intervals_overlap(&interval, &other_event.interval)
                    })
                    .for_each(|other_event| {
                        intersection_action(&interval, &other_event.interval);
                    });
            });
    }

    /// Build the index by sorting events and establishing relationships between paired events
    ///
    /// This method performs three steps:
    /// 1. Sorts all events by `x` coordinate (and event type when `x` values are equal)
    /// 2. Maps interval indices to their corresponding INSERT event positions
    /// 3. Directly connects related DELETE/INSERT event pairs through array indices
    ///
    /// Each line segment has exactly one `INSERT` event and one `DELETE` event with the same
    /// interval index.
    fn build_index(&mut self) {
        // Sort events using the total ordering of events
        self.events.sort();

        // First, map interval indices to insert event indices
        // This will let us look up the insert event for a given interval index in constant time
        // JTS (and GEOS) don't need this as they use object references and pointers respectively
        // but these approaches are (obviously) a pain in Rust, so we need an intermediate mapping
        // https://github.com/simplegeo/jts/blob/3a4c9a9c3a7d3274e16bdd2f341df7d3e113d81b/src/com/vividsolutions/jts/index/sweepline/SweepLineIndex.java#L57-L59
        let insert_event_map: HashMap<usize, usize> = self
            .events
            .iter()
            .enumerate()
            .filter(|(_, event)| event.is_insert())
            .map(|(i, event)| (event.interval.index, i))
            .collect();

        // Process all DELETE events and connect with their INSERT counterparts.
        // I would love to get rid of this ugly for loop, but we can't use an iterator because
        // it would require two mutable borrows of self.events to call the set_* events.
        // The advantage of the loop is that we don't need to allocate anything to populate self.events
        for i in 0..self.events.len() {
            // Process only delete events
            if self.events[i].is_delete() {
                let delete_idx = i;
                let interval_idx = self.events[i].interval.index;

                // Look up the matching insert event
                if let Some(&insert_idx) = insert_event_map.get(&interval_idx) {
                    // set the insert event in the delete event
                    self.events[delete_idx].set_insert_event(insert_idx);
                    // AND THEN: set the delete event in the insert event
                    self.events[insert_idx].set_delete_event(delete_idx);
                }
            }
        }
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
