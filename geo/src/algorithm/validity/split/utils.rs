use geo_types::*;

use crate::line_intersection::line_intersection;
use crate::{Area, Contains, GeoFloat, LineIntersection, Scale, Winding};

/// given a linestring, returns a function which takes two points as an argument
///
/// The returned function will return all the points between the two argument points also including
/// these two argument points at the start and end
pub(crate) fn iter_between<F: GeoFloat>(
    ls: LineString<F>,
) -> impl Fn([Point<F>; 2]) -> Vec<Coord<F>> {
    move |[a, b]| {
        // start
        std::iter::once(a.0)
            // between
            .chain(
                ls.0.iter()
                    .cycle()
                    .skip_while(move |&&c| c != a.0)
                    .skip(1)
                    .take_while(move |&&c| c != b.0)
                    .cloned(),
            )
            // end
            .chain(std::iter::once(b.0))
            .collect::<Vec<_>>()
    }
}

/// Creates a `LineString` between two points with custom strategies for each half.
///
/// A strategy is a function that collects all the points between it's two argument points.
///
/// # Parameters
/// - `first_half_maker`: Closure for the first half strategy.
/// - `second_half_maker`: Closure for the second half strategy.
///
/// # Returns
/// A closure generating a `LineString` from two points using the specified strategies.
pub(crate) fn create_linestring_between_points<F, FN>(
    first_half_maker: FN,
    second_half_maker: FN,
) -> impl Fn([Point<F>; 2]) -> LineString<F>
where
    F: GeoFloat,
    FN: Fn([Point<F>; 2]) -> Vec<Coord<F>>,
{
    move |[first_p, second_p]: [Point<F>; 2]| {
        let ext_part_1 = first_half_maker([first_p; 2]);
        let ext_part_2 = second_half_maker([second_p; 2]);
        let mut ext_coords = [ext_part_1.clone(), ext_part_2.clone()].concat();
        ext_coords.dedup();

        LineString::new(ext_coords)
    }
}

/// given N polygons and a collection of potential holes, looks at which holes fit into which
/// polygon and assignes the holes correctly
pub(crate) fn reassociate_holes<const N: usize, F: GeoFloat>(
    mut polys: [Polygon<F>; N],
    holes: impl IntoIterator<Item = LineString<F>>,
) -> [Polygon<F>; N] {
    holes.into_iter().for_each(|hole| {
        if let Some(poly) = polys
            .iter_mut()
            .filter(|poly| poly.unsigned_area() > F::zero())
            .find(|poly| {
                hole.0
                    .iter()
                    .all(|p| poly.scale(F::from(0.99).unwrap()).contains(p))
            })
        {
            poly.interiors_push(hole);
        }
    });
    polys
}

/// remove the specified hole from the polygon and return the
///
/// - the polygon without that hole
/// - the hole (optionally, if it exists)
pub(crate) fn remove_nth_hole<F: GeoFloat>(
    p: &Polygon<F>,
    i: usize,
) -> Option<(Polygon<F>, LineString<F>)> {
    let hole = p.interiors().iter().nth(i).cloned()?;
    let p = Polygon::new(
        p.exterior().clone(),
        p.interiors()
            .iter()
            .take(i)
            .chain(p.interiors().iter().skip(i + 1))
            .cloned()
            .collect::<Vec<_>>(),
    );
    Some((p, hole))
}

/// finds a polygon fulfilling the predicate given to this function and removes it from the
/// MultiPolygon
pub(crate) fn find_and_remove_polygon<F: GeoFloat>(
    mp: &mut MultiPolygon<F>,
    predicate: impl Fn(&Polygon<F>) -> bool,
) -> Option<Polygon<F>> {
    let idx = mp.iter().position(predicate)?;
    Some(mp.0.remove(idx))
}

/// remove the two specified holes from the polygon and return the
///
/// - the polygon without that hole
/// - the holes (optionally, if two exists)
///
/// this function additionally takes care of removing the holes in the right order, to prevent
/// index invalidation
pub(crate) fn remove_two_holes<F: GeoFloat>(
    poly: &Polygon<F>,
    idxs: [usize; 2],
) -> Option<(Polygon<F>, LineString<F>, LineString<F>)> {
    let [a, b] = idxs;

    let max = a.max(b);
    let min = a.min(b);

    let (poly, other_hole) = remove_nth_hole(poly, max)?;
    let (poly, first_hole) = remove_nth_hole(&poly, min)?;

    Some((poly, other_hole, first_hole))
}

/// predicate to check if a point is both present in the exterior and interior
pub(crate) fn is_contact_point_of_exterior_interior<F: GeoFloat>(
    polygon: &Polygon<F>,
    point: &Point<F>,
) -> bool {
    polygon.exterior().contains(point) && polygon.interiors().iter().any(|int| int.contains(point))
}

/// predicate that checks whether the given point is only traveresd once
pub(crate) fn is_point_traversed_once<F: GeoFloat>(lines: &[Line<F>], point: Point<F>) -> bool {
    lines
        .iter()
        .filter(|l| l.start == point.0 || l.end == point.0)
        .count()
        == 2
}

/// given a set of lines and a start point, returns a predicate that checks if for a given endpoint
/// a new line would intersect one of the given lines
pub(crate) fn exclude_points_forming_invalid_lines<F: GeoFloat>(
    lines: &[Line<F>],
    point_in_hole: Point<F>,
) -> impl Fn(&Point<F>) -> bool + '_ {
    move |p| {
        // filter out len zero lines as they are invalid
        if *p == point_in_hole {
            return true;
        }

        let new_line = Line::new(p.0, point_in_hole.0).scale(F::from(0.99).unwrap());
        !lines.iter().any(|old_line| {
            let maybe_intersection = line_intersection(*old_line, new_line);
            maybe_intersection
                .filter(|kind| match kind {
                    // is_proper ensures they don't intersect in endpoints only
                    LineIntersection::SinglePoint { is_proper, .. } => *is_proper,
                    LineIntersection::Collinear { .. } => true,
                })
                .is_some()
        })
    }
}

/// creates a closure that maps [`exclude_lines_intersecting`] over a range of points and only
/// return the valid ones
pub(crate) fn filter_points_not_creating_intersections<F: GeoFloat>(
    lines: &[Line<F>],
    point_in_hole: Point<F>,
) -> impl Fn(&LineString<F>) -> MultiPoint<F> + '_ {
    move |ls| {
        let points = ls
            .points()
            .filter(exclude_points_forming_invalid_lines(lines, point_in_hole))
            .collect::<Vec<_>>();
        MultiPoint::new(points)
    }
}

/// The polygon splitting algorithm expects it's input polygon to be in a certain shape
///
/// - the exterior needs to be ccw
/// - all interiors need to be cw
///
/// Otherwise connecting two linestrings or splitting one up would fail in unexpected ways. That's
/// why this function is preparing these invariants.
pub(crate) fn prepare_winding<F: GeoFloat>(mut mp: MultiPolygon<F>) -> MultiPolygon<F> {
    mp.iter_mut().for_each(|p| {
        p.exterior_mut(|ls| ls.make_ccw_winding());
        p.interiors_mut(|lss| lss.iter_mut().for_each(|ls| ls.make_cw_winding()));
    });
    mp
}

/// function that always returns true
pub(crate) fn const_true<T>(_: T) -> bool {
    true
}
