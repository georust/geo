use crate::{
    Contains, ConvexHull, CoordNum, Coordinate, GeoFloat, Intersects, LineString, MultiPoint,
    Point, Polygon,
};
use num_traits::Float;
use rstar::RTreeNum;
use std::cmp::max;

const K_MULTIPLIER: f32 = 1.5;

/// Another approach for [concave hull](trait.algorithm.ConcaveHull.html). This algorithm is based
/// on a [k nearest neighbours approach](https://pdfs.semanticscholar.org/2397/17005c3ebd5d6a42fc833daf97a0edee1ce4.pdf)
/// by Adriano Moreira and Maribel Santos.
///
/// The idea of the algorithm is simple:
/// 1. Find a point on a future hull (e. g. a point with the smallest Y coordinate).
/// 2. Find K nearest neighbours to the chosen point.
/// 3. As the next point on the hull chose one of the nearest points, that would make the largest
///    left hand turn from the previous segment.
/// 4. Repeat 2-4.
///
/// In cases when the hull cannot be calculated for the given K, a larger value is chosen and
/// calculation starts from the beginning.
///
/// In the worst case scenario, when no K can be found to build a correct hull, the convex hull is
/// returned.
///
/// This algorithm is generally several times slower then the one used in the
/// [ConcaveHull](trait.algorithm.ConcaveHull.html) trait, but gives better results and
/// does not require manual coefficient adjustment.
///
/// The larger K is given to the algorithm, the more "smooth" the hull will generally be, but the
/// longer calculation may take. If performance is not critical, K=3 is a safe value to set
/// (lower values do not make sense for this algorithm). If K is equal or larger than the number of
/// input points, the convex hull will be produced.
pub trait KNearestConcaveHull {
    type Scalar: CoordNum;
    fn k_nearest_concave_hull(&self, k: u32) -> Polygon<Self::Scalar>;
}

impl<T> KNearestConcaveHull for Vec<Point<T>>
where
    T: GeoFloat + RTreeNum,
{
    type Scalar = T;
    fn k_nearest_concave_hull(&self, k: u32) -> Polygon<Self::Scalar> {
        concave_hull(self.iter().map(|point| &point.0), k)
    }
}

impl<T> KNearestConcaveHull for [Point<T>]
where
    T: GeoFloat + RTreeNum,
{
    type Scalar = T;
    fn k_nearest_concave_hull(&self, k: u32) -> Polygon<Self::Scalar> {
        concave_hull(self.iter().map(|point| &point.0), k)
    }
}

impl<T> KNearestConcaveHull for Vec<Coordinate<T>>
where
    T: GeoFloat + RTreeNum,
{
    type Scalar = T;
    fn k_nearest_concave_hull(&self, k: u32) -> Polygon<Self::Scalar> {
        concave_hull(self.iter(), k)
    }
}

impl<T> KNearestConcaveHull for [Coordinate<T>]
where
    T: GeoFloat + RTreeNum,
{
    type Scalar = T;
    fn k_nearest_concave_hull(&self, k: u32) -> Polygon<Self::Scalar> {
        concave_hull(self.iter(), k)
    }
}

impl<T> KNearestConcaveHull for MultiPoint<T>
where
    T: GeoFloat + RTreeNum,
{
    type Scalar = T;
    fn k_nearest_concave_hull(&self, k: u32) -> Polygon<Self::Scalar> {
        concave_hull(self.iter().map(|point| &point.0), k)
    }
}

fn concave_hull<'a, T: 'a>(coords: impl Iterator<Item = &'a Coordinate<T>>, k: u32) -> Polygon<T>
where
    T: GeoFloat + RTreeNum,
{
    let dataset = prepare_dataset(coords);
    concave_hull_inner(dataset, k)
}

const DELTA: f32 = 0.000000001;

/// Removes duplicate coords from the dataset.
fn prepare_dataset<'a, T: 'a>(
    coords: impl Iterator<Item = &'a Coordinate<T>>,
) -> rstar::RTree<Coordinate<T>>
where
    T: GeoFloat + RTreeNum,
{
    let mut dataset: rstar::RTree<Coordinate<T>> = rstar::RTree::new();
    for coord in coords {
        let closest = dataset.nearest_neighbor(coord);
        if let Some(closest) = closest {
            if coords_are_equal(coord, closest) {
                continue;
            }
        }

        dataset.insert(*coord)
    }

    dataset
}

/// The points are considered equal, if both coordinate values are same with 0.0000001% range
/// (see the value of DELTA constant).
fn coords_are_equal<T>(c1: &Coordinate<T>, c2: &Coordinate<T>) -> bool
where
    T: GeoFloat + RTreeNum,
{
    float_equal(c1.x, c2.x) && float_equal(c1.y, c2.y)
}

fn float_equal<T>(a: T, b: T) -> bool
where
    T: GeoFloat,
{
    let da = a * T::from(DELTA)
        .expect("Conversion from constant is always valid.")
        .abs();
    b > (a - da) && b < (a + da)
}

fn polygon_from_tree<T>(dataset: &rstar::RTree<Coordinate<T>>) -> Polygon<T>
where
    T: GeoFloat + RTreeNum,
{
    assert!(dataset.size() <= 3);

    let mut coords: Vec<Coordinate<T>> = dataset.iter().cloned().collect();
    if !coords.is_empty() {
        // close the linestring provided it's not empty
        coords.push(coords[0]);
    }

    Polygon::new(LineString::from(coords), vec![])
}

fn concave_hull_inner<T>(original_dataset: rstar::RTree<Coordinate<T>>, k: u32) -> Polygon<T>
where
    T: GeoFloat + RTreeNum,
{
    let set_length = original_dataset.size();
    if set_length <= 3 {
        return polygon_from_tree(&original_dataset);
    }
    if k >= set_length as u32 {
        return fall_back_hull(&original_dataset);
    }

    let k_adjusted = adjust_k(k);
    let mut dataset = original_dataset.clone();

    let first_coord = get_first_coord(&dataset);
    let mut hull = vec![first_coord];

    let mut current_coord = first_coord;
    dataset.remove(&first_coord);

    let mut prev_coord = current_coord;
    let mut curr_step = 2;
    while (current_coord != first_coord || curr_step == 2) && dataset.size() > 0 {
        if curr_step == 5 {
            dataset.insert(first_coord);
        }

        let mut nearest_coords = get_nearest_coords(&dataset, &current_coord, k_adjusted).collect();
        sort_by_angle(&mut nearest_coords, &current_coord, &prev_coord);

        let selected = nearest_coords
            .iter()
            .find(|x| !intersects(&hull, &[&current_coord, x]));

        if let Some(sel) = selected {
            prev_coord = current_coord;
            current_coord = **sel;
            hull.push(current_coord);
            dataset.remove(&current_coord);

            curr_step += 1;
        } else {
            return concave_hull_inner(original_dataset, get_next_k(k_adjusted));
        }
    }

    let poly = Polygon::new(LineString::from(hull), vec![]);

    if original_dataset
        .iter()
        .any(|&coord| !coord_inside(&coord, &poly))
    {
        return concave_hull_inner(original_dataset, get_next_k(k_adjusted));
    }

    poly
}

fn fall_back_hull<T>(dataset: &rstar::RTree<Coordinate<T>>) -> Polygon<T>
where
    T: GeoFloat + RTreeNum,
{
    let multipoint = MultiPoint::from(dataset.iter().cloned().collect::<Vec<Coordinate<T>>>());
    multipoint.convex_hull()
}

fn get_next_k(curr_k: u32) -> u32 {
    max(curr_k + 1, ((curr_k as f32) * K_MULTIPLIER) as u32)
}

fn adjust_k(k: u32) -> u32 {
    max(k, 3)
}

fn get_first_coord<T>(coord_set: &rstar::RTree<Coordinate<T>>) -> Coordinate<T>
where
    T: GeoFloat + RTreeNum,
{
    let mut min_y = Float::max_value();
    let mut result = coord_set
        .iter()
        .next()
        .expect("We checked that there are more then 3 coords in the set before.");

    for coord in coord_set.iter() {
        if coord.y < min_y {
            min_y = coord.y;
            result = coord;
        }
    }

    *result
}

fn get_nearest_coords<'a, T>(
    dataset: &'a rstar::RTree<Coordinate<T>>,
    base_coord: &Coordinate<T>,
    candidate_no: u32,
) -> impl Iterator<Item = &'a Coordinate<T>>
where
    T: GeoFloat + RTreeNum,
{
    dataset
        .nearest_neighbor_iter(base_coord)
        .take(candidate_no as usize)
}

fn sort_by_angle<T>(
    coords: &mut Vec<&Coordinate<T>>,
    curr_coord: &Coordinate<T>,
    prev_coord: &Coordinate<T>,
) where
    T: GeoFloat,
{
    let base_angle = pseudo_angle(prev_coord.x - curr_coord.x, prev_coord.y - curr_coord.y);
    coords.sort_by(|a, b| {
        let mut angle_a = pseudo_angle(a.x - curr_coord.x, a.y - curr_coord.y) - base_angle;
        if angle_a < T::zero() {
            angle_a = angle_a + T::from(4.0).unwrap();
        }

        let mut angle_b = pseudo_angle(b.x - curr_coord.x, b.y - curr_coord.y) - base_angle;
        if angle_b < T::zero() {
            angle_b = angle_b + T::from(4.0).unwrap();
        }

        angle_a.partial_cmp(&angle_b).unwrap().reverse()
    });
}

fn pseudo_angle<T>(dx: T, dy: T) -> T
where
    T: GeoFloat,
{
    if dx == T::zero() && dy == T::zero() {
        return T::zero();
    }

    let p = dx / (dx.abs() + dy.abs());
    if dy < T::zero() {
        T::from(3.).unwrap() + p
    } else {
        T::from(1.).unwrap() - p
    }
}

fn intersects<T>(hull: &[Coordinate<T>], line: &[&Coordinate<T>; 2]) -> bool
where
    T: GeoFloat,
{
    // This is the case of finishing the contour.
    if *line[1] == hull[0] {
        return false;
    }

    let coords = hull.iter().take(hull.len() - 1).cloned().collect();
    let linestring = LineString::new(coords);
    let line = crate::Line::new(*line[0], *line[1]);
    linestring.intersects(&line)
}

fn coord_inside<T>(coord: &Coordinate<T>, poly: &Polygon<T>) -> bool
where
    T: GeoFloat,
{
    poly.contains(coord) || poly.exterior().contains(coord)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::coords_iter::CoordsIter;
    use crate::geo_types::coord;

    #[test]
    fn coord_ordering() {
        let coords = vec![
            coord!(x: 1.0, y: 1.0),
            coord!(x: -1.0, y: 0.0),
            coord!(x: 0.0, y: 1.0),
            coord!(x: 1.0, y: 0.0),
        ];

        let mut coords_mapped: Vec<&Coordinate<f32>> = coords.iter().collect();

        let center = coord!(x: 0.0, y: 0.0);
        let prev_coord = coord!(x: 1.0, y: 1.0);

        let expected = vec![&coords[3], &coords[1], &coords[2], &coords[0]];

        sort_by_angle(&mut coords_mapped, &center, &prev_coord);
        assert_eq!(coords_mapped, expected);

        let expected = vec![&coords[1], &coords[2], &coords[0], &coords[3]];

        let prev_coord = coord!(x: 1.0, y: -1.0);
        sort_by_angle(&mut coords_mapped, &center, &prev_coord);
        assert_eq!(coords_mapped, expected);
    }

    #[test]
    fn get_first_coord_test() {
        let coords = vec![
            coord!(x: 1.0, y: 1.0),
            coord!(x: -1.0, y: 0.0),
            coord!(x: 0.0, y: 1.0),
            coord!(x: 0.0, y: 0.5),
        ];
        let tree = rstar::RTree::bulk_load(coords);
        let first = coord!(x: -1.0, y: 0.0);

        assert_eq!(get_first_coord(&tree), first);
    }

    #[test]
    fn concave_hull_test() {
        let coords = vec![
            coord!(x: 0.0, y: 0.0),
            coord!(x: 1.0, y: 0.0),
            coord!(x: 2.0, y: 0.0),
            coord!(x: 3.0, y: 0.0),
            coord!(x: 0.0, y: 1.0),
            coord!(x: 1.0, y: 1.0),
            coord!(x: 2.0, y: 1.0),
            coord!(x: 3.0, y: 1.0),
            coord!(x: 0.0, y: 2.0),
            coord!(x: 1.0, y: 2.5),
            coord!(x: 2.0, y: 2.5),
            coord!(x: 3.0, y: 2.0),
            coord!(x: 0.0, y: 3.0),
            coord!(x: 3.0, y: 3.0),
        ];

        let poly = concave_hull(coords.iter(), 3);
        assert_eq!(poly.exterior().coords_count(), 12);

        let must_not_be_in = vec![&coords[6]];
        for coord in poly.exterior().coords_iter() {
            for not_coord in must_not_be_in.iter() {
                assert_ne!(&coord, *not_coord);
            }
        }
    }

    #[test]
    fn empty_hull() {
        let actual: Polygon<f64> = concave_hull(vec![].iter(), 3);
        let expected = Polygon::new(LineString::new(vec![]), vec![]);
        assert_eq!(actual, expected);
    }
}
