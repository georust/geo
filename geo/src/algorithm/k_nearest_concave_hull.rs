use crate::{Point, Polygon, LineString, CoordNum, GeoFloat, MultiPoint, Coordinate};
use std::cmp::max;
use crate::algorithm::contains::Contains;
use crate::algorithm::intersects::Intersects;
use rstar::RTreeNum;
use crate::algorithm::convex_hull::ConvexHull;
use num_traits::Float;

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
    where T: GeoFloat + RTreeNum
{
    type Scalar = T;
    fn k_nearest_concave_hull(&self, k: u32) -> Polygon<Self::Scalar> {
        concave_hull(self.clone(), k)
    }
}

impl<T> KNearestConcaveHull for [Point<T>]
    where T: GeoFloat + RTreeNum
{
    type Scalar = T;
    fn k_nearest_concave_hull(&self, k: u32) -> Polygon<Self::Scalar> {
        concave_hull(Vec::from(self), k)
    }
}

impl<T> KNearestConcaveHull for MultiPoint<T>
    where T: GeoFloat + RTreeNum
{
    type Scalar = T;
    fn k_nearest_concave_hull(&self, k: u32) -> Polygon<Self::Scalar> {
        concave_hull(self.iter().map(|x| x.clone()).collect(), k)
    }
}

fn concave_hull<T>(points: Vec<Point<T>>, k: u32) -> Polygon<T>
    where T: GeoFloat + RTreeNum
{
    let dataset = prepare_dataset(&points);
    concave_hull_inner(dataset, k)
}

const DELTA: f32 = 0.000000001;

// Removes duplicate points from the dataset.
fn prepare_dataset<T>(points: &Vec<Point<T>>) -> rstar::RTree<Point<T>>
    where T: GeoFloat + RTreeNum
{
    let mut dataset: rstar::RTree<Point<T>> = rstar::RTree::new();
    for point in points {
        let closest = dataset.nearest_neighbor(point);
        if let Some(closest) = closest {
            if points_are_equal(point, closest) {
                continue;
            }
        }
        
        dataset.insert(point.clone())
    }

    dataset
}

fn points_are_equal<T>(p1: &Point<T>, p2: &Point<T>) -> bool
    where T: GeoFloat + RTreeNum
{
    float_equal(p1.x(), p2.x()) && float_equal(p1.x(), p2.y())
}

fn float_equal<T>(a: T, b: T) -> bool
    where T: GeoFloat
{
    let da = a * T::from(DELTA).expect("Conversion from constant is always valid.");
    b > (a - da) && b < (a + da)
}

fn polygon_from_tree<T>(dataset: &rstar::RTree<Point<T>>) -> Polygon<T>
    where T: GeoFloat + RTreeNum
{
    assert!(dataset.size() <= 3);

    let mut points: Vec<Coordinate<T>> = dataset.iter().map(|p| p.0).collect();
    points.push(points[0]);
    
    return Polygon::new(
        LineString::from(points),
        vec![],
    )
}

fn concave_hull_inner<T>(original_dataset: rstar::RTree<Point<T>>, k: u32) -> Polygon<T>
    where T: GeoFloat + RTreeNum    
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

    let first_point = get_first_point(&dataset);
    let mut hull = vec![first_point];

    let mut current_point = first_point;
    dataset.remove(&first_point);
    
    let mut prev_angle = T::zero();
    let mut curr_step = 2;
    while (current_point != first_point || curr_step == 2) && dataset.size() > 0 {
        if curr_step == 5 {
            dataset.insert(first_point);
        }

        let mut nearest_points = get_nearest_points(&dataset, &current_point, k_adjusted);
        sort_by_angle(&mut nearest_points, &current_point, prev_angle);

        let selected = nearest_points.iter().find(|x| !intersects(&hull, &[&current_point, x]));

        if let Some(sel) = selected {
            current_point = **sel;
            hull.push(current_point);
            prev_angle = get_angle(&[&hull[hull.len() - 1], &hull[hull.len() - 2]]);
            dataset.remove(&current_point);

            curr_step += 1;
        } else {
            return concave_hull_inner(original_dataset, get_next_k(k_adjusted));
        }
    }

    let poly = Polygon::new(LineString::from(hull), vec![]);

    if original_dataset.iter().any(|&p| !point_inside(&p, &poly)) {
        return concave_hull_inner(original_dataset, get_next_k(k_adjusted));
    }

    poly
}

fn fall_back_hull<T>(dataset: &rstar::RTree<Point<T>>) -> Polygon<T>
where T: GeoFloat + RTreeNum
{
    let multipoint = MultiPoint::from(dataset.iter().map(|p| p.clone()).collect::<Vec<Point<T>>>());
    multipoint.convex_hull()
}

fn get_next_k(curr_k: u32) -> u32 {
    max(curr_k + 1, ((curr_k as f32) * K_MULTIPLIER) as u32)
}

fn adjust_k(k: u32) -> u32 {
    max(k, 3)
}

fn get_first_point<T>(point_set: &rstar::RTree<Point<T>>) -> Point<T>
    where T: GeoFloat + RTreeNum
{
    let mut min_y = Float::max_value();
    let mut result = point_set.iter().next().expect("We checked that there are more then 3 points in the set before.");

    for p in point_set.iter() {
        if p.y() < min_y {
            min_y = p.y();
            result = p;
        }
    }

    *result
}

fn get_nearest_points<'a, T>(dataset: &'a rstar::RTree<Point<T>>, base_point: &Point<T>, candidate_no: u32) -> Vec<&'a Point<T>>
where T: GeoFloat + RTreeNum
{
    dataset.nearest_neighbor_iter(base_point).take(candidate_no as usize).collect()
}

fn sort_by_angle<'a, T>(points: &'a mut Vec<&Point<T>>, curr_point: &Point<T>, prev_angle: T)
where T: GeoFloat
{
    points.sort_by(|a, b| {
        let mut angle_a = get_angle(&[curr_point, a]) - prev_angle;
        let mut angle_b = get_angle(&[curr_point, b]) - prev_angle;
        if angle_a < T::zero() { angle_a = angle_a + two_pi(); }
        if angle_b < T::zero() { angle_b = angle_b + two_pi(); }
        angle_a.partial_cmp(&angle_b).unwrap().reverse()
    });
}

fn intersects<T>(hull: &Vec<Point<T>>, line: &[&Point<T>; 2]) -> bool
where T: GeoFloat
{
    // This is the case of finishing the contour.
    if *line[1] == hull[0] { return false; }

    let points = hull.iter().take(hull.len() - 1).map(|x| crate::Coordinate::from(x.clone())).collect();
    let linestring = LineString(points);
    let line = crate::Line::new(*line[0], *line[1]);
    linestring.intersects(&line)
}

fn get_angle<T>(line: &[&Point<T>; 2]) -> T
where T: GeoFloat
{
    let x1 = line[0].x();
    let y1 = line[0].y();
    let x2 = line[1].x();
    let y2 = line[1].y();

    let a = x2 - x1;
    let b = y2 - y1;
    let c = (a*a + b*b).sqrt();
    if c == T::zero() {
        return T::zero();
    }

    let cos = a / c;
    let mut acos = cos.acos();
    if y1 > y2 {
        acos = two_pi::<T>() - acos;
    }

    acos
}

fn point_inside<T>(point: &Point<T>, poly: &Polygon<T>) -> bool
where T: GeoFloat
{
    poly.contains(point) || poly.exterior().contains(point)
}

fn two_pi<T>() -> T
where T: GeoFloat
{
    T::from(std::f64::consts::PI * 2f64).unwrap()
}

#[cfg(test)]
mod tests {
    use test_case::test_case;
    use crate::point;
    use super::*;
    use crate::coords_iter::CoordsIter;
    use num_traits::FloatConst;
    use std::f32::consts::PI;

    #[test_case(0., 0., 1., 0., 0.)]
    #[test_case(0., 0., 0., 1., PI / 2.)]
    #[test_case(0., 0., -1., 0., PI)]
    #[test_case(0., 0., 0., -1., PI / 2. * 3.)]
    #[test_case(0., 0., -1., 1., PI / 4. * 3.)]
    #[test_case(0., 0., -1., -1., PI / 4. * 5.)]
    fn get_angle_test(x1: f32, y1: f32, x2: f32, y2: f32, angle: f32) {
        let p1 = point!(x: x1, y: y1);
        let p2 = point!(x: x2, y: y2);
        assert_eq!(get_angle(&[&p1, &p2]), angle);
    }

    #[test]
    fn point_ordering() {
        let points = vec![
            point!(x: 1.0, y: 1.0),
            point!(x: -1.0, y: 0.0),
            point!(x: 0.0, y: 1.0),
            point!(x: 1.0, y: 0.0),
        ];

        let mut points_mapped: Vec<&Point<f32>> = points.iter().map(|x| x).collect();

        let center = point!(x: 0.0, y: 0.0);
        let angle = FloatConst::FRAC_PI_4();

        let expected = vec![
            &points[3],
            &points[1],
            &points[2],
            &points[0],
        ];

        sort_by_angle(&mut points_mapped, &center, angle);
        assert_eq!(points_mapped, expected);

        let expected = vec![
            &points[1],
            &points[2],
            &points[0],
            &points[3],
        ];
        sort_by_angle(&mut points_mapped, &center, -angle);
        assert_eq!(points_mapped, expected);
    }

    #[test]
    fn get_first_point_test() {
        let points = vec![
            point!(x: 1.0, y: 1.0),
            point!(x: -1.0, y: 0.0),
            point!(x: 0.0, y: 1.0),
            point!(x: 0.0, y: 0.0),
        ];
        let tree = rstar::RTree::bulk_load(points);
        let first = point!(x: -1.0, y: 0.0);

        assert_eq!(get_first_point(&tree), first);
    }

    #[test]
    fn concave_hull_test() {
        let points = vec![
            point!(x: 0.0, y: 0.0),
            point!(x: 1.0, y: 0.0),
            point!(x: 2.0, y: 0.0),
            point!(x: 3.0, y: 0.0),
            point!(x: 0.0, y: 1.0),
            point!(x: 1.0, y: 1.0),
            point!(x: 2.0, y: 1.0),
            point!(x: 3.0, y: 1.0),
            point!(x: 0.0, y: 2.0),
            point!(x: 1.0, y: 2.5),
            point!(x: 2.0, y: 2.5),
            point!(x: 3.0, y: 2.0),
            point!(x: 0.0, y: 3.0),
            point!(x: 3.0, y: 3.0),
        ];

        let poly = concave_hull(points.clone(), 3);
        assert_eq!(poly.exterior().coords_count(), 12);

        let must_not_be_in = vec![&points[6]];
        for p in poly.exterior().points_iter() {
            for not_p in must_not_be_in.iter() {
                assert_ne!(&p, *not_p);
            }
        }
    }
}
