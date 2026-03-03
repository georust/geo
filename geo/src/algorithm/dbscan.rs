use std::collections::VecDeque;

use crate::{GeoFloat, MultiPoint, Point};

use rstar::RTree;
use rstar::primitives::GeomWithData;

/// Perform [DBSCAN](https://en.wikipedia.org/wiki/DBSCAN) (Density-Based Spatial Clustering of Applications with Noise) clustering on a set of points.
///
/// Based on: Ester, M., Kriegel, H., Sander, J., & Xu, X. (1996). [*A density-based algorithm for discovering clusters in large spatial
/// databases with noise.*](https://dl.acm.org/doi/10.5555/3001460.3001507) In Proceedings of the Second International Conference on Knowledge Discovery and Data Mining (KDD-96), pp 226-231.
///
/// DBSCAN is a density-based clustering algorithm that groups together points that are closely packed together (points with many nearby neighbours),
/// marking as outliers points that lie alone in low-density regions.
///
/// # Parameters
///
/// - `epsilon`: The maximum distance between two points for one to be considered as in the neighbourhood of the other.
///   This is **not** a maximum bound on the distances of points within a cluster. This is the most important DBSCAN parameter to choose appropriately for your dataset
/// - `min_samples`: The number of points (or total weight) in a neighbourhood for a point to be considered as a core point.
///   This includes the point itself. Larger values lead to more conservative clusters.
///
/// # Returns
///
/// A vector of cluster labels, one for each input point, in the same order as the input points:
/// - `Some(cluster_id)`: The point belongs to cluster with ID `cluster_id` (starting from 0)
/// - `None`: The point is considered noise (doesn't belong to any cluster)
///
/// # Algorithm
///
/// The algorithm works as follows:
/// 1. For each unvisited point, find all points within distance `epsilon` (the point's neighbourhood)
/// 2. If the neighbourhood contains at least `min_samples` points (including the point itself), start a new cluster:
///    - Add all points in the neighbourhood to the cluster
///    - For each newly added point, if it's also a core point (has >= `min_samples` neighbours), add its neighbours to the cluster (expansion)
/// 3. If the neighbourhood contains fewer than `min_samples` points, mark the point as noise (it may be reassigned later if it's in another point's neighbourhood)
/// 4. Repeat until all points have been visited
///
/// # Notes
///
/// This implementation uses an R-tree spatial index for efficient neighbourhood queries, and should be O(n log n)
/// for typical cases.
///
/// # Examples
///
/// ## Basic clustering with MultiPoint
///
/// ```
/// use geo::{Dbscan, MultiPoint, point};
///
/// let points = MultiPoint::new(vec![
///     point!(x: 0.0, y: 0.0),
///     point!(x: 1.0, y: 0.0),
///     point!(x: 0.0, y: 1.0),
///     point!(x: 10.0, y: 10.0),
///     point!(x: 11.0, y: 10.0),
///     point!(x: 10.0, y: 11.0),
/// ]);
///
/// let labels = points.dbscan(2.0, 2);
///
/// // Points 0, 1, 2 form one cluster
/// assert_eq!(labels[0], Some(0));
/// assert_eq!(labels[1], Some(0));
/// assert_eq!(labels[2], Some(0));
///
/// // Points 3, 4, 5 form another cluster
/// assert_eq!(labels[3], Some(1));
/// assert_eq!(labels[4], Some(1));
/// assert_eq!(labels[5], Some(1));
/// ```
///
/// ## Detecting noise points
///
/// ```
/// use geo::{Dbscan, point, Point};
///
/// let points = vec![
///     point!(x: 0.0, y: 0.0),
///     point!(x: 1.0, y: 0.0),
///     point!(x: 0.0, y: 1.0),
///     point!(x: 100.0, y: 100.0), // outlier
/// ];
///
/// let labels = points.dbscan(2.0, 2);
///
/// // First three points form a cluster
/// assert_eq!(labels[0], Some(0));
/// assert_eq!(labels[1], Some(0));
/// assert_eq!(labels[2], Some(0));
///
/// // Last point is noise
/// assert_eq!(labels[3], None);
/// ```
///
/// ## Using with a Vec of Points
///
/// ```
/// use geo::{Dbscan, point};
///
/// let points = vec![
///     point!(x: 0.0, y: 0.0),
///     point!(x: 1.0, y: 1.0),
///     point!(x: 1.0, y: 0.0),
///     point!(x: 0.0, y: 1.0),
/// ];
///
/// let labels = points.dbscan(1.5, 3);
///
/// // All points form a single cluster
/// assert!(labels.iter().all(|&label| label == Some(0)));
/// ```
pub trait Dbscan<T>
where
    T: GeoFloat,
{
    /// Perform DBSCAN clustering on the points.
    ///
    /// See the [module-level documentation](self) for details on the algorithm and parameters.
    fn dbscan(&self, epsilson: T, min_samples: usize) -> Vec<Option<usize>>;
}

/// Internal state tracking for a point during DBSCAN
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum PointState {
    Unvisited,
    Noise,
    Queued,
    Clustered(usize),
}

fn dbscan_impl<T>(points: &[Point<T>], epsilon: T, min_samples: usize) -> Vec<Option<usize>>
where
    T: GeoFloat,
{
    let n = points.len();

    // Handle edge cases
    if n == 0 {
        return Vec::new();
    }

    if min_samples == 0 || min_samples > n {
        // No points can form a cluster
        return vec![None; n];
    }

    // Build spatial index with point indices
    let tree = RTree::bulk_load(
        points
            .iter()
            .enumerate()
            .map(|(idx, &point)| GeomWithData::new(point, idx))
            .collect(),
    );

    // Track state of each point
    let mut states = vec![PointState::Unvisited; n];
    let mut cluster_id = 0;

    // Reusable buffers to avoid repeated allocations
    let mut neighbours_buf = Vec::with_capacity(min_samples);
    let mut queue = VecDeque::new();

    // Process each point
    for point_idx in 0..n {
        if states[point_idx] != PointState::Unvisited {
            continue;
        }

        // Reuse queue for each new cluster
        queue.clear();
        queue.extend(
            tree.locate_within_distance(points[point_idx], epsilon * epsilon)
                .map(|geom_with_data| geom_with_data.data),
        );

        if queue.len() < min_samples {
            // Not enough neighbours, mark as noise for now
            states[point_idx] = PointState::Noise;
            continue;
        }

        // Start a new cluster
        states[point_idx] = PointState::Clustered(cluster_id);

        // Mark initial neighbours as queued to prevent re-queuing
        for &neighbour_idx in &queue {
            if matches!(
                states[neighbour_idx],
                PointState::Unvisited | PointState::Noise
            ) {
                states[neighbour_idx] = PointState::Queued;
            }
        }

        // Expand cluster using BFS (iterative to avoid stack overflow)
        while let Some(current_idx) = queue.pop_front() {
            match states[current_idx] {
                PointState::Queued => {
                    // Add to cluster and check if it's a core point
                    states[current_idx] = PointState::Clustered(cluster_id);

                    // Reuse buffer for neighbour queries
                    neighbours_buf.clear();
                    neighbours_buf.extend(
                        tree.locate_within_distance(points[current_idx], epsilon * epsilon)
                            .map(|geom_with_data| geom_with_data.data),
                    );

                    if neighbours_buf.len() >= min_samples {
                        // This is a core point, add its neighbours to the queue for expansion
                        for &neighbour_idx in &neighbours_buf {
                            if matches!(
                                states[neighbour_idx],
                                PointState::Unvisited | PointState::Noise
                            ) {
                                queue.push_back(neighbour_idx);
                                states[neighbour_idx] = PointState::Queued;
                            }
                        }
                    }
                }
                _ => {
                    // Covers both Clustered and unexpected states
                    continue;
                }
            }
        }

        cluster_id += 1;
    }

    // Convert states to output format
    states
        .into_iter()
        .map(|state| match state {
            PointState::Clustered(id) => Some(id),
            _ => None,
        })
        .collect()
}

impl<T> Dbscan<T> for MultiPoint<T>
where
    T: GeoFloat,
{
    fn dbscan(&self, epsilon: T, min_samples: usize) -> Vec<Option<usize>> {
        dbscan_impl(&self.0, epsilon, min_samples)
    }
}

impl<T> Dbscan<T> for &MultiPoint<T>
where
    T: GeoFloat,
{
    fn dbscan(&self, epsilon: T, min_samples: usize) -> Vec<Option<usize>> {
        dbscan_impl(&self.0, epsilon, min_samples)
    }
}

impl<T> Dbscan<T> for [Point<T>]
where
    T: GeoFloat,
{
    fn dbscan(&self, epsilon: T, min_samples: usize) -> Vec<Option<usize>> {
        dbscan_impl(self, epsilon, min_samples)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::point;

    #[test]
    fn test_dbscan_empty() {
        let points: Vec<Point<f64>> = vec![];
        let labels = points.dbscan(1.0, 2);
        assert_eq!(labels.len(), 0);
    }

    #[test]
    fn test_dbscan_single_point() {
        let points = [point!(x: 0.0, y: 0.0)];
        let labels = points.dbscan(1.0, 2);
        assert_eq!(labels, vec![None]); // Single point cannot form a cluster with min_samples=2
    }

    #[test]
    fn test_dbscan_all_noise() {
        let points = [
            point!(x: 0.0, y: 0.0),
            point!(x: 10.0, y: 10.0),
            point!(x: 20.0, y: 20.0),
        ];
        let labels = points.dbscan(1.0, 2);
        assert_eq!(labels, vec![None, None, None]);
    }

    #[test]
    fn test_dbscan_single_cluster() {
        let points = [
            point!(x: 0.0, y: 0.0),
            point!(x: 1.0, y: 0.0),
            point!(x: 0.0, y: 1.0),
            point!(x: 1.0, y: 1.0),
        ];
        let labels = points.dbscan(1.5, 2);

        // All points should be in the same cluster
        assert_eq!(labels[0], Some(0));
        assert_eq!(labels[1], Some(0));
        assert_eq!(labels[2], Some(0));
        assert_eq!(labels[3], Some(0));
    }

    #[test]
    fn test_dbscan_two_clusters() {
        let points = [
            // Cluster 1
            point!(x: 0.0, y: 0.0),
            point!(x: 1.0, y: 0.0),
            point!(x: 0.0, y: 1.0),
            // Cluster 2
            point!(x: 10.0, y: 10.0),
            point!(x: 11.0, y: 10.0),
            point!(x: 10.0, y: 11.0),
        ];
        let labels = points.dbscan(2.0, 2);

        // First three points in cluster 0
        assert_eq!(labels[0], Some(0));
        assert_eq!(labels[1], Some(0));
        assert_eq!(labels[2], Some(0));

        // Last three points in cluster 1
        assert_eq!(labels[3], Some(1));
        assert_eq!(labels[4], Some(1));
        assert_eq!(labels[5], Some(1));
    }

    #[test]
    fn test_dbscan_with_noise() {
        let points = [
            // Cluster
            point!(x: 0.0, y: 0.0),
            point!(x: 1.0, y: 0.0),
            point!(x: 0.0, y: 1.0),
            // Noise
            point!(x: 100.0, y: 100.0),
        ];
        let labels = points.dbscan(2.0, 2);

        // First three points in a cluster
        assert_eq!(labels[0], Some(0));
        assert_eq!(labels[1], Some(0));
        assert_eq!(labels[2], Some(0));

        // Last point is noise
        assert_eq!(labels[3], None);
    }

    #[test]
    fn test_dbscan_border_points() {
        // Test that border points (non-core points in a cluster) are correctly assigned
        let points = [
            // Core points
            point!(x: 0.0, y: 0.0),
            point!(x: 1.0, y: 0.0),
            point!(x: 0.5, y: 0.5),
            // Border point (only reachable from core points)
            point!(x: 2.0, y: 0.0),
        ];
        let labels = points.dbscan(1.5, 2);

        // All points should be in the same cluster
        assert_eq!(labels[0], Some(0));
        assert_eq!(labels[1], Some(0));
        assert_eq!(labels[2], Some(0));
        assert_eq!(labels[3], Some(0));
    }

    #[test]
    fn test_dbscan_multipoint() {
        let points = MultiPoint::new(vec![
            point!(x: 0.0, y: 0.0),
            point!(x: 1.0, y: 0.0),
            point!(x: 0.0, y: 1.0),
            point!(x: 10.0, y: 10.0),
            point!(x: 11.0, y: 10.0),
            point!(x: 10.0, y: 11.0),
        ]);

        let labels = points.dbscan(2.0, 2);

        // Two clusters
        assert_eq!(labels[0], Some(0));
        assert_eq!(labels[1], Some(0));
        assert_eq!(labels[2], Some(0));
        assert_eq!(labels[3], Some(1));
        assert_eq!(labels[4], Some(1));
        assert_eq!(labels[5], Some(1));
    }

    #[test]
    fn test_dbscan_min_samples_includes_self() {
        // With min_samples=1, every point should form its own cluster
        // (since each point is in its own neighbourhood)
        let points = [
            point!(x: 0.0, y: 0.0),
            point!(x: 10.0, y: 10.0),
            point!(x: 20.0, y: 20.0),
        ];
        let labels = points.dbscan(1.0, 1);

        // Each point forms its own cluster
        assert_eq!(labels[0], Some(0));
        assert_eq!(labels[1], Some(1));
        assert_eq!(labels[2], Some(2));
    }

    #[test]
    fn test_dbscan_varying_density() {
        // Test with clusters of different densities
        let points = [
            // Dense cluster
            point!(x: 0.0, y: 0.0),
            point!(x: 0.5, y: 0.0),
            point!(x: 1.0, y: 0.0),
            point!(x: 0.5, y: 0.5),
            // Sparse cluster (points are further apart)
            point!(x: 10.0, y: 10.0),
            point!(x: 12.0, y: 10.0),
            point!(x: 11.0, y: 12.0),
        ];
        // Use epsilon=2.5 to capture both dense and sparse clusters
        let labels = points.dbscan(2.5, 2);

        // Dense cluster
        assert_eq!(labels[0], Some(0));
        assert_eq!(labels[1], Some(0));
        assert_eq!(labels[2], Some(0));
        assert_eq!(labels[3], Some(0));

        // Sparse cluster
        assert_eq!(labels[4], Some(1));
        assert_eq!(labels[5], Some(1));
        assert_eq!(labels[6], Some(1));
    }

    #[test]
    fn test_dbscan_min_samples_too_large() {
        let points = [
            point!(x: 0.0, y: 0.0),
            point!(x: 1.0, y: 0.0),
            point!(x: 0.0, y: 1.0),
        ];
        // min_samples larger than total points
        let labels = points.dbscan(2.0, 10);
        assert_eq!(labels, vec![None, None, None]);
    }

    #[test]
    fn test_dbscan_min_samples_zero() {
        let points = [point!(x: 0.0, y: 0.0), point!(x: 1.0, y: 0.0)];
        // min_samples = 0 should result in all noise
        let labels = points.dbscan(2.0, 0);
        assert_eq!(labels, vec![None, None]);
    }

    #[test]
    fn test_dbscan_identical_points() {
        // Test with duplicate points at the same location
        let points = [
            point!(x: 0.0, y: 0.0),
            point!(x: 0.0, y: 0.0),
            point!(x: 0.0, y: 0.0),
        ];
        let labels = points.dbscan(0.1, 2);

        // All points should cluster together (they're at the same location)
        assert_eq!(labels[0], Some(0));
        assert_eq!(labels[1], Some(0));
        assert_eq!(labels[2], Some(0));
    }

    #[test]
    fn test_dbscan_linear_cluster() {
        // Test a linear arrangement of points
        let points = [
            point!(x: 0.0, y: 0.0),
            point!(x: 1.0, y: 0.0),
            point!(x: 2.0, y: 0.0),
            point!(x: 3.0, y: 0.0),
            point!(x: 4.0, y: 0.0),
        ];
        let labels = points.dbscan(1.5, 2);

        // All points should form a single cluster
        assert!(labels.iter().all(|&label| label == Some(0)));
    }
}
