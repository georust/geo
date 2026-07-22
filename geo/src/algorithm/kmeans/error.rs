use crate::GeoFloat;
use rand::distr::weighted::Error;

/// Errors that can occur during k-means++ initialisation
#[derive(Debug, Clone, PartialEq, thiserror::Error)]
pub enum KMeansInitError {
    /// Input data contains `NaN` coordinates
    ///
    /// This typically indicates invalid or corrupted input data. Check your point coordinates
    /// for NaN values before running k-means.
    #[error("Input data contains NaN coordinates")]
    NaNCoordinate,

    /// Input data contains infinite coordinates
    ///
    /// This typically indicates invalid or corrupted input data. Check your point coordinates
    /// for infinite values before running k-means.
    #[error("Input data contains infinite coordinates")]
    InfiniteCoordinate,

    /// All points are at identical locations, making clustering impossible
    ///
    /// When all points have the same coordinates, k-means++ cannot select diverse initial
    /// centroids because all distances are zero. This makes the weighted sampling distribution
    /// degenerate.
    ///
    /// Consider:
    /// - Checking if your data has been properly loaded
    /// - Adding small random perturbations to break ties if appropriate.
    #[error("All points are at identical locations, making clustering impossible")]
    DegenerateData,

    /// Failed to create weighted distribution for k-means++ sampling
    ///
    /// This error occurs when the random weighted sampling distribution cannot be constructed,
    /// typically because all weights sum to zero or contain invalid values.
    #[error("Failed to create weighted distribution for k-means++ sampling: {error}")]
    WeightedDistributionFailed {
        /// The underlying error from rand's WeightedIndex
        #[source]
        error: Error,
    },
}

/// Errors that can occur during k-means clustering
#[derive(Debug, Clone, PartialEq, thiserror::Error)]
pub enum KMeansError<T: GeoFloat> {
    /// Invalid number of clusters requested
    #[error("Invalid k={k}: must be > 0 and <= {n} (number of points)")]
    InvalidK {
        /// The requested number of clusters
        k: usize,
        /// The number of points in the dataset
        n: usize,
    },

    /// An empty cluster was encountered and could not be recovered
    #[error("Empty cluster {cluster_id} at iteration {iteration} could not be recovered")]
    EmptyCluster {
        /// The iteration number when the error occurred
        iteration: usize,
        /// The cluster ID that became empty
        cluster_id: usize,
    },

    /// Initialisation failed due to degenerate or invalid data
    ///
    /// Contains specific information about the initialisation failure mode.
    /// See [`KMeansInitError`] for details on each failure type.
    #[error("K-means initialisation failed: {0}")]
    InitializationFailed(#[source] KMeansInitError),

    /// Maximum iterations reached without convergence
    ///
    /// The algorithm completed `iterations` iterations without converging (no assignment
    /// changes and centroid movement below tolerance). The cluster assignments from the
    /// final iteration are included and may still be useful.
    ///
    /// ## Interpreting the Results
    ///
    /// Consider using the partial result if:
    /// - `max_centroid_shift` is relatively small compared to your data scale
    /// - `changed_assignments` is a small fraction of total points
    /// - Visual inspection or validation metrics are acceptable
    ///
    /// Consider increasing `max_iter` if:
    /// - `max_centroid_shift` is large relative to tolerance
    /// - A significant number of points are still changing clusters
    /// - You require guaranteed convergence
    #[error(
        "k-means did not converge within {iterations} iterations. \
         Final centroid shift: {:.6} (tolerance: {:.6}), \
         {changed_assignments} points changed clusters in final iteration. \
         Consider increasing max_iter or using the partial result.",
        .max_centroid_shift.to_f64().expect("max_centroid_shift must be representable as f64"),
        .tolerance.to_f64().expect("tolerance must be representable as f64")
    )]
    MaxIterationsReached {
        /// Cluster assignments from the final iteration.
        ///
        /// These are valid assignments (each point assigned to nearest centroid)
        /// but may improve with additional iterations.
        assignments: Vec<usize>,
        /// Number of iterations completed (equal to max_iter)
        iterations: usize,
        /// Maximum distance any centroid shifted in the final iteration
        max_centroid_shift: T,
        /// The tolerance threshold for convergence
        tolerance: T,
        /// Number of points that changed clusters in the final iteration
        changed_assignments: usize,
    },
}
