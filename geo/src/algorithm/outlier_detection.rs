use std::iter::Sum;
use std::ops::RangeInclusive;

use crate::{GeoFloat, MultiPoint, Point};

use rstar::primitives::GeomWithData;
use rstar::RTree;

/// Calculate the [Local Outlier Factor](https://en.wikipedia.org/wiki/Local_outlier_factor) of a set of points
///
/// Based on: Breunig, M., Kriegel, H., Ng, R., and Sander, J. (2000). *LOF: identifying density-based local
/// outliers.* In ACM Int. Conf. on Management of Data, pages 93-104. doi: [10.1145/335191.335388](https://doi.org/10.1145/335191.335388)
///
/// LOF is an unsupervised algorithm that uses local data for anomaly detection.
///
/// Outlier vs inlier classification is **highly dependent** on the shape of the data. LOF values <= 1
/// can generally be considered inliers, but e.g. a highly concentrated, uniform dataset could result in
/// points with a LOF of 1.05 being outliers.
/// LOF scores should thus be evaluated in the context of the dataset as a whole in order to classify outliers.
///
/// If you wish to run multiple outlier detection processes with differing neighbour counts in order
/// to build up data for more robust detection (see p. 100-1 above), you can use the [`OutlierDetection::prepared_detector`] method, which retains
/// the spatial index and point set between runs for greater efficiency. The [`OutlierDetection::generate_ensemble`] method
/// will efficiently run the LOF algorithm over a contiguous range of neighbour inputs,
/// allowing aggregations to be carried out over the resulting data.
pub trait OutlierDetection<T>
where
    T: GeoFloat,
{
    /// The LOF algorithm. `k_neighbours` specifies the number of neighbours to use for local outlier
    /// classification. The paper linked above (see p. 100) suggests a `k_neighbours` value of 10 - 20
    /// as a lower bound for "real-world"
    /// data.
    ///
    /// # Note on Erroneous Input
    /// If `k_neighbours` >= points in the set, or `k_neighbours` < 1, all input points will be returned with an LOF score of 1.
    /// If there are at least `k_neighbours` duplicate points of an input point, LOF scores can be `∞` or `NaN`.
    /// It is thus advisable to **deduplicate** or otherwise ensure the uniqueness of the input points.
    ///
    /// # Note on Returned Points
    /// Outlier scores are always returned corresponding to input point order
    ///
    /// # Examples
    ///
    /// ## MultiPoint
    ///
    /// ```
    /// use approx::assert_relative_eq;
    /// use geo::OutlierDetection;
    /// use geo::{point, MultiPoint};
    ///
    /// let v = vec![
    ///     point!(x: 0.0, y: 0.0),
    ///     point!(x: 0.0, y: 1.0),
    ///     point!(x: 3.0, y: 0.0),
    ///     point!(x: 1.0, y: 1.0),
    /// ];
    ///
    /// let lofscores = v.outliers(2);
    /// // The third point is an outlier, resulting in a large LOF score
    /// assert_relative_eq!(lofscores[2], 3.0);
    /// // The last point is an inlier, resulting in a small LOF score
    /// assert_relative_eq!(lofscores[3], 1.0);
    /// ```
    ///
    /// ## Computing indices, sorting by LOF score
    ///```
    /// use geo::OutlierDetection;
    /// use geo::{point, MultiPoint};
    ///
    /// // these points contain 4 strong outliers
    /// let v = vec![
    ///     point!(x: 0.16, y: 0.14),
    ///     point!(x: 0.15, y: 0.33),
    ///     point!(x: 0.37, y: 0.25),
    ///     point!(x: 0.3 , y: 0.4),
    ///     point!(x: 0.3 , y: 0.1),
    ///     point!(x: 0.3 , y: 0.2),
    ///     point!(x: 1.3 , y: 2.3),
    ///     point!(x: 1.7 , y: 0.2),
    ///     point!(x: 0.7 , y: -0.9),
    ///     point!(x: 0.21, y: 2.45),
    ///     point!(x: 0.8 , y: 0.7),
    ///     point!(x: 0.9 , y: 0.7),
    ///     point!(x: 0.8 , y: 0.6),
    ///     point!(x: 0.73, y: 0.65),
    ///     point!(x: 0.9 , y: 0.6),
    ///     point!(x: 1.0, y: 0.6),
    ///     point!(x: 1.0, y: 0.7),
    ///     point!(x: 0.25, y: 0.29),
    ///     point!(x: 0.2 , y: 0.2),
    /// ];
    /// let lofs = &mut v.outliers(3);
    /// let mut idx_lofs: Vec<(usize, f64)> = lofs
    ///     .iter()
    ///     .enumerate()
    ///     .map(|(idx, score)| (idx, *score))
    ///     .collect();
    /// // sort by LOF score
    /// idx_lofs.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap());
    /// // most likely outliers first
    /// idx_lofs.reverse();
    /// // four outliers, LOF scores way above 10
    /// idx_lofs
    ///     .iter()
    ///     .take(4)
    ///     .for_each(|score| assert!(score.1 > 10.0));
    ///```
    fn outliers(&self, k_neighbours: usize) -> Vec<T>;

    /// Create a prepared outlier detector allowing multiple runs to retain the spatial index in use.
    /// A [`PreparedDetector`] can efficiently recompute outliers with different `k_neigbhours` values.
    fn prepared_detector(&self) -> PreparedDetector<T>;

    /// Perform successive runs with `k_neighbours` values between `bounds`,
    /// generating an ensemble of LOF scores, which may be aggregated using e.g. min, max, or mean
    ///
    /// # Examples
    ///```
    /// use geo::OutlierDetection;
    /// use geo::{point, Point, MultiPoint};
    /// let v: Vec<Point<f64>> = vec![
    ///     point!(x: 0.16, y: 0.14),
    ///     point!(x: 0.15, y: 0.33),
    ///     point!(x: 0.37, y: 0.25),
    ///     point!(x: 0.3 , y: 0.4),
    ///     point!(x: 0.3 , y: 0.1),
    ///     point!(x: 0.3 , y: 0.2),
    ///     point!(x: 1.3 , y: 2.3),
    ///     point!(x: 1.7 , y: 0.2),
    ///     point!(x: 0.7 , y: -0.9),
    ///     point!(x: 0.21, y: 2.45),
    ///     point!(x: 0.8 , y: 0.7),
    ///     point!(x: 0.9 , y: 0.7),
    ///     point!(x: 0.8 , y: 0.6),
    ///     point!(x: 0.73, y: 0.65),
    ///     point!(x: 0.9 , y: 0.6),
    ///     point!(x: 1.0, y: 0.6),
    ///     point!(x: 1.0, y: 0.7),
    ///     point!(x: 0.25, y: 0.29),
    ///     point!(x: 0.2 , y: 0.2),
    /// ];
    /// let ensemble = v.generate_ensemble((2..=5));
    /// // retain the maximum LOF value for each point for all runs
    /// // this will result in a single Vec
    /// let aggregated = ensemble[1..].iter().fold(ensemble[0].clone(), |acc, xs| {
    ///     acc.iter()
    ///         .zip(xs)
    ///         .map(|(elem1, elem2)| elem1.min(*elem2))
    ///         .collect()
    /// });
    /// assert_eq!(v.len(), aggregated.len());
    ///```
    fn generate_ensemble(&self, bounds: RangeInclusive<usize>) -> Vec<Vec<T>>;

    /// Convenience method to efficiently calculate the minimum values of an LOF ensemble
    fn ensemble_min(&self, bounds: RangeInclusive<usize>) -> Vec<T>;

    /// Convenience method to efficiently calculate the maximum values of an LOF ensemble
    fn ensemble_max(&self, bounds: RangeInclusive<usize>) -> Vec<T>;
}

/// This struct allows multiple detection operations to be run on a point set using varying `k_neighbours` sizes
/// without having to rebuild the underlying spatial index. Its [`PreparedDetector::outliers`] method
/// has the same signature as [`OutlierDetection::outliers`], but retains the underlying spatial index and point set
/// for greater efficiency.
#[derive(Clone, Debug)]
pub struct PreparedDetector<'a, T>
where
    T: GeoFloat,
{
    tree: RTree<GeomWithData<Point<T>, usize>>,
    points: &'a [Point<T>],
}

impl<'a, T> PreparedDetector<'a, T>
where
    T: GeoFloat + Sum,
{
    /// Create a new "prepared" detector which allows repeated LOF algorithm calls with varying neighbour sizes
    fn new(points: &'a [Point<T>]) -> Self {
        let geoms: Vec<GeomWithData<_, usize>> = points
            .iter()
            .enumerate()
            .map(|(idx, point)| GeomWithData::new(*point, idx))
            .collect();
        let tree = RTree::bulk_load(geoms);
        Self { tree, points }
    }

    /// See [`OutlierDetection::outliers`] for usage
    pub fn outliers(&self, kneighbours: usize) -> Vec<T> {
        lof(self.points, &self.tree, kneighbours)
    }
}

fn lof<T>(
    points: &[Point<T>],
    tree: &RTree<GeomWithData<Point<T>, usize>>,
    kneighbours: usize,
) -> Vec<T>
where
    T: GeoFloat + Sum,
{
    debug_assert!(kneighbours > 0);
    if points.len() <= kneighbours || kneighbours < 1 {
        // no point in trying to run the algorithm in this case
        return points.iter().map(|_| T::one()).collect();
    }
    let knn_dists = points
        .iter()
        .map(|point| {
            tree.nearest_neighbor_iter_with_distance_2(point)
                .take(kneighbours)
                .collect()
        })
        .collect::<Vec<Vec<_>>>();
    // calculate LRD (local reachability density) of each point
    // LRD is the estimated distance at which a point can be found by its neighbours:
    // count(neighbour_set) / sum(max(point.kTh_dist, point.dist2(other point)) for all points in neighbour_set)
    // we call this sum-of–max-distances reachDistance
    let local_reachability_densities: Vec<T> = knn_dists
        .iter()
        .map(|neighbours| {
            // for each point's neighbour set, calculate kth distance
            let kth_dist = neighbours
                .iter()
                .map(|(_, distance)| distance)
                .last()
                .unwrap();
            T::from(neighbours.len()).unwrap()
                / neighbours
                    .iter()
                    // sum the max between neighbour distance and kth distance for the neighbour set
                    .map(|(_, distance)| distance.max(*kth_dist))
                    .sum()
        })
        .collect();
    // LOF of a point p is the sum of the LRD of all the points
    // in the set kNearestSet(p) * the sum of the reachDistance of all the points of the same set,
    // to the point p, all divided by the number of items in p's kNN set, squared.
    knn_dists
        .iter()
        .map(|neighbours| {
            // for each point's neighbour set, calculate kth distance
            let kth_dist = neighbours
                .iter()
                .map(|(_, distance)| distance)
                .last()
                .unwrap();
            // sum neighbour set LRD scores
            let lrd_scores: T = neighbours
                .iter()
                .map(|(neighbour, _)| local_reachability_densities[neighbour.data])
                .sum();
            // sum neighbour set reachDistance
            let sum_rd: T = neighbours
                .iter()
                .map(|(_, distance)| distance.max(*kth_dist))
                .sum();
            (lrd_scores * sum_rd) / T::from(neighbours.len().pow(2)).unwrap()
        })
        .collect()
}

impl<T> OutlierDetection<T> for MultiPoint<T>
where
    T: GeoFloat + Sum,
{
    fn outliers(&self, k_neighbours: usize) -> Vec<T> {
        let pd = self.prepared_detector();
        pd.outliers(k_neighbours)
    }

    fn prepared_detector(&self) -> PreparedDetector<T> {
        PreparedDetector::new(&self.0)
    }

    fn generate_ensemble(&self, bounds: RangeInclusive<usize>) -> Vec<Vec<T>> {
        let pd = self.prepared_detector();
        bounds.map(|kneighbours| pd.outliers(kneighbours)).collect()
    }
    fn ensemble_min(&self, bounds: RangeInclusive<usize>) -> Vec<T> {
        let pd = self.prepared_detector();
        bounds
            .map(|kneighbours| pd.outliers(kneighbours))
            .reduce(|acc, vec| acc.iter().zip(vec).map(|(a, b)| a.min(b)).collect())
            .unwrap()
    }

    fn ensemble_max(&self, bounds: RangeInclusive<usize>) -> Vec<T> {
        let pd = self.prepared_detector();
        bounds
            .map(|kneighbours| pd.outliers(kneighbours))
            .reduce(|acc, vec| acc.iter().zip(vec).map(|(a, b)| a.max(b)).collect())
            .unwrap()
    }
}

impl<T> OutlierDetection<T> for [Point<T>]
where
    T: GeoFloat + Sum,
{
    fn outliers(&self, k_neighbours: usize) -> Vec<T> {
        let pd = self.prepared_detector();
        pd.outliers(k_neighbours)
    }

    fn prepared_detector(&self) -> PreparedDetector<T> {
        PreparedDetector::new(self)
    }

    fn generate_ensemble(&self, bounds: RangeInclusive<usize>) -> Vec<Vec<T>> {
        let pd = self.prepared_detector();
        bounds.map(|kneighbours| pd.outliers(kneighbours)).collect()
    }

    fn ensemble_min(&self, bounds: RangeInclusive<usize>) -> Vec<T> {
        let pd = self.prepared_detector();
        bounds
            .map(|kneighbours| pd.outliers(kneighbours))
            .reduce(|acc, vec| acc.iter().zip(vec).map(|(a, b)| a.min(b)).collect())
            .unwrap()
    }

    fn ensemble_max(&self, bounds: RangeInclusive<usize>) -> Vec<T> {
        let pd = self.prepared_detector();
        bounds
            .map(|kneighbours| pd.outliers(kneighbours))
            .reduce(|acc, vec| acc.iter().zip(vec).map(|(a, b)| a.max(b)).collect())
            .unwrap()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::point;

    #[test]
    fn test_lof() {
        // third point is an outlier
        let v = [
            Point::new(0.0, 0.0),
            Point::new(0.0, 1.0),
            Point::new(3.0, 0.0),
            Point::new(1.0, 1.0),
        ];

        let lofs = &v.outliers(3);
        assert_eq!(lofs[2], 3.3333333333333335);
    }
    #[test]
    fn test_lof2() {
        // fourth point is an outlier
        let v = [
            Point::new(0.0, 0.0),
            Point::new(1.0, 0.0),
            Point::new(1.0, 1.0),
            Point::new(0.0, 3.0),
        ];
        let lofs = &v.outliers(3);
        assert_eq!(lofs[3], 3.3333333333333335);
    }
    #[test]
    fn test_lof3() {
        // second point is an outlier, sort and reverse so scores are in descending order
        let v = [
            Point::new(0.0, 0.0),
            Point::new(0.0, 3.0),
            Point::new(1.0, 0.0),
            Point::new(1.0, 1.0),
        ];
        let lofs = &mut v.outliers(3);
        lofs.sort_by(|a, b| a.partial_cmp(b).unwrap());
        lofs.reverse();
        assert_eq!(lofs[0], 3.3333333333333335);
    }
    #[test]
    fn test_lof4() {
        // this dataset contains 4 outliers
        // indices 6, 7, 8, 9 should be detected
        // order: 9, 6, 8, 7
        let v = vec![
            point!(x: 0.16, y: 0.14),
            point!(x: 0.15, y: 0.33),
            point!(x: 0.37, y: 0.25),
            point!(x: 0.3 , y: 0.4),
            point!(x: 0.3 , y: 0.1),
            point!(x: 0.3 , y: 0.2),
            point!(x: 1.3 , y: 2.3),
            point!(x: 1.7 , y: 0.2),
            point!(x: 0.7 , y: -0.9),
            point!(x: 0.21, y: 2.45),
            point!(x: 0.8 , y: 0.7),
            point!(x: 0.9 , y: 0.7),
            point!(x: 0.8 , y: 0.6),
            point!(x: 0.73, y: 0.65),
            point!(x: 0.9 , y: 0.6),
            point!(x: 1.0, y: 0.6),
            point!(x: 1.0, y: 0.7),
            point!(x: 0.25, y: 0.29),
            point!(x: 0.2 , y: 0.2),
        ];
        let lofs = &mut v.outliers(3);
        let mut idx_lofs: Vec<(usize, f64)> = lofs
            .iter()
            .enumerate()
            .map(|(idx, score)| (idx, *score))
            .collect();
        idx_lofs.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap());
        idx_lofs.reverse();
        // four outliers, scores way above 10
        idx_lofs
            .iter()
            .take(4)
            .for_each(|score| assert!(score.1 > 10.0));
        // the rest below 2
        idx_lofs
            .iter()
            .skip(4)
            .for_each(|score| assert!(score.1 < 2.0));
        // ensure that scores are being computed correctly
        assert_eq!(idx_lofs[0].0, 9);
        assert_eq!(idx_lofs[1].0, 6);
        assert_eq!(idx_lofs[2].0, 8);
        assert_eq!(idx_lofs[3].0, 7);
    }
    #[test]
    fn test_lof5() {
        // third point is an outlier
        let v = [
            Point::new(0.0, 0.0),
            Point::new(0.0, 1.0),
            Point::new(3.0, 0.0),
            Point::new(1.0, 1.0),
        ];

        let prepared = &v.prepared_detector();
        let s1 = prepared.outliers(2);
        let s2 = prepared.outliers(3);
        // different neighbour sizes give different scores
        assert_ne!(s1[2], s2[2]);
    }
}
