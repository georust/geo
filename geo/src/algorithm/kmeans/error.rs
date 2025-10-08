use crate::GeoFloat;
use rand::distributions::WeightedError;

/// Errors that can occur during k-means++ initialisation
#[derive(Debug, Clone, PartialEq)]
pub enum KMeansInitError {
    /// Input data contains `NaN` coordinates
    ///
    /// This typically indicates invalid or corrupted input data. Check your point coordinates
    /// for NaN values before running k-means.
    NaNCoordinate,

    /// Input data contains infinite coordinates
    ///
    /// This typically indicates invalid or corrupted input data. Check your point coordinates
    /// for infinite values before running k-means.
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
    DegenerateData,

    /// Failed to create weighted distribution for k-means++ sampling
    ///
    /// This error occurs when the random weighted sampling distribution cannot be constructed,
    /// typically because all weights sum to zero or contain invalid values.
    WeightedDistributionFailed {
        /// The underlying error from rand's WeightedIndex
        error: WeightedError,
    },
}

impl std::fmt::Display for KMeansInitError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            KMeansInitError::NaNCoordinate => {
                write!(f, "Input data contains NaN coordinates")
            }
            KMeansInitError::InfiniteCoordinate => {
                write!(f, "Input data contains infinite coordinates")
            }
            KMeansInitError::DegenerateData => {
                write!(
                    f,
                    "All points are at identical locations, making clustering impossible"
                )
            }
            KMeansInitError::WeightedDistributionFailed { error } => {
                write!(
                    f,
                    "Failed to create weighted distribution for k-means++ sampling: {}",
                    error
                )
            }
        }
    }
}

/// Errors that can occur during k-means clustering
#[derive(Debug, Clone, PartialEq)]
pub enum KMeansError<T: GeoFloat> {
    /// Invalid number of clusters requested
    InvalidK {
        /// The requested number of clusters
        k: usize,
        /// The number of points in the dataset
        n: usize,
    },

    /// An empty cluster was encountered and could not be recovered
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
    InitializationFailed(KMeansInitError),

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

impl<T: GeoFloat> std::fmt::Display for KMeansError<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            KMeansError::InvalidK { k, n } => {
                write!(
                    f,
                    "Invalid k={}: must be > 0 and <= {} (number of points)",
                    k, n
                )
            }
            KMeansError::EmptyCluster {
                iteration,
                cluster_id,
            } => {
                write!(
                    f,
                    "Empty cluster {} at iteration {} could not be recovered",
                    cluster_id, iteration
                )
            }
            KMeansError::InitializationFailed(init_error) => {
                write!(f, "K-means initialisation failed: {}", init_error)
            }
            KMeansError::MaxIterationsReached {
                iterations,
                max_centroid_shift,
                tolerance,
                changed_assignments,
                ..
            } => {
                let shift_f64 = max_centroid_shift
                    .to_f64()
                    .expect("max_centroid_shift must be representable as f64");
                let tol_f64 = tolerance
                    .to_f64()
                    .expect("tolerance must be representable as f64");
                write!(
                    f,
                    "k-means did not converge within {} iterations. \
                     Final centroid shift: {:.6} (tolerance: {:.6}), \
                     {} points changed clusters in final iteration. \
                     Consider increasing max_iter or using the partial result.",
                    iterations, shift_f64, tol_f64, changed_assignments
                )
            }
        }
    }
}

impl<T: GeoFloat> std::error::Error for KMeansError<T> {}
