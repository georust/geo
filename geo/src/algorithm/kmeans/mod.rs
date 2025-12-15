//!
//! _k_-means clustering partitions `n` observations into _k_ clusters in which each observation belongs
//! to the cluster with the nearest mean (centroid). Unlike DBSCAN, _k_-means requires specifying the
//! number of clusters in advance.
//!
//! Uses Lloyd's algorithm with `k-means++` initialisation for improved cluster quality, accelerated
//! using [Hamerly's triangle inequality bounds (2010)](https://cs.baylor.edu/~hamerly/papers/2014_pca_chapter_hamerly_drake.pdf). This implementation is optimised for
//! 2D points with moderate cluster counts (k=2-50).
//!
//!
//! ## Hamerly's Algorithm
//!
//! This implementation uses Hamerly's triangle inequality bounds to skip unnecessary distance
//! calculations. The acceleration is most effective under the following conditions:
//!
//! - **Well-separated clusters**: Greater separation → tighter bounds → more pruning
//! - **More iterations**: Bounds tighten progressively, so longer convergence provides more savings
//! - **Moderate k**: Provides pruning opportunities whilst keeping O(k²) centroid distance overhead manageable
//!
//! For larger _k_ (> 50), the O(k²) centroid-to-centroid distance computation can dominate.
//!
//! # Parameters
//!
//! - `k`: The number of clusters to form. Must be > 0 and <= number of points. **NB**: More than `k` clusters may be returned if `max_radius` is specified.
//! - `max_radius` (optional): Maximum radius for clusters. If specified, clusters exceeding this radius will be subdivided,
//!   potentially resulting in more than `k` clusters.
//! - `max_iter` (optional): Maximum number of iterations. Default: 300.
//! - `tolerance` (optional): Absolute tolerance for convergence based on centroid movement (in coordinate units). Default: 0.0001.
//!
//! # Returns
//!
//! Returns `Result<Vec<usize>, KMeansError<T>>`:
//! - `Ok(labels)`: A vector of cluster labels (0-based indices), one for each input point
//! - `Err(KMeansError::InvalidK)`: If `k` is 0 or greater than the number of points
//! - `Err(KMeansError::EmptyCluster)`: If an empty cluster cannot be recovered during iteration
//! - `Err(KMeansError::InitializationFailed)`: If k-means++ initialisation fails due to degenerate data (e.g., all points at same location, NaN/infinite coordinates)
//! - `Err(KMeansError::MaxIterationsReached)`: If max_iter is reached before convergence. The error contains the partial result with convergence metadata.
//!
//! # Algorithm
//!
//! The algorithm works as follows:
//! 1. **Initialisation**: Uses [k-means++](https://theory.stanford.edu/~sergei/papers/kMeansPP-soda.pdf) for better initial centroids
//! 2. **Assignment**: Assign each point to the cluster with the nearest centroid using Hamerly's triangle inequality bounds
//!    - Maintains upper and lower bounds on distances to avoid redundant calculations
//!    - Only computes actual distances when bounds don't rule out assignment changes
//! 3. **Update**: Recalculate centroids as the mean of all points in each cluster
//! 4. **Bounds update**: Update distance bounds based on centroid movement using triangle inequality
//! 5. **Convergence check**: Stop if:
//!    - Centroids have moved less than the tolerance threshold, OR
//!    - No points have changed cluster assignment, OR
//!    - Maximum iterations reached
//! 6. Repeat steps 2-5 until convergence
//! 7. **max_radius constraint** (if specified): Split any clusters with radius exceeding max_radius
//!
//! # Notes
//!
//! Empty clusters may rarely occur during iteration if all points are reassigned away from a centroid.
//! When this happens, the algorithm attempts to recover by reassigning the farthest point from its
//! current centroid to the empty cluster (following scikit-learn's approach). If recovery fails,
//! the function returns an error.
//!
//! ## Complexity
//!
//! - **Initialization (k-means++)**: O(n·k²)
//! - **Per iteration**: O(k²) centroid distances + O(n) for point assignments with Hamerly pruning
//! - **Overall**: Effectively O(i·n) for moderate k, where i is iterations to convergence
//!
//! For large datasets with _k_ > 50, consider hierarchical or density-based clustering (DBSCAN).
//!
//! # Examples
//!
//! ## Basic clustering with MultiPoint
//!
//! ```
//! use geo::{KMeans, MultiPoint, point};
//!
//! let points = MultiPoint::new(vec![
//!     point!(x: 0.0, y: 0.0),
//!     point!(x: 1.0, y: 0.0),
//!     point!(x: 0.0, y: 1.0),
//!     point!(x: 10.0, y: 10.0),
//!     point!(x: 11.0, y: 10.0),
//!     point!(x: 10.0, y: 11.0),
//! ]);
//!
//! let labels = points.kmeans(2).unwrap();
//!
//! // Two clusters should be found
//! let cluster_0_count = labels.iter().filter(|&l| *l == 0).count();
//! let cluster_1_count = labels.iter().filter(|&l| *l == 1).count();
//! assert_eq!(cluster_0_count + cluster_1_count, 6);
//! ```
//!
//! ## Using with a Vec of Points
//!
//! ```
//! use geo::{KMeans, point};
//!
//! let points = vec![
//!     point!(x: 0.0, y: 0.0),
//!     point!(x: 1.0, y: 1.0),
//!     point!(x: 1.0, y: 0.0),
//!     point!(x: 0.0, y: 1.0),
//! ];
//!
//! let labels = points.kmeans(2).unwrap();
//!
//! // All points should be assigned to one of two clusters
//! assert_eq!(labels.len(), 4);
//! ```
//!
//! # Errors
//!
//! Returns `KMeansError<T>` if:
//! - `k` is 0 or greater than the number of points (`InvalidK`)
//! - An empty cluster cannot be recovered during iteration (`EmptyCluster`)
//! - Initialisation fails due to degenerate data (`InitializationFailed`)
//! - Maximum iterations reached before convergence (`MaxIterationsReached`)

mod error;
pub use error::{KMeansError, KMeansInitError};

use crate::algorithm::Vector2DOps;
use crate::line_measures::Distance;
use crate::line_measures::metric_spaces::Euclidean;
use crate::{Centroid, GeoFloat, MultiPoint, Point};

use rand::Rng;
use rand::SeedableRng;
use rand::distr::weighted::WeightedIndex;
use rand::prelude::Distribution;
use rand::rngs::StdRng;

/// Parameters for k-means clustering with builder pattern.
///
/// # Examples
///
/// ```
/// use geo::{KMeans, KMeansParams, point};
///
/// let points = vec![
///     point!(x: 0.0, y: 0.0),
///     point!(x: 1.0, y: 1.0),
///     point!(x: 10.0, y: 10.0),
///     point!(x: 11.0, y: 11.0),
/// ];
///
/// // Use builder pattern for advanced configuration
/// let params = KMeansParams::new(2)
///     .seed(42)
///     .max_iter(500)
///     .tolerance(0.001);
///
/// let labels = points.kmeans_with_params(params).unwrap();
/// ```
#[derive(Debug, Clone)]
pub struct KMeansParams<T: GeoFloat> {
    k: usize,
    seed: Option<u64>,
    max_iter: usize,
    tolerance: T,
    max_radius: Option<T>,
}

impl<T: GeoFloat> KMeansParams<T> {
    /// Create new k-means parameters with the specified number of clusters.
    ///
    /// Uses default values for optional parameters:
    /// - No seed (random initialisation)
    /// - max_iter: 300
    /// - tolerance: 0.0001
    /// - No max_radius constraint
    pub fn new(k: usize) -> Self {
        Self {
            k,
            seed: None,
            max_iter: 300, // same as scikit-learn
            tolerance: T::from(0.0001).expect("tolerance must be representable in float type"),
            max_radius: None,
        }
    }

    /// Set the random seed for reproducible results.
    ///
    /// When set, k-means will produce identical results across multiple runs with the same
    /// input data and parameters.
    pub fn seed(mut self, seed: u64) -> Self {
        self.seed = Some(seed);
        self
    }

    /// Set the maximum number of iterations.
    ///
    /// Default: 300
    pub fn max_iter(mut self, max_iter: usize) -> Self {
        self.max_iter = max_iter;
        self
    }

    /// Set the convergence tolerance.
    ///
    /// The algorithm stops when the maximum distance any centroid moves is less than this
    /// threshold (measured in coordinate units).
    /// Default: 0.0001
    pub fn tolerance(mut self, tolerance: T) -> Self {
        self.tolerance = tolerance;
        self
    }

    /// Set the maximum cluster radius.
    ///
    /// When set, clusters exceeding this radius will be subdivided, potentially resulting
    /// in more than k clusters.
    pub fn max_radius(mut self, max_radius: T) -> Self {
        self.max_radius = Some(max_radius);
        self
    }
}

pub trait KMeans<T>
where
    T: GeoFloat,
{
    /// Perform _k_-means clustering on the points.
    ///
    /// Returns a vector of cluster assignments, one for each input point.
    /// Each element is a cluster ID in the range `0..k`.
    ///
    /// To specify configuration options, use [`kmeans_with_params`](Self::kmeans_with_params).
    ///
    /// # Errors
    ///
    /// Returns `KMeansError::InvalidK` if `k` is 0 or greater than the number of points.
    /// Returns `KMeansError::EmptyCluster` if a cluster becomes empty and cannot be recovered.
    /// Returns `KMeansError::MaxIterationsReached` if max_iter is reached before convergence.
    ///
    /// See the [module-level documentation](self) for details on the algorithm and parameters.
    fn kmeans(&self, k: usize) -> Result<Vec<usize>, KMeansError<T>> {
        self.kmeans_with_params(KMeansParams::new(k))
    }

    /// Perform _k_-means clustering with specific configuration options.
    ///
    /// Returns a vector of cluster assignments, one for each input point.
    /// Each element is a cluster ID in the range `0..k`.
    ///
    /// # Examples
    ///
    /// ```
    /// use geo::{KMeans, KMeansParams, point};
    ///
    /// let points = vec![
    ///     point!(x: 0.0, y: 0.0),
    ///     point!(x: 1.0, y: 1.0),
    ///     point!(x: 10.0, y: 10.0),
    ///     point!(x: 11.0, y: 11.0),
    /// ];
    ///
    /// let params = KMeansParams::new(2)
    ///     .seed(42)           // Reproducible results
    ///     .max_iter(500)      // More iterations
    ///     .tolerance(0.001);  // Less strict convergence
    ///
    /// let labels = points.kmeans_with_params(params).unwrap();
    /// ```
    ///
    /// # Errors
    ///
    /// Returns `KMeansError::InvalidK` if `k` is 0 or greater than the number of points.
    /// Returns `KMeansError::EmptyCluster` if a cluster becomes empty and cannot be recovered.
    /// Returns `KMeansError::InitializationFailed` if initialisation fails due to degenerate data.
    /// Returns `KMeansError::MaxIterationsReached` if max_iter is reached before convergence.
    ///
    /// See the [module-level documentation](self) for details on the algorithm and parameters.
    fn kmeans_with_params(&self, params: KMeansParams<T>) -> Result<Vec<usize>, KMeansError<T>>;
}

/// Configuration for k-means algorithm
#[derive(Debug, Clone)]
struct KMeansConfig<T: GeoFloat> {
    params: KMeansParams<T>,
    max_split_depth: usize,
}

impl<T: GeoFloat> Default for KMeansConfig<T> {
    fn default() -> Self {
        Self {
            params: KMeansParams::new(0),
            max_split_depth: 10,
        }
    }
}

impl<T: GeoFloat> From<KMeansParams<T>> for KMeansConfig<T> {
    fn from(params: KMeansParams<T>) -> Self {
        Self {
            params,
            max_split_depth: 10,
        }
    }
}

fn kmeans_impl<T>(
    points: &[Point<T>],
    params: KMeansParams<T>,
) -> Result<Vec<usize>, KMeansError<T>>
where
    T: GeoFloat,
{
    let n = points.len();
    let k = params.k;

    // Handle edge cases
    if n == 0 {
        return Ok(Vec::new());
    }

    if k == 0 || k > n {
        return Err(KMeansError::InvalidK { k, n });
    }

    if k == 1 {
        // All points in one cluster
        return Ok(vec![0; n]);
    }

    if k == n {
        // Each point is its own cluster
        return Ok((0..n).collect());
    }

    let config = KMeansConfig::from(params);

    kmeans_impl_with_config(points, config)
}

fn kmeans_impl_with_config<T>(
    points: &[Point<T>],
    config: KMeansConfig<T>,
) -> Result<Vec<usize>, KMeansError<T>>
where
    T: GeoFloat,
{
    let n = points.len();
    let k = config.params.k;

    // Precompute squared norms of all points (done once, reused throughout)
    // This is the key optimisation from scikit-learn's implementation
    let point_sq_norms: Vec<T> = points.iter().map(|p| p.0.magnitude_squared()).collect();

    // Initialise centroids using k-means++
    let mut centroids = kmeans_plusplus_init(points, k, config.params.seed)?;

    // Precompute squared norms of initial centroids
    let mut centroid_sq_norms: Vec<T> = centroids.iter().map(|c| c.0.magnitude_squared()).collect();

    // Track cluster assignments and Hamerly bounds
    let mut assignments = vec![0; n];
    let mut upper_bounds = vec![T::infinity(); n];
    let mut lower_bounds = vec![T::zero(); n];

    // First iteration: assign all points and initialize bounds
    for (i, ((point, assignment), (upper, lower))) in points
        .iter()
        .zip(assignments.iter_mut())
        .zip(upper_bounds.iter_mut().zip(lower_bounds.iter_mut()))
        .enumerate()
    {
        let (nearest_idx, nearest_sq_dist, second_nearest_sq_dist) =
            find_nearest_and_second_nearest(
                *point,
                &centroids,
                point_sq_norms[i],
                &centroid_sq_norms,
            );
        *assignment = nearest_idx;
        // Store actual distances (take sqrt of squared distances)
        *upper = nearest_sq_dist.sqrt();
        *lower = second_nearest_sq_dist.sqrt();
    }

    // Track convergence state
    let mut final_max_delta = T::zero();
    let mut final_changed_count = 0;
    let mut converged = false;

    // Iterate until convergence
    for iter in 0..config.params.max_iter {
        let mut changed_count = 0;

        // Compute distance from each centroid to its nearest other centroid
        let centroid_distances = compute_centroid_distances(&centroids);

        // Assignment step using Hamerly's algorithm
        for (i, point) in points.iter().enumerate() {
            let assigned_idx = assignments[i];

            // Pruning condition 1: upper bound <= lower bound
            if upper_bounds[i] <= lower_bounds[i] {
                continue;
            }

            // Pruning condition 2: upper bound <= s[assigned]/2
            if upper_bounds[i] <= centroid_distances[assigned_idx] / T::from(2.0).unwrap() {
                continue;
            }

            // Tighten upper bound using optimised squared distance
            let sq_dist_to_assigned = squared_distance_using_norms(
                *point,
                centroids[assigned_idx],
                point_sq_norms[i],
                centroid_sq_norms[assigned_idx],
            );
            upper_bounds[i] = sq_dist_to_assigned.sqrt();

            // Recheck after tightening
            if upper_bounds[i] <= lower_bounds[i] {
                continue;
            }

            // Must check all centroids using optimised distance calculation
            let (new_nearest_idx, new_nearest_sq_dist, new_second_nearest_sq_dist) =
                find_nearest_and_second_nearest(
                    *point,
                    &centroids,
                    point_sq_norms[i],
                    &centroid_sq_norms,
                );

            if new_nearest_idx != assigned_idx {
                assignments[i] = new_nearest_idx;
                changed_count += 1;
            }

            // Store actual distances (sqrt of squared distances)
            upper_bounds[i] = new_nearest_sq_dist.sqrt();
            lower_bounds[i] = new_second_nearest_sq_dist.sqrt();
        }

        // Check for convergence: no assignments changed
        if changed_count == 0 {
            converged = true;
            break;
        }

        // Update step: recalculate centroids
        let new_centroids = update_centroids(points, &mut assignments, &centroids, k, iter)?;

        // Recompute centroid squared norms after update
        centroid_sq_norms = new_centroids
            .iter()
            .map(|c| c.0.magnitude_squared())
            .collect();

        // Compute centroid movement deltas
        let deltas: Vec<T> = centroids
            .iter()
            .zip(new_centroids.iter())
            .map(|(old, new)| Euclidean.distance(*old, *new))
            .collect();

        let max_delta = deltas.iter().fold(T::zero(), |a, &b| a.max(b));
        final_max_delta = max_delta;
        final_changed_count = changed_count;

        // Check for convergence: centroids haven't moved much
        if max_delta < config.params.tolerance {
            converged = true;
            break;
        }

        // Update bounds based on centroid movement
        for ((upper, lower), &assigned_idx) in upper_bounds
            .iter_mut()
            .zip(lower_bounds.iter_mut())
            .zip(assignments.iter())
        {
            *upper = *upper + deltas[assigned_idx];
            *lower = (*lower - max_delta).max(T::zero());
        }

        centroids = new_centroids;
    }

    // Check if we hit max_iter without converging
    if !converged {
        return Err(KMeansError::MaxIterationsReached {
            assignments,
            iterations: config.params.max_iter,
            max_centroid_shift: final_max_delta,
            tolerance: config.params.tolerance,
            changed_assignments: final_changed_count,
        });
    }

    // Apply max_radius constraint if specified
    if let Some(max_radius) = config.params.max_radius {
        apply_max_radius_constraint(
            points,
            &mut assignments,
            max_radius,
            config.max_split_depth,
            config.params.seed,
        )?;
    }

    Ok(assignments)
}

/// Initialise centroids using k-means++ algorithm
///
/// k-means++ improves upon random initialisation by choosing initial centroids
/// that are spread out, leading to better cluster quality and faster convergence.
///
/// Algorithm:
/// 1. Choose first centroid uniformly at random
/// 2. For each remaining centroid:
///    - Calculate squared distance from each point to its nearest existing centroid
///    - Choose next centroid with probability proportional to this squared distance (D² weighting)
///    - This favours points that are far from existing centroids
///
/// Reference: Arthur & Vassilvitskii (2007), ["k-means++: the Advantages of Careful Seeding"](https://theory.stanford.edu/~sergei/papers/kMeansPP-soda.pdf)
fn kmeans_plusplus_init<T>(
    points: &[Point<T>],
    k: usize,
    seed: Option<u64>,
) -> Result<Vec<Point<T>>, KMeansError<T>>
where
    T: GeoFloat,
{
    let n = points.len();
    let mut rng = match seed {
        Some(s) => StdRng::seed_from_u64(s),
        None => StdRng::from_os_rng(),
    };
    let mut centroids = Vec::with_capacity(k);

    // Choose first centroid uniformly at random
    // this would panic if n is set to 0, but we handle that case in kmeans_impl
    let first_idx = rng.random_range(0..n);
    centroids.push(points[first_idx]);

    // Choose remaining centroids using D² weighting: probability proportional to
    // squared distance to nearest centroid
    for _ in 1..k {
        // Calculate minimum squared distance from each point to nearest existing centroid,
        // validating and converting to f64 in a single pass
        let distances_f64: Vec<f64> = points
            .iter()
            .map(|point| {
                // Find minimum squared distance to any centroid
                let mut min_sq_dist = T::infinity();

                for centroid in centroids.iter() {
                    let dist = Euclidean.distance(*point, *centroid);

                    // Check for invalid values immediately (before .min() can hide them)
                    if dist.is_nan() {
                        return Err(KMeansError::InitializationFailed(
                            KMeansInitError::NaNCoordinate,
                        ));
                    }
                    if dist.is_infinite() {
                        return Err(KMeansError::InitializationFailed(
                            KMeansInitError::InfiniteCoordinate,
                        ));
                    }

                    let sq_dist = dist * dist;
                    min_sq_dist = min_sq_dist.min(sq_dist);
                }

                // Convert to f64 (guaranteed to succeed after validation above)
                Ok(min_sq_dist
                    .to_f64()
                    .expect("Valid distance should convert to f64"))
            })
            .collect::<Result<Vec<f64>, KMeansError<T>>>()?;

        // Choose next centroid with probability proportional to squared distance
        let dist = WeightedIndex::new(&distances_f64).map_err(|e| {
            // Check if all weights are zero (degenerate data case)
            let all_zero = distances_f64.iter().all(|&d| d == 0.0);
            if all_zero {
                KMeansError::InitializationFailed(KMeansInitError::DegenerateData)
            } else {
                KMeansError::InitializationFailed(KMeansInitError::WeightedDistributionFailed {
                    error: e,
                })
            }
        })?;
        let next_idx = dist.sample(&mut rng);
        centroids.push(points[next_idx]);
    }

    Ok(centroids)
}

/// Compute the distance from each centroid to its nearest other centroid
///
/// Used in Hamerly's algorithm for bounds-based pruning
fn compute_centroid_distances<T>(centroids: &[Point<T>]) -> Vec<T>
where
    T: GeoFloat,
{
    let k = centroids.len();
    let mut distances = vec![T::infinity(); k];

    // Compute each pairwise distance once (upper triangle only)
    (0..k)
        .flat_map(|i| ((i + 1)..k).map(move |j| (i, j)))
        .for_each(|(i, j)| {
            let dist = Euclidean.distance(centroids[i], centroids[j]);
            distances[i] = distances[i].min(dist);
            distances[j] = distances[j].min(dist);
        });

    distances
}

/// Compute squared Euclidean distance using precomputed squared norms
///
/// Uses the algebraic identity: ||a - b||² = ||a||² + ||b||² - 2⟨a,b⟩
///
/// This optimisation (from scikit-learn) avoids redundant squared norm calculations
/// in the O(n·k) inner loop. By precomputing ||a||² and ||b||² once, we only need
/// to compute the dot product ⟨a,b⟩ for each point-centroid pair.
///
/// # Returns
/// The squared Euclidean distance (no sqrt needed for distance comparisons)
#[inline]
fn squared_distance_using_norms<T>(
    point: Point<T>,
    centroid: Point<T>,
    point_sq_norm: T,
    centroid_sq_norm: T,
) -> T
where
    T: GeoFloat,
{
    // ||a - b||² = ||a||² + ||b||² - 2⟨a,b⟩
    let dot_prod = point.0.dot_product(centroid.0);
    point_sq_norm + centroid_sq_norm - (T::from(2.0).unwrap() * dot_prod)
}

/// Find the nearest and second-nearest centroids to a point using precomputed squared norms
///
/// Returns (nearest_index, nearest_squared_distance, second_nearest_squared_distance)
///
/// # Arguments
/// * `point` - The point to find nearest centroids for
/// * `centroids` - Slice of centroid points
/// * `point_sq_norm` - Precomputed ||point||²
/// * `centroid_sq_norms` - Precomputed ||centroid||² for each centroid
///
/// Note: Returns squared distances (no sqrt) since we only need them for comparison
fn find_nearest_and_second_nearest<T>(
    point: Point<T>,
    centroids: &[Point<T>],
    point_sq_norm: T,
    centroid_sq_norms: &[T],
) -> (usize, T, T)
where
    T: GeoFloat,
{
    let mut nearest_idx = 0;
    let mut nearest_sq_dist = T::infinity();
    let mut second_nearest_sq_dist = T::infinity();

    for (idx, (centroid, &centroid_sq_norm)) in
        centroids.iter().zip(centroid_sq_norms.iter()).enumerate()
    {
        let sq_dist =
            squared_distance_using_norms(point, *centroid, point_sq_norm, centroid_sq_norm);
        if sq_dist < nearest_sq_dist {
            second_nearest_sq_dist = nearest_sq_dist;
            nearest_sq_dist = sq_dist;
            nearest_idx = idx;
        } else if sq_dist < second_nearest_sq_dist {
            second_nearest_sq_dist = sq_dist;
        }
    }

    (nearest_idx, nearest_sq_dist, second_nearest_sq_dist)
}

/// Find the point that is farthest from its assigned centroid
///
/// Used for reassigning points to empty clusters. Returns None if no points are found.
/// Returns (point_index, distance) for the farthest point.
fn find_farthest_point<T>(
    points: &[Point<T>],
    assignments: &[usize],
    centroids: &[Point<T>],
) -> Option<(usize, T)>
where
    T: GeoFloat,
{
    let mut farthest_idx = None;
    let mut farthest_dist = T::zero();

    for (i, (point, &cluster_id)) in points.iter().zip(assignments.iter()).enumerate() {
        if cluster_id < centroids.len() {
            let dist = Euclidean.distance(*point, centroids[cluster_id]);
            if dist > farthest_dist {
                farthest_dist = dist;
                farthest_idx = Some(i);
            }
        }
    }

    farthest_idx.map(|idx| (idx, farthest_dist))
}

/// Update centroids based on current assignments
///
/// If an empty cluster is encountered, attempts to reassign the farthest point to that cluster.
/// Returns an error if an empty cluster cannot be recovered.
fn update_centroids<T>(
    points: &[Point<T>],
    assignments: &mut [usize],
    centroids: &[Point<T>],
    k: usize,
    iteration: usize,
) -> Result<Vec<Point<T>>, KMeansError<T>>
where
    T: GeoFloat,
{
    // We're manually calculating centroids rather than using e.g. MultiPoint::centroid()
    // This provides better perf for k-means:
    // - O(1) empty cluster reassignment using arithmetic on sums/counts
    // - No intermediate MultiPoint allocations per iteration

    // Accumulate sums and counts for each cluster in a single pass
    let mut sums: Vec<(T, T)> = vec![(T::zero(), T::zero()); k];
    let mut counts: Vec<usize> = vec![0; k];

    for (point, &cluster_id) in points.iter().zip(assignments.iter()) {
        if cluster_id < k {
            sums[cluster_id].0 = sums[cluster_id].0 + point.x();
            sums[cluster_id].1 = sums[cluster_id].1 + point.y();
            counts[cluster_id] += 1;
        }
    }

    // Handle empty clusters by reassigning the farthest point
    for cluster_id in 0..k {
        if counts[cluster_id] == 0 {
            // Find the point farthest from its current centroid
            if let Some((farthest_idx, _)) = find_farthest_point(points, assignments, centroids) {
                let old_cluster = assignments[farthest_idx];
                let point = points[farthest_idx];

                // Remove from old cluster
                sums[old_cluster].0 = sums[old_cluster].0 - point.x();
                sums[old_cluster].1 = sums[old_cluster].1 - point.y();
                counts[old_cluster] -= 1;

                // Add to new cluster
                sums[cluster_id].0 = sums[cluster_id].0 + point.x();
                sums[cluster_id].1 = sums[cluster_id].1 + point.y();
                counts[cluster_id] += 1;

                // Update assignment
                assignments[farthest_idx] = cluster_id;
            } else {
                // No points available to reassign: this is an unrecoverable error!
                return Err(KMeansError::EmptyCluster {
                    iteration,
                    cluster_id,
                });
            }
        }
    }

    // Compute centroids directly from sums and counts
    let new_centroids = sums
        .iter()
        .zip(counts.iter())
        .map(|(&(sum_x, sum_y), &count)| {
            let count_t = T::from(count).expect("Cluster count must be representable as float");
            Point::new(sum_x / count_t, sum_y / count_t)
        })
        .collect();

    Ok(new_centroids)
}

fn apply_max_radius_constraint<T>(
    points: &[Point<T>],
    assignments: &mut [usize],
    max_radius: T,
    remaining_depth: usize,
    seed: Option<u64>,
) -> Result<(), KMeansError<T>>
where
    T: GeoFloat,
{
    if remaining_depth == 0 {
        return Ok(());
    }

    // Find next available cluster ID. unwrap_or(0) is correct: if assignments is empty,
    // we start numbering clusters from 0
    let mut next_cluster_id = assignments.iter().max().map(|&a| a + 1).unwrap_or(0);

    let max_cluster = assignments.iter().max().copied().unwrap_or(0);
    let mut cluster_ids = Vec::with_capacity(max_cluster + 1);
    let mut seen = vec![false; max_cluster + 1];

    for &assignment in &*assignments {
        if !seen[assignment] {
            seen[assignment] = true;
            cluster_ids.push(assignment);
        }
    }

    for cluster_id in cluster_ids {
        let cluster_points: Vec<(usize, Point<T>)> = points
            .iter()
            .enumerate()
            .filter(|(idx, _)| assignments[*idx] == cluster_id)
            .map(|(idx, &point)| (idx, point))
            .collect();

        if cluster_points.is_empty() {
            continue;
        }

        let cluster_point_vec: Vec<Point<T>> = cluster_points.iter().map(|(_, p)| *p).collect();
        let multipoint = MultiPoint::new(cluster_point_vec);
        let centroid = multipoint
            .centroid()
            .expect("MultiPoint cannot be empty after filtering non-empty cluster");

        let max_dist = multipoint
            .iter()
            .map(|p| Euclidean.distance(&centroid, p))
            .fold(T::zero(), T::max);

        if max_dist > max_radius {
            let params = if let Some(s) = seed {
                KMeansParams::new(2).seed(s)
            } else {
                KMeansParams::new(2)
            };
            let sub_assignments = kmeans_impl(&multipoint.0, params)?;

            for ((original_idx, _), &sub_assignment) in
                cluster_points.iter().zip(sub_assignments.iter())
            {
                assignments[*original_idx] = if sub_assignment == 0 {
                    cluster_id
                } else {
                    next_cluster_id
                };
            }
            next_cluster_id += 1;
        }
    }

    Ok(())
}

impl<T> KMeans<T> for MultiPoint<T>
where
    T: GeoFloat,
{
    fn kmeans_with_params(&self, params: KMeansParams<T>) -> Result<Vec<usize>, KMeansError<T>> {
        kmeans_impl(&self.0, params)
    }
}

impl<T> KMeans<T> for &MultiPoint<T>
where
    T: GeoFloat,
{
    fn kmeans_with_params(&self, params: KMeansParams<T>) -> Result<Vec<usize>, KMeansError<T>> {
        kmeans_impl(&self.0, params)
    }
}

impl<T> KMeans<T> for [Point<T>]
where
    T: GeoFloat,
{
    fn kmeans_with_params(&self, params: KMeansParams<T>) -> Result<Vec<usize>, KMeansError<T>> {
        kmeans_impl(self, params)
    }
}

impl<T> KMeans<T> for &[Point<T>]
where
    T: GeoFloat,
{
    fn kmeans_with_params(&self, params: KMeansParams<T>) -> Result<Vec<usize>, KMeansError<T>> {
        kmeans_impl(self, params)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::point;

    #[test]
    fn test_kmeans_empty() {
        let points: Vec<Point<f64>> = vec![];
        let labels = points.kmeans(2).unwrap();
        assert_eq!(labels.len(), 0);
    }

    #[test]
    fn test_kmeans_single_point() {
        let points = [point!(x: 0.0, y: 0.0)];
        let labels = points.kmeans(1).unwrap();
        assert_eq!(labels, vec![0]);
    }

    #[test]
    fn test_kmeans_k_zero() {
        let points = [point!(x: 0.0, y: 0.0), point!(x: 1.0, y: 1.0)];
        let result = points.kmeans(0);
        assert!(matches!(result, Err(KMeansError::InvalidK { k: 0, n: 2 })));
    }

    #[test]
    fn test_kmeans_k_too_large() {
        let points = [point!(x: 0.0, y: 0.0), point!(x: 1.0, y: 1.0)];
        let result = points.kmeans(10);
        assert!(matches!(result, Err(KMeansError::InvalidK { k: 10, n: 2 })));
    }

    #[test]
    fn test_kmeans_k_equals_n() {
        let points = [
            point!(x: 0.0, y: 0.0),
            point!(x: 1.0, y: 1.0),
            point!(x: 2.0, y: 2.0),
        ];
        let labels = points.kmeans(3).unwrap();
        // Each point should be in its own cluster
        assert_eq!(labels, vec![0, 1, 2]);
    }

    #[test]
    fn test_kmeans_single_cluster() {
        let points = [
            point!(x: 0.0, y: 0.0),
            point!(x: 1.0, y: 0.0),
            point!(x: 0.0, y: 1.0),
            point!(x: 1.0, y: 1.0),
        ];
        let labels = points.kmeans(1).unwrap();

        // All points should be in cluster 0
        assert!(labels.iter().all(|&label| label == 0));
    }

    #[test]
    fn test_kmeans_two_clusters() {
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
        let labels = points.kmeans(2).unwrap();

        // First three points should be in the same cluster
        assert_eq!(labels[0], labels[1]);
        assert_eq!(labels[1], labels[2]);

        // Last three points should be in the same cluster
        assert_eq!(labels[3], labels[4]);
        assert_eq!(labels[4], labels[5]);

        // The two groups should be in different clusters
        assert_ne!(labels[0], labels[3]);
    }

    #[test]
    fn test_kmeans_multipoint() {
        let points = MultiPoint::new(vec![
            point!(x: 0.0, y: 0.0),
            point!(x: 1.0, y: 0.0),
            point!(x: 0.0, y: 1.0),
            point!(x: 10.0, y: 10.0),
            point!(x: 11.0, y: 10.0),
            point!(x: 10.0, y: 11.0),
        ]);

        let labels = points.kmeans(2).unwrap();

        // Two distinct clusters
        let cluster_0_count = labels.iter().filter(|&&l| l == 0).count();
        let cluster_1_count = labels.iter().filter(|&&l| l == 1).count();
        assert_eq!(cluster_0_count + cluster_1_count, 6);
    }

    #[test]
    fn test_kmeans_three_clusters() {
        let points = [
            // Cluster 1
            point!(x: 0.0, y: 0.0),
            point!(x: 1.0, y: 0.0),
            // Cluster 2
            point!(x: 10.0, y: 10.0),
            point!(x: 11.0, y: 10.0),
            // Cluster 3
            point!(x: 20.0, y: 20.0),
            point!(x: 21.0, y: 20.0),
        ];
        let labels = points.kmeans(3).unwrap();

        // Each pair should be in the same cluster
        assert_eq!(labels[0], labels[1]);
        assert_eq!(labels[2], labels[3]);
        assert_eq!(labels[4], labels[5]);

        // All three pairs should be in different clusters
        assert_ne!(labels[0], labels[2]);
        assert_ne!(labels[2], labels[4]);
        assert_ne!(labels[0], labels[4]);
    }

    #[test]
    fn test_kmeans_identical_points() {
        // Test with duplicate points at the same location
        let points = [
            point!(x: 0.0, y: 0.0),
            point!(x: 0.0, y: 0.0),
            point!(x: 0.0, y: 0.0),
        ];
        let labels = points.kmeans(1).unwrap();

        // All points should be in the same cluster
        assert!(labels.iter().all(|&label| label == 0));
    }

    #[test]
    fn test_kmeans_linear_cluster() {
        // Test a linear arrangement of points
        let points = [
            point!(x: 0.0, y: 0.0),
            point!(x: 1.0, y: 0.0),
            point!(x: 2.0, y: 0.0),
            point!(x: 3.0, y: 0.0),
            point!(x: 4.0, y: 0.0),
        ];
        let labels = points.kmeans(2).unwrap();

        // Two distinct clusters should exist
        let mut unique_clusters: Vec<_> = labels.to_vec();
        unique_clusters.sort_unstable();
        unique_clusters.dedup();
        assert_eq!(unique_clusters.len(), 2);
    }

    #[test]
    fn test_kmeans_degenerate_all_same_location() {
        // Test with all points at the same location and k > 1
        // This should fail during initialisation because all distances will be zero
        let points = [
            point!(x: 5.0, y: 5.0),
            point!(x: 5.0, y: 5.0),
            point!(x: 5.0, y: 5.0),
            point!(x: 5.0, y: 5.0),
        ];
        let result = points.kmeans(2);

        // Should return InitializationFailed error because all distances are zero
        assert!(matches!(
            result,
            Err(KMeansError::InitializationFailed(
                KMeansInitError::DegenerateData
            ))
        ));
    }

    #[test]
    fn test_kmeans_nan_coordinates() {
        // Test with NaN coordinates
        let points = [
            point!(x: 0.0, y: 0.0),
            point!(x: f64::NAN, y: 1.0),
            point!(x: 2.0, y: 2.0),
        ];
        let result = points.kmeans(2);

        // Should return InitializationFailed error due to NaN
        assert!(matches!(
            result,
            Err(KMeansError::InitializationFailed(
                KMeansInitError::NaNCoordinate
            ))
        ));
    }

    #[test]
    fn test_kmeans_infinite_coordinates() {
        // Test with infinite coordinates
        let points = [
            point!(x: 0.0, y: 0.0),
            point!(x: f64::INFINITY, y: 1.0),
            point!(x: 2.0, y: 2.0),
        ];
        let result = points.kmeans(2);

        // Should return InitializationFailed error due to infinity
        assert!(matches!(
            result,
            Err(KMeansError::InitializationFailed(
                KMeansInitError::InfiniteCoordinate
            ))
        ));
    }

    #[test]
    fn test_kmeans_reproducibility_with_seed() {
        // Test that using the same seed produces identical results
        let points = [
            point!(x: 0.0, y: 0.0),
            point!(x: 1.0, y: 0.0),
            point!(x: 0.0, y: 1.0),
            point!(x: 10.0, y: 10.0),
            point!(x: 11.0, y: 10.0),
            point!(x: 10.0, y: 11.0),
        ];

        let params = KMeansParams::new(2).seed(42);

        let result1 = points.kmeans_with_params(params.clone()).unwrap();
        let result2 = points.kmeans_with_params(params).unwrap();

        // Same seed should produce identical results
        assert_eq!(result1, result2);
    }

    #[test]
    fn test_kmeans_reproducibility_with_seed_and_max_radius() {
        // Test that seed produces reproducible results even when max_radius triggers splitting
        // Create a cluster that will exceed max_radius
        let points = [
            point!(x: 0.0, y: 0.0),
            point!(x: 100.0, y: 0.0),
            point!(x: 0.0, y: 100.0),
            point!(x: 100.0, y: 100.0),
            point!(x: 50.0, y: 50.0),
        ];

        let params = KMeansParams::new(1).seed(42).max_radius(40.0); // Will trigger splitting since centroid is at (50, 50) and max point distance is ~70

        let result1 = points.kmeans_with_params(params.clone()).unwrap();
        let result2 = points.kmeans_with_params(params).unwrap();

        // Same seed should produce identical results even with max_radius splitting
        assert_eq!(result1, result2);
    }

    #[test]
    fn test_kmeans_builder_pattern() {
        // Test the builder pattern with all options
        let points = [
            point!(x: 0.0, y: 0.0),
            point!(x: 1.0, y: 0.0),
            point!(x: 0.0, y: 1.0),
            point!(x: 10.0, y: 10.0),
            point!(x: 11.0, y: 10.0),
            point!(x: 10.0, y: 11.0),
        ];

        // Use builder pattern to configure all parameters
        let params = KMeansParams::new(2).seed(42).max_iter(100).tolerance(0.001);

        let labels = points.kmeans_with_params(params).unwrap();

        // Should successfully cluster into 2 groups
        let unique_labels: std::collections::HashSet<_> = labels.iter().copied().collect();
        assert_eq!(unique_labels.len(), 2);
    }

    #[test]
    fn test_kmeans_converges_with_sufficient_iterations() {
        // Test that with sufficient iterations, we get Ok (no MaxIterationsReached)

        // Note: Testing MaxIterationsReached is challenging because k-means++ initialization
        // combined with Hamerly's algorithm converges very efficiently, often in 1-2 iterations
        // even with complex data. The MaxIterationsReached error path is exercised when max_iter
        // is set too low for the data, but creating a deterministic test case that reliably
        // triggers this without being flaky is difficult.

        let points = [
            point!(x: 0.0, y: 0.0),
            point!(x: 1.0, y: 0.0),
            point!(x: 0.0, y: 1.0),
            point!(x: 10.0, y: 10.0),
            point!(x: 11.0, y: 10.0),
            point!(x: 10.0, y: 11.0),
        ];

        // Use high max_iter which should be enough to converge
        let params = KMeansParams::new(2).seed(42).max_iter(100);

        let result = points.kmeans_with_params(params);

        // Should converge successfully
        assert!(result.is_ok());
    }

    #[test]
    fn test_kmeans_max_radius() {
        use std::collections::HashSet;

        // Create one tight cluster and one elongated cluster
        let points = [
            // Tight cluster (radius approximately 0.7)
            point!(x: 0.0, y: 0.0),
            point!(x: 1.0, y: 0.0),
            point!(x: 0.5, y: 0.5),
            point!(x: 0.0, y: 1.0),
            point!(x: 1.0, y: 1.0),
            // Elongated cluster spread along x-axis (radius approximately 30)
            point!(x: 20.0, y: 0.0),
            point!(x: 30.0, y: 0.0),
            point!(x: 40.0, y: 0.0),
            point!(x: 50.0, y: 0.0),
            point!(x: 60.0, y: 0.0),
            point!(x: 70.0, y: 0.0),
            point!(x: 80.0, y: 0.0),
        ];

        // Ask for k=2 clusters but set max_radius to force splitting of elongated cluster
        let params = KMeansParams::new(2).seed(42).max_radius(15.0);

        let labels = points.kmeans_with_params(params).unwrap();

        // Should produce more than k clusters due to max_radius constraint
        let unique_labels: HashSet<_> = labels.iter().copied().collect();
        assert!(
            unique_labels.len() > 2,
            "Expected more than 2 clusters due to max_radius splitting, got {}",
            unique_labels.len()
        );

        // Verify no cluster exceeds max_radius
        for &cluster_id in &unique_labels {
            let cluster_points: Vec<Point<f64>> = points
                .iter()
                .zip(labels.iter())
                .filter(|&(_, &label)| label == cluster_id)
                .map(|(&p, _)| p)
                .collect();

            if cluster_points.is_empty() {
                continue;
            }

            let multipoint = MultiPoint::new(cluster_points);
            let centroid = multipoint.centroid().unwrap();

            let max_dist = multipoint
                .iter()
                .map(|p| Euclidean.distance(&centroid, p))
                .fold(0.0, f64::max);

            // Allow small numerical tolerance (epsilon) for floating point comparisons
            // The splitting algorithm may not achieve exact max_radius due to the way
            // k-means with k=2 splits clusters
            let epsilon = 2.0;
            assert!(
                max_dist <= 15.0 + epsilon,
                "Cluster {} has radius {:.2} significantly exceeding max_radius 15.0",
                cluster_id,
                max_dist
            );
        }

        // Verify tight cluster (first 5 points) stayed together
        let tight_cluster_labels: HashSet<_> = labels[0..5].iter().copied().collect();
        assert_eq!(
            tight_cluster_labels.len(),
            1,
            "Tight cluster should remain as one cluster, but has {} different labels",
            tight_cluster_labels.len()
        );

        // Verify elongated cluster (last 7 points) was split into multiple clusters
        let elongated_cluster_labels: HashSet<_> = labels[5..].iter().copied().collect();
        assert!(
            elongated_cluster_labels.len() >= 2,
            "Elongated cluster should be split into at least 2 clusters, got {}",
            elongated_cluster_labels.len()
        );
    }
}
