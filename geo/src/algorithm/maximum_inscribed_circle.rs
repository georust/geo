use std::cmp::Ordering;
use std::collections::BinaryHeap;

use rstar::RTree;
use rstar::primitives::CachedEnvelope;

use crate::algorithm::{
    Area, BoundingRect, CoordsIter, Distance, Euclidean, HasDimensions, InteriorPoint, Intersects,
    LinesIter, indexed::IntervalTreeMultiPolygon,
};
use crate::geometry::*;
use crate::{GeoFloat, coord};

/// Calculate the center (holes are a boundary too) of the largest circle inside a `Polygon` or `MultiPolygon`.
/// This is also known as "the pole of inaccessibility".
///
/// Port of JTS's [`MaximumInscribedCircle`](https://github.com/locationtech/jts/blob/3460207/modules/core/src/main/java/org/locationtech/jts/algorithm/construct/MaximumInscribedCircle.java) and [`ExactMaxInscribedCircle`](https://github.com/locationtech/jts/blob/3460207/modules/core/src/main/java/org/locationtech/jts/algorithm/construct/ExactMaxInscribedCircle.java).
/// JTS is copyright Martin Davis and contributors, available under the Eclipse Distribution License 1.0.
/// Triangles and convex quadrilaterals are solved exactly with everything else useing a branch-and-bound grid search.
///
/// # Examples
///
/// ```
/// use approx::assert_relative_eq;
/// use geo::{point, wkt, Area, Contains, MaximumInscribedCircle, Polygon};
///
/// let chevron: Polygon = wkt!(POLYGON((1. 1.,6. 9.,3.7 2.5,9. 1.,1. 1.)));
/// let tolerance = chevron.unsigned_area().sqrt() * 0.1;
/// let center = chevron.maximum_inscribed_circle(tolerance).unwrap();
/// assert!(chevron.contains(&center));
///
/// let triangle: Polygon = wkt!(POLYGON((1. 1.,1. 7.,9. 1.,1. 1.)));
/// let center = triangle.maximum_inscribed_circle(0.001).unwrap();
/// assert_relative_eq!(center, point!(x: 3., y: 3.));
/// ```
pub trait MaximumInscribedCircle<T: GeoFloat> {
    /// Calculate the center to within `tolerance`.
    ///
    /// Returns `None` if
    /// - the geometry is empty,
    /// - `tolerance` is not positive, or
    /// - the coordinates are not finite or too large to square.
    fn maximum_inscribed_circle(&self, tolerance: T) -> Option<Point<T>>;
}

impl<T: GeoFloat> MaximumInscribedCircle<T> for Polygon<T> {
    fn maximum_inscribed_circle(&self, tolerance: T) -> Option<Point<T>> {
        if !is_valid_input(self, tolerance) {
            return None;
        }
        first_if_no_area(self)
            .or_else(|| exact::center(self))
            .or_else(|| {
                branch_and_bound(
                    &MultiPolygon::from(self.clone()),
                    self.interior_point(),
                    tolerance,
                )
            })
    }
}

impl<T: GeoFloat> MaximumInscribedCircle<T> for MultiPolygon<T> {
    fn maximum_inscribed_circle(&self, tolerance: T) -> Option<Point<T>> {
        if !is_valid_input(self, tolerance) {
            return None;
        }
        first_if_no_area(self).or_else(|| branch_and_bound(self, self.interior_point(), tolerance))
    }
}

fn is_valid_input<T, G>(geometry: &G, tolerance: T) -> bool
where
    T: GeoFloat,
    G: BoundingRect<T, Output = Option<Rect<T>>> + CoordsIter<Scalar = T> + HasDimensions,
{
    tolerance > T::zero()
        && !geometry.is_empty()
        // `InteriorPoint` panics on NaN
        && geometry
            .coords_iter()
            .all(|coord| coord.x.is_finite() && coord.y.is_finite())
// and `RTree::nearest_neighbor` panics on infinite distances.
        && geometry.bounding_rect().is_some_and(|rect| {
            let extent = rect.width().max(rect.height());
            let bound = extent + extent;
            (bound * bound + bound * bound).is_finite()
        })
}

fn first_if_no_area<T, G>(geometry: &G) -> Option<Point<T>>
where
    T: GeoFloat,
    G: Area<T> + CoordsIter<Scalar = T>,
{
    if geometry.unsigned_area() == T::zero() {
        geometry.coords_iter().next().map(Point::from)
    } else {
        None
    }
}

fn branch_and_bound<T: GeoFloat>(
    polygons: &MultiPolygon<T>,
    seed: Option<Point<T>>,
    tolerance: T,
) -> Option<Point<T>> {
    let rect = polygons.bounding_rect()?;
    let boundary = SignedBoundaryDistance::new(polygons);
    let two = T::one() + T::one();
    let rect_center = rect.center();
    let diameter = Euclidean.distance(rect.min(), rect.max());

    let mut queue = BinaryHeap::new();
    let cell_size = rect.width().max(rect.height());
    if cell_size != T::zero() {
        queue.push(Cell::new(rect_center, cell_size / two, &boundary));
    }

    let mut farthest = Cell::new(seed.map_or(rect_center, Coord::from), diameter, &boundary);

    for _ in 0..max_iterations(diameter, tolerance) {
        let Some(cell) = queue.pop() else {
            break;
        };

        if cell.distance > farthest.distance {
            farthest = cell;
        }
        if cell.max_distance - farthest.distance < tolerance {
            break;
        }
        let h = cell.half_side / two;
        queue.extend(
            [(-h, -h), (h, -h), (-h, h), (h, h)]
                .map(|(x, y)| Cell::new(cell.center + coord! { x: x, y: y }, h, &boundary)),
        );
    }
    Some(farthest.center.into())
}

/// Bounds the search on very thin geometries.
fn max_iterations<T: GeoFloat>(diameter: T, tolerance: T) -> i64 {
    let cells = (diameter / tolerance).to_f64().unwrap_or(f64::NAN);
    let factor = i64::from(cells.ln() as i32).max(1);
    2000 + 2000 * factor
}

/// Distance to the nearest ring segment, negative outside.
struct SignedBoundaryDistance<T: GeoFloat> {
    segments: RTree<CachedEnvelope<Line<T>>>,
    area: IntervalTreeMultiPolygon<T>,
}

impl<T: GeoFloat> SignedBoundaryDistance<T> {
    fn new(polygons: &MultiPolygon<T>) -> Self {
        Self {
            segments: RTree::bulk_load(polygons.lines_iter().map(CachedEnvelope::new).collect()),
            area: IntervalTreeMultiPolygon::new(polygons),
        }
    }

    fn distance(&self, coord: Coord<T>) -> T {
        let distance = self
            .segments
            .nearest_neighbor(Point::from(coord))
            .map_or_else(T::zero, |segment| Euclidean.distance(coord, &**segment));
        // points on the boundary count as inside, as in JTS
        if self.area.intersects(&coord) {
            distance
        } else {
            -distance
        }
    }
}

/// A square cell, ordered by the largest boundary distance any of its points could have.
#[derive(Clone, Copy)]
struct Cell<T: GeoFloat> {
    center: Coord<T>,
    half_side: T,
    distance: T,
    max_distance: T,
}

impl<T: GeoFloat> Cell<T> {
    fn new(center: Coord<T>, half_side: T, boundary: &SignedBoundaryDistance<T>) -> Self {
        let distance = boundary.distance(center);
        let half_diagonal = half_side * T::from(std::f64::consts::SQRT_2).unwrap();
        Self {
            center,
            half_side,
            distance,
            max_distance: distance + half_diagonal,
        }
    }
}

impl<T: GeoFloat> PartialEq for Cell<T> {
    fn eq(&self, other: &Self) -> bool {
        self.cmp(other) == Ordering::Equal
    }
}

impl<T: GeoFloat> Eq for Cell<T> {}

impl<T: GeoFloat> PartialOrd for Cell<T> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl<T: GeoFloat> Ord for Cell<T> {
    fn cmp(&self, other: &Self) -> Ordering {
        self.max_distance.total_cmp(&other.max_distance)
    }
}

/// Exact centers for triangles and convex quadrilaterals.
mod exact {
    use crate::kernels::{Kernel, Orientation, RobustKernel};
    use crate::line_intersection::{LineIntersection, line_intersection};
    use crate::{
        BoundingRect, Coord, Distance, Euclidean, GeoFloat, Intersects, IsConvex, Line, Point,
        Polygon, Winding,
    };

    pub(super) fn center<T: GeoFloat>(polygon: &Polygon<T>) -> Option<Point<T>> {
        if !polygon.interiors().is_empty() {
            return None;
        }
        match polygon.exterior().0.as_slice() {
            &[a, b, c, _] => Some(incenter(a, b, c).into()),
            // allows collinear vertices, as JTS does
            [_, _, _, _, _] if polygon.exterior().is_convex() => {
                convex_quadrilateral(polygon).map(Point::from)
            }
            _ => None,
        }
    }

    /// The intersection of adjacent corner bisectors that is farthest from the boundary.
    fn convex_quadrilateral<T: GeoFloat>(polygon: &Polygon<T>) -> Option<Coord<T>> {
        let ring = polygon.exterior();
        let &[p0, p1, p2, p3, _] = ring.0.as_slice() else {
            return None;
        };
        // clockwise
        let [a, b, c, d] = if ring.is_ccw() {
            [p0, p3, p2, p1]
        } else {
            [p0, p1, p2, p3]
        };

        let rect = ring.bounding_rect()?;
        let diameter = Euclidean.distance(rect.min(), rect.max());
        // expand the bisectors for robustness, as JTS does
        let length = diameter + diameter;

        let [Some(at_a), Some(at_b), Some(at_c), Some(at_d)] =
            [(d, a, b), (a, b, c), (b, c, d), (c, d, a)]
                .map(|(previous, corner, next)| corner_bisector(previous, corner, next, length))
        else {
            return None;
        };

        [(at_a, at_b), (at_b, at_c), (at_c, at_d), (at_d, at_a)]
            .into_iter()
            .filter_map(|(first, second)| match line_intersection(first, second)? {
                LineIntersection::SinglePoint { intersection, .. } => Some(intersection),
                LineIntersection::Collinear { intersection } => Some(intersection.start),
            })
            // the boundary counts as inside
            .filter(|&node| polygon.intersects(&node))
            .map(|node| (node, Euclidean.distance(&Point::from(node), ring)))
            // keeps the first of equally distant nodes, as JTS does
            .reduce(|best, candidate| {
                if candidate.1 > best.1 {
                    candidate
                } else {
                    best
                }
            })
            .map(|(node, _)| node)
    }

    fn corner_bisector<T: GeoFloat>(
        previous: Coord<T>,
        corner: Coord<T>,
        next: Coord<T>,
        length: T,
    ) -> Option<Line<T>> {
        if RobustKernel::orient2d(previous, corner, next) == Orientation::CounterClockwise {
            return None;
        }
        let pi = T::from(std::f64::consts::PI)?;
        let tau = pi + pi;
        let two = T::one() + T::one();
        // JTS Angle.bisector and Angle.angleBetweenOriented
        let to_previous = angle(corner, previous);
        let mut delta = angle(corner, next) - to_previous;
        if delta <= -pi {
            delta = delta + tau;
        } else if delta > pi {
            delta = delta - tau;
        }
        let mut bisector = to_previous + delta / two;
        // JTS Angle.normalize
        while bisector > pi {
            bisector = bisector - tau;
        }
        while bisector <= -pi {
            bisector = bisector + tau;
        }
        // JTS Angle.project
        let snap_threshold = T::from(5e-16)?;
        let snap = |value: T| {
            if value.abs() < snap_threshold {
                T::zero()
            } else {
                value
            }
        };
        let end = Coord {
            x: corner.x + length * snap(bisector.cos()),
            y: corner.y + length * snap(bisector.sin()),
        };
        Some(Line::new(corner, end))
    }

    fn angle<T: GeoFloat>(from: Coord<T>, to: Coord<T>) -> T {
        (to.y - from.y).atan2(to.x - from.x)
    }

    fn incenter<T: GeoFloat>(a: Coord<T>, b: Coord<T>, c: Coord<T>) -> Coord<T> {
        let len_ab = Euclidean.distance(a, b);
        let len_bc = Euclidean.distance(b, c);
        let len_ca = Euclidean.distance(c, a);
        let perimeter = len_bc + len_ca + len_ab;
        (a * len_bc + b * len_ca + c * len_ab) / perimeter
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::{
        AffineOps, AffineTransform, Contains, Distance, Euclidean, LineString, LinesIter, point,
        polygon, wkt,
    };

    fn boundary_distance<'a, G: LinesIter<'a, Scalar = f64>>(
        geometry: &'a G,
        center: Point,
    ) -> f64 {
        geometry
            .lines_iter()
            .map(|line| Euclidean.distance(&line, &center))
            .fold(f64::INFINITY, f64::min)
    }

    #[test]
    fn empty_input_has_no_center() {
        let empty_polygon: Polygon = wkt!(POLYGON EMPTY);
        assert_eq!(empty_polygon.maximum_inscribed_circle(1.), None);

        let empty_multi_polygon: MultiPolygon = wkt!(MULTIPOLYGON EMPTY);
        assert_eq!(empty_multi_polygon.maximum_inscribed_circle(1.), None);

        let only_empty_members = MultiPolygon::new(vec![Polygon::empty(), Polygon::empty()]);
        assert_eq!(only_empty_members.maximum_inscribed_circle(1.), None);
    }

    #[test]
    fn tolerance_must_be_positive() {
        let square: Polygon = wkt!(POLYGON((0. 0.,10. 0.,10. 10.,0. 10.,0. 0.)));
        assert_eq!(square.maximum_inscribed_circle(0.), None);
        assert_eq!(square.maximum_inscribed_circle(-0.), None);
        assert_eq!(square.maximum_inscribed_circle(-1.), None);
        assert_eq!(square.maximum_inscribed_circle(f64::NAN), None);
        assert_eq!(square.maximum_inscribed_circle(f64::NEG_INFINITY), None);

        let squares: MultiPolygon = wkt!(MULTIPOLYGON(((0. 0.,10. 0.,10. 10.,0. 10.,0. 0.))));
        assert_eq!(squares.maximum_inscribed_circle(0.), None);
        assert_eq!(squares.maximum_inscribed_circle(-1.), None);
        assert_eq!(squares.maximum_inscribed_circle(f64::NAN), None);
    }

    // JTS testCollapsedLine
    #[test]
    fn collapsed_line_returns_first_coordinate() {
        let collapsed_line: Polygon = wkt!(POLYGON((100. 100.,200. 200.,100. 100.,100. 100.)));

        let center = collapsed_line.maximum_inscribed_circle(0.01).unwrap();

        assert_relative_eq!(center, point!(x: 100., y: 100.));
    }

    // JTS testCollapsedLineFlat
    #[test]
    fn collapsed_flat_line_returns_first_coordinate() {
        let collapsed_line: Polygon = wkt!(POLYGON((1. 2.,1. 2.,1. 2.,1. 2.,3. 2.,1. 2.)));

        let center = collapsed_line.maximum_inscribed_circle(0.01).unwrap();

        assert_relative_eq!(center, point!(x: 1., y: 2.));
    }

    // JTS testCollapsedPoint
    #[test]
    fn collapsed_point_returns_first_coordinate() {
        let collapsed_point: Polygon = wkt!(POLYGON((100. 100.,100. 100.,100. 100.,100. 100.)));

        let center = collapsed_point.maximum_inscribed_circle(0.01).unwrap();

        assert_relative_eq!(center, point!(x: 100., y: 100.));
    }

    #[test]
    fn flat_multi_polygon_returns_first_coordinate() {
        let flat: MultiPolygon = wkt!(MULTIPOLYGON(
            EMPTY,
            ((5. 5.,6. 6.,7. 7.,5. 5.)),
            ((0. 0.,1. 0.,2. 0.,0. 0.))
        ));

        let center = flat.maximum_inscribed_circle(0.01).unwrap();

        assert_relative_eq!(center, point!(x: 5., y: 5.));
    }

    // JTS testTriangleRight
    #[test]
    fn right_triangle() {
        let triangle: Polygon = wkt!(POLYGON((1. 1.,1. 7.,9. 1.,1. 1.)));

        let center = triangle.maximum_inscribed_circle(0.001).unwrap();

        assert_relative_eq!(center, point!(x: 3., y: 3.), epsilon = 1e-12);
        assert_relative_eq!(boundary_distance(&triangle, center), 2., epsilon = 1e-12);
    }

    // JTS testTriangleObtuse
    #[test]
    fn obtuse_triangle() {
        let triangle: Polygon = wkt!(POLYGON((1. 1.,1. 9.,2. 2.,1. 1.)));

        let center = triangle.maximum_inscribed_circle(0.001).unwrap();

        assert_relative_eq!(
            center,
            point!(x: 1.4852813742385702, y: 2.17157287525381),
            epsilon = 1e-12
        );
        assert_relative_eq!(
            boundary_distance(&triangle, center),
            0.4852813742385702,
            epsilon = 1e-12
        );
    }

    // JTS testSquare
    #[test]
    fn square() {
        let square: Polygon = wkt!(POLYGON((100. 200.,200. 200.,200. 100.,100. 100.,100. 200.)));

        let center = square.maximum_inscribed_circle(0.001).unwrap();

        assert_relative_eq!(center, point!(x: 150., y: 150.), epsilon = 1e-9);
        assert_relative_eq!(boundary_distance(&square, center), 50., epsilon = 1e-9);
    }

    // JTS testThinQuad
    #[test]
    fn thin_quad() {
        let quad: Polygon = wkt!(POLYGON((1. 2.,9. 3.,9. 1.,1. 1.,1. 2.)));

        let center = quad.maximum_inscribed_circle(0.001).unwrap();

        assert_relative_eq!(
            center,
            point!(x: 8.06225774829855, y: 1.9377422517014502),
            epsilon = 1e-9
        );
        assert_relative_eq!(
            boundary_distance(&quad, center),
            0.937742251701450,
            epsilon = 1e-9
        );
    }

    // JTS testDiamond
    #[test]
    fn diamond() {
        let diamond: Polygon = wkt!(POLYGON((150. 250.,50. 150.,150. 50.,250. 150.,150. 250.)));

        let center = diamond.maximum_inscribed_circle(0.001).unwrap();

        assert_relative_eq!(center, point!(x: 150., y: 150.), epsilon = 1e-9);
        assert_relative_eq!(
            boundary_distance(&diamond, center),
            70.71067811865476,
            epsilon = 1e-9
        );
    }

    // JTS testKite
    #[test]
    fn kite() {
        let kite: Polygon = wkt!(POLYGON((100. 0.,200. 200.,300. 200.,300. 100.,100. 0.)));

        let center = kite.maximum_inscribed_circle(0.01).unwrap();

        assert_relative_eq!(
            center,
            point!(x: 238.19660112501052, y: 138.19660112501052),
            epsilon = 1e-9
        );
        assert_relative_eq!(
            boundary_distance(&kite, center),
            61.803398874989476,
            epsilon = 1e-9
        );
    }

    // JTS testKite with the ring reversed; JTS gives the same center
    #[test]
    fn counter_clockwise_kite() {
        let kite: Polygon = wkt!(POLYGON((100. 0.,300. 100.,300. 200.,200. 200.,100. 0.)));

        let center = kite.maximum_inscribed_circle(0.01).unwrap();

        assert_relative_eq!(
            center,
            point!(x: 238.19660112501052, y: 138.19660112501052),
            epsilon = 1e-9
        );
        assert_relative_eq!(
            boundary_distance(&kite, center),
            61.803398874989476,
            epsilon = 1e-9
        );
    }

    // JTS testQuadWithCollinearVertex
    #[test]
    fn quad_with_collinear_vertex() {
        let quad: Polygon = wkt!(POLYGON((1. 5.,5. 5.,9. 5.,5. 1.,1. 5.)));

        let center = quad.maximum_inscribed_circle(0.001).unwrap();

        assert_relative_eq!(center, point!(x: 5., y: 3.34314575050762), epsilon = 1e-9);
        assert_relative_eq!(
            boundary_distance(&quad, center),
            1.6568542494923801,
            epsilon = 1e-9
        );
    }

    #[test]
    fn quad_with_collinear_vertex_on_the_y_axis() {
        let quad: Polygon = wkt!(POLYGON((-4. 0.,0. 0.,4. 0.,0. -4.,-4. 0.)));

        let center = quad.maximum_inscribed_circle(0.001).unwrap();

        // the bisector at (0 0) points straight down; JTS snaps its direction to exactly
        // vertical, so the center is exactly on x = 0 rather than 1e-16 off it
        assert_relative_eq!(center, point!(x: 0., y: -1.6568542494923804), epsilon = 0.);
    }

    #[test]
    fn quad_with_nearly_collinear_vertex() {
        let quad: Polygon = wkt!(POLYGON((1. 5.,5. 5.,9. 4.999999998,5. 1.,1. 5.)));

        let center = quad.maximum_inscribed_circle(0.001).unwrap();

        // JTS: the bisector at (5 5) is 2.5e-10 radians off vertical, which moves the center
        // 2.9e-10 off x = 5
        assert_relative_eq!(
            center,
            point!(x: 5.000000000292894, y: 3.343145750386299),
            epsilon = 1e-12
        );
        assert_relative_eq!(
            boundary_distance(&quad, center),
            1.6568542496137002,
            epsilon = 1e-12
        );
    }

    // JTS testChevron
    #[test]
    fn chevron() {
        let chevron: Polygon = wkt!(POLYGON((1. 1.,6. 9.,3.7 2.5,9. 1.,1. 1.)));

        let center = chevron.maximum_inscribed_circle(0.001).unwrap();

        assert!(chevron.contains(&center));
        assert_relative_eq!(center, point!(x: 2.82, y: 2.008), epsilon = 0.002);
        assert_relative_eq!(boundary_distance(&chevron, center), 1.008, epsilon = 0.002);
    }

    // JTS testChevronFat
    #[test]
    fn chevron_fat() {
        let chevron: Polygon = wkt!(POLYGON((1. 1.,6. 9.,5.9 5.,9. 1.,1. 1.)));

        let center = chevron.maximum_inscribed_circle(0.001).unwrap();

        assert!(chevron.contains(&center));
        assert_relative_eq!(center, point!(x: 4.7545, y: 3.0809), epsilon = 0.002);
        assert_relative_eq!(boundary_distance(&chevron, center), 2.081, epsilon = 0.002);
    }

    // JTS testCircle: POINT (100 100) buffered by 100
    #[test]
    fn circle() {
        #[rustfmt::skip]
        let circle: Polygon = wkt!(POLYGON((200. 100.,199.6917333733128 92.15409042721551,198.76883405951378 84.35655349597691,197.23699203976764 76.65546361440946,195.10565162951536 69.09830056250526,192.3879532511287 61.731656763491024,189.10065241883677 54.600950026045325,185.26401643540922 47.75014352840512,180.90169943749476 41.221474770752685,176.0405965600031 35.05519516698163,170.71067811865476 29.28932188134526,164.94480483301837 23.959403439996905,158.77852522924732 19.09830056250526,152.24985647159488 14.735983564590782,145.39904997395467 10.899347581163227,138.268343236509 7.612046748871322,130.90169943749476 4.89434837048465,123.34453638559054 2.7630079602323434,115.64344650402309 1.2311659404862212,107.8459095727845 0.3082666266871996,100. 0.,92.15409042721551 0.3082666266871996,84.35655349597693 1.2311659404862212,76.65546361440947 2.763007960232329,69.09830056250527 4.894348370484636,61.731656763491024 7.612046748871322,54.60095002604533 10.899347581163212,47.75014352840512 14.735983564590768,41.2214747707527 19.09830056250526,35.05519516698165 23.95940343999689,29.28932188134526 29.289321881345245,23.959403439996905 35.05519516698162,19.098300562505273 41.22147477075268,14.735983564590782 47.75014352840511,10.899347581163227 54.60095002604531,7.612046748871322 61.73165676349101,4.89434837048465 69.09830056250524,2.7630079602323434 76.65546361440946,1.2311659404862354 84.3565534959769,0.3082666266871996 92.1540904272155,0. 100.,0.3082666266871996 107.84590957278448,1.2311659404862212 115.64344650402307,2.763007960232329 123.34453638559053,4.894348370484636 130.90169943749473,7.612046748871322 138.26834323650897,10.899347581163212 145.39904997395467,14.735983564590768 152.24985647159485,19.098300562505244 158.7785252292473,23.95940343999689 164.94480483301834,29.28932188134523 170.71067811865476,35.05519516698159 176.04059656000305,41.22147477075268 180.90169943749473,47.75014352840514 185.26401643540925,54.60095002604531 189.10065241883677,61.73165676349097 192.38795325112864,69.09830056250524 195.10565162951536,76.65546361440948 197.23699203976767,84.3565534959769 198.76883405951378,92.15409042721544 199.6917333733128,100. 200.,107.84590957278452 199.6917333733128,115.64344650402307 198.76883405951378,123.34453638559047 197.23699203976767,130.90169943749473 195.10565162951536,138.268343236509 192.38795325112866,145.39904997395467 189.10065241883677,152.24985647159485 185.26401643540925,158.7785252292473 180.90169943749476,164.9448048330184 176.04059656000308,170.71067811865476 170.71067811865476,176.04059656000305 164.94480483301842,180.90169943749473 158.77852522924735,185.26401643540925 152.24985647159485,189.10065241883677 145.3990499739547,192.38795325112864 138.26834323650905,195.10565162951536 130.90169943749476,197.23699203976767 123.34453638559052,198.76883405951378 115.64344650402312,199.6917333733128 107.84590957278456,200. 100.)));

        let center = circle.maximum_inscribed_circle(0.01).unwrap();

        assert_relative_eq!(center, point!(x: 100., y: 100.), epsilon = 0.02);
        assert_relative_eq!(boundary_distance(&circle, center), 99.92, epsilon = 0.02);
    }

    // JTS testKiteWithHole
    #[test]
    fn kite_with_hole() {
        let kite: Polygon = wkt!(POLYGON(
            (100. 0.,200. 200.,300. 200.,300. 100.,100. 0.),
            (200. 150.,200. 100.,260. 100.,200. 150.)
        ));

        let center = kite.maximum_inscribed_circle(0.01).unwrap();

        assert!(kite.contains(&center));
        assert_relative_eq!(center, point!(x: 257.47, y: 157.47), epsilon = 0.02);
        assert_relative_eq!(boundary_distance(&kite, center), 42.529, epsilon = 0.02);
    }

    // JTS testDoubleKite
    #[test]
    fn double_kite() {
        let kites: MultiPolygon = wkt!(MULTIPOLYGON(
            ((150. 200.,100. 150.,150. 100.,250. 150.,150. 200.)),
            ((400. 250.,300. 150.,400. 50.,560. 150.,400. 250.))
        ));

        let center = kites.maximum_inscribed_circle(0.01).unwrap();

        assert!(kites.contains(&center));
        assert_relative_eq!(center, point!(x: 411.38, y: 149.99), epsilon = 0.02);
        assert_relative_eq!(boundary_distance(&kites, center), 78.75, epsilon = 0.02);
    }

    #[test]
    fn multi_polygon_ignores_empty_members() {
        let kites: MultiPolygon = wkt!(MULTIPOLYGON(
            EMPTY,
            ((150. 200.,100. 150.,150. 100.,250. 150.,150. 200.)),
            EMPTY,
            ((400. 250.,300. 150.,400. 50.,560. 150.,400. 250.))
        ));

        let center = kites.maximum_inscribed_circle(0.01).unwrap();

        assert!(kites.contains(&center));
        assert_relative_eq!(center, point!(x: 411.38, y: 149.99), epsilon = 0.02);
        assert_relative_eq!(boundary_distance(&kites, center), 78.75, epsilon = 0.02);
    }

    #[test]
    fn multi_polygon_of_one_triangle_uses_the_search() {
        let triangle: MultiPolygon = wkt!(MULTIPOLYGON(((1. 1.,1. 7.,9. 1.,1. 1.))));

        let center = triangle.maximum_inscribed_circle(0.001).unwrap();

        // the incenter is (3 3) with radius 2, found to within the tolerance
        assert!(triangle.contains(&center));
        assert_relative_eq!(center, point!(x: 3., y: 3.), epsilon = 0.002);
        assert_relative_eq!(boundary_distance(&triangle, center), 2., epsilon = 0.001);
    }

    // JTS testTriangleRight and testChevron, shrunk to building size in Web Mercator coordinates
    #[test]
    fn tiny_shapes_far_from_origin() {
        let shrink = AffineTransform::scale(1e-7, 1e-7, (0., 0.)).translated(0.5, 0.3);

        let triangle: Polygon = wkt!(POLYGON((1. 1.,1. 7.,9. 1.,1. 1.))).affine_transform(&shrink);
        let center = triangle.maximum_inscribed_circle(1e-8).unwrap();
        assert_relative_eq!(center, point!(x: 0.5000003, y: 0.3000003), epsilon = 1e-15);
        assert_relative_eq!(boundary_distance(&triangle, center), 2e-7, epsilon = 1e-15);

        let chevron: Polygon =
            wkt!(POLYGON((1. 1.,6. 9.,3.7 2.5,9. 1.,1. 1.))).affine_transform(&shrink);
        let center = chevron.maximum_inscribed_circle(1e-10).unwrap();
        assert!(chevron.contains(&center));
        assert_relative_eq!(
            center,
            point!(x: 0.500000282, y: 0.3000002008),
            epsilon = 2e-10
        );
        assert_relative_eq!(
            boundary_distance(&chevron, center),
            1.008e-7,
            epsilon = 2e-10
        );
    }

    // JTS testNearlyFlat (https://github.com/libgeos/geos/issues/875)
    #[test]
    fn nearly_flat() {
        let polygon: Polygon = wkt!(POLYGON((
            59.3 100.00000000000001,99.7 100.00000000000001,99.7 100.,59.3 100.,59.3 100.00000000000001
        )));

        let center = polygon.maximum_inscribed_circle(0.01).unwrap();

        // like JTS, only check that the search terminates near the boundary
        assert!(boundary_distance(&polygon, center) < 0.02);
    }

    // JTS testVeryThin
    #[test]
    fn very_thin() {
        let polygon: Polygon = wkt!(POLYGON((
            100. 100.,200. 300.,300. 100.,450. 250.,300. 99.999999,200. 299.99999,100. 100.
        )));

        let center = polygon.maximum_inscribed_circle(0.01).unwrap();

        // like JTS, only check that the search terminates near the boundary
        assert!(boundary_distance(&polygon, center) < 0.02);
    }

    #[test]
    fn self_intersecting_polygon_does_not_panic() {
        let bowtie: Polygon = wkt!(POLYGON((0. 0.,10. 10.,10. 0.,0. 10.,0. 0.)));
        assert!(bowtie.maximum_inscribed_circle(0.01).is_some());
    }

    #[test]
    fn hole_outside_shell_does_not_panic() {
        let polygon: Polygon = wkt!(POLYGON(
            (0. 0.,10. 0.,10. 10.,0. 10.,0. 0.),
            (20. 20.,30. 20.,30. 30.,20. 20.)
        ));
        assert!(polygon.maximum_inscribed_circle(0.01).is_some());
    }

    #[test]
    fn concave_quad_uses_the_search() {
        let quad: Polygon = wkt!(POLYGON((0. 0.,10. 0.,1. 1.,0. 10.,0. 0.)));

        let center = quad.maximum_inscribed_circle(0.01).unwrap();

        // JTS: center 0.5859375 0.5859375, radius 0.5855728031701096
        assert!(quad.contains(&center));
        assert_relative_eq!(center, point!(x: 0.5859375, y: 0.5859375), epsilon = 0.02);
        assert_relative_eq!(
            boundary_distance(&quad, center),
            0.5855728031701096,
            epsilon = 0.01
        );
    }

    #[test]
    fn non_finite_coordinates_have_no_center() {
        let nan_triangle = polygon![(x: 0., y: 0.), (x: f64::NAN, y: 0.), (x: 0., y: 10.)];
        assert_eq!(nan_triangle.maximum_inscribed_circle(0.01), None);

        let infinite_quad = polygon![
            (x: 0., y: 0.),
            (x: 10., y: 0.),
            (x: f64::INFINITY, y: 10.),
            (x: 0., y: 10.),
        ];
        assert_eq!(infinite_quad.maximum_inscribed_circle(0.01), None);

        let negative_infinite_hexagon = polygon![
            (x: 0., y: 0.),
            (x: 10., y: 0.),
            (x: 12., y: 5.),
            (x: f64::NEG_INFINITY, y: 10.),
            (x: 0., y: 10.),
            (x: -2., y: 5.),
        ];
        assert_eq!(
            negative_infinite_hexagon.maximum_inscribed_circle(0.01),
            None
        );

        let nan_hexagons = MultiPolygon::new(vec![polygon![
            (x: 0., y: 0.),
            (x: 10., y: 0.),
            (x: 12., y: 5.),
            (x: f64::NAN, y: 10.),
            (x: 0., y: 10.),
            (x: -2., y: 5.),
        ]]);
        assert_eq!(nan_hexagons.maximum_inscribed_circle(0.01), None);
    }

    #[test]
    fn overflowing_extent_has_no_center() {
        let max = f64::MAX;
        let polygon = Polygon::new(
            LineString::from(vec![
                (-max, -1.),
                (max, -1.),
                (max, 1.),
                (0., 0.),
                (-max, 1.),
            ]),
            vec![],
        );
        assert_eq!(polygon.maximum_inscribed_circle(1.), None);
    }

    #[test]
    fn extent_with_overflowing_squared_distances_has_no_center() {
        let big = 1e200;
        let hexagon = Polygon::new(
            LineString::from(vec![
                (0., 0.),
                (big, 0.),
                (big * 1.1, big * 0.5),
                (big, big),
                (0., big),
                (-big * 0.1, big * 0.5),
                (0., 0.),
            ]),
            vec![],
        );
        assert_eq!(hexagon.maximum_inscribed_circle(big * 0.01), None);
        let wide = 1e300;
        let sliver = Polygon::new(
            LineString::from(vec![
                (-wide, -1.),
                (wide, -1.),
                (wide, 1.),
                (0., 0.5),
                (-wide, 1.),
            ]),
            vec![],
        );
        assert_eq!(sliver.maximum_inscribed_circle(1.), None);
    }

    #[test]
    fn large_extent_has_center() {
        let big = 1e150;
        let hexagon = Polygon::new(
            LineString::from(vec![
                (0., 0.),
                (big, 0.),
                (big * 1.1, big * 0.5),
                (big, big),
                (0., big),
                (-big * 0.1, big * 0.5),
                (0., 0.),
            ]),
            vec![],
        );
        let center = hexagon.maximum_inscribed_circle(big * 0.01).unwrap();
        assert!(hexagon.contains(&center));
    }

    #[test]
    fn extreme_tolerances_do_not_panic() {
        let chevron: Polygon = wkt!(POLYGON((1. 1.,6. 9.,3.7 2.5,9. 1.,1. 1.)));
        assert!(chevron.maximum_inscribed_circle(f64::INFINITY).is_some());
        assert!(chevron.maximum_inscribed_circle(f64::MAX).is_some());
        assert!(
            chevron
                .maximum_inscribed_circle(1e-3 * f64::EPSILON)
                .is_some()
        );
    }

    #[test]
    fn f32_input() {
        let triangle: Polygon<f32> = wkt!(POLYGON((1. 1.,1. 7.,9. 1.,1. 1.)));
        let center = triangle.maximum_inscribed_circle(0.001).unwrap();
        assert_relative_eq!(center, point!(x: 3., y: 3.));

        let chevron: Polygon<f32> = wkt!(POLYGON((1. 1.,6. 9.,3.7 2.5,9. 1.,1. 1.)));
        let center = chevron.maximum_inscribed_circle(0.001).unwrap();
        assert!(chevron.contains(&center));
        assert_relative_eq!(center, point!(x: 2.82, y: 2.008), epsilon = 0.002);
    }
}
