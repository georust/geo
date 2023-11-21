mod banana_utils;
mod types;
mod utils;

use geo_types::*;

use banana_utils::{find_and_split_outer_banana, find_closest_lines_for_banana, is_banana_polygon};
use types::{
    fold_closest, fold_closest_precise, ClosestPointInfo, ClosestPointPreciseInfo, ConnectionKind,
    PolyAndClosest, Retry,
};
use utils::{
    create_linestring_between_points, filter_points_not_creating_intersections,
    find_and_remove_polygon, is_contact_point_of_exterior_interior, is_point_traversed_once,
    iter_between, prepare_winding, reassociate_holes, remove_nth_hole, remove_two_holes,
};

use crate::{ClosestPoint, GeoFloat, LinesIter};

/// find's two connections between the first hole of a polygon and the rest of the polygon in the
/// following ordering:
///
/// - Let H: be the hole-linestring of the first hole of the poly
/// - Let R: be the rest of the polygon (exterior + every but the first hole)
///
/// 1. find a point in `H` with the minimum distance to any point `R` and determine the connection to
///    this other point
/// 2. find a point in `H` with the maximum distance to the first point in the hole-linestring found
///    in 1.
/// 3. find the minimum distance connection between the point from 2. and any other point in `R`
fn find_closest_lines_for_first_hole<F: GeoFloat>(
    polygon: &Polygon<F>,
) -> Option<ClosestPointPreciseInfo<F>> {
    // Define some helping closures
    let lines = polygon
        .lines_iter()
        .filter(|l| l.start != l.end)
        .collect::<Vec<_>>();
    let first_hole = polygon.interiors().first().cloned()?;
    let iter_targets = || {
        std::iter::once(polygon.exterior())
            .chain(polygon.interiors().iter().skip(1))
            .enumerate()
    };
    // exclusion set is to prevent the algo from choosing the same point twice since that would
    // potentially to banana polygons which are not considered valid
    let find_closest_for = |point_in_self: Point<F>| {
        iter_targets()
            .map(|(id, linestring)| {
                let ps =
                    filter_points_not_creating_intersections(&lines, point_in_self)(linestring);
                (id, ps)
            })
            .map(|(id, target_points)| (id, target_points.closest_point(&point_in_self)))
            .map(|(id, point_in_other)| ClosestPointInfo {
                point_in_other,
                point_in_self,
                from_linestring: ConnectionKind::from_normal_index(id),
            })
            .fold(None, fold_closest)
    };

    let valid_point_filter = |p: &Point<F>| {
        // this is to give the algo a chance to just cut the poly at the contact point
        let is_contact = is_contact_point_of_exterior_interior(polygon, p);
        // these are all other points that guarantee that the connection isn't made between another
        // connection part
        let is_traversed_once = is_point_traversed_once(&lines, *p);
        is_contact || is_traversed_once
    };

    let closest_connection = first_hole
        .points()
        .filter(valid_point_filter)
        .filter_map(find_closest_for)
        .fold(None, fold_closest)
        .and_then(ClosestPointPreciseInfo::from_unprecise)?;

    Some(closest_connection)
}

/// based on the location of the two closest points found, dispatch to functions that use these
/// points to take one step into splitting up the polygon
fn handle_closest_points_found<F: GeoFloat>(args: PolyAndClosest<F>) -> MultiPolygon<F> {
    handle_closest_points_found_normal_cases(args)
        .unwrap_or_else(handle_closest_points_found_banana_cases)
}

fn handle_closest_points_found_normal_cases<F: GeoFloat>(args: PolyAndClosest<F>) -> Retry<F> {
    // dispatch to specialized functions
    match args.closest.from_linestring {
        // we're splitting up the polygon into two new polygons
        ConnectionKind::Exterior => connect_hole_to_ext(args),
        // in any other case it's enough to make progress by merging one hole
        ConnectionKind::Interior(_) => merge_holes(args),
    }
}

fn handle_closest_points_found_banana_cases<F: GeoFloat>(
    args: PolyAndClosest<F>,
) -> MultiPolygon<F> {
    // dispatch to specialized functions
    match args.closest.from_linestring {
        // we're splitting up the polygon into two new polygons
        ConnectionKind::Exterior => make_banana_split(args),
        // in any other case it's enough to make progress by merging one hole
        ConnectionKind::Interior(_) => connect_hole_to_banana(args),
    }
}

/// hmm, yummy 🍨🍌🍨
fn make_banana_split<F: GeoFloat>(
    PolyAndClosest { poly, closest }: PolyAndClosest<F>,
) -> MultiPolygon<F> {
    let ext = poly.exterior().clone();

    let ext_iter = iter_between(ext);

    let create_poly_with_idxs = |[a, b]: [Point<F>; 2]| {
        let mut ext_coords = ext_iter([a, b]);
        ext_coords.dedup();
        Polygon::new(LineString::new(ext_coords), vec![])
    };

    let p1 = create_poly_with_idxs([closest.point_in_self, closest.point_in_other]);
    let p2 = create_poly_with_idxs([closest.point_in_other, closest.point_in_self]);

    let new_polys = reassociate_holes([p1, p2], poly.interiors().iter().cloned());

    MultiPolygon::new(new_polys.to_vec())
}

fn connect_hole_to_banana<F: GeoFloat>(
    PolyAndClosest { poly, closest }: PolyAndClosest<F>,
) -> MultiPolygon<F> {
    // get the index of the second hole (which isn't the first hole)
    let (c, index) = match closest.from_linestring {
        ConnectionKind::Interior(index) => (closest, index),
        _ => unreachable!("got ext connection when hole was expected"),
    };

    let ext = poly.exterior().clone();

    let (poly_without_first_hole, hole) =
        remove_nth_hole(&poly, index).expect("this hole exists since it produced the index");

    let ext_iter = iter_between(ext);
    let hole_iter = iter_between(hole);

    let create_poly_with_idxs = |[a, b]: [Point<F>; 2]| {
        let ext_part_1 = ext_iter([a, a]);
        let ext_part_2 = hole_iter([b, b]);
        let mut ext_coords = [ext_part_1.clone(), ext_part_2.clone()].concat();
        ext_coords.dedup();

        Polygon::new(LineString::new(ext_coords), vec![])
    };

    //  ┌──────────x───────────┐
    //  │          │           │
    //  │      ┌───x───┐       │
    //  │      │       │       │
    //  │      │       │       │
    //  │      └───────┘       │
    //  │                      │
    //  └──────────────────────┘

    let mut result_poly = create_poly_with_idxs([c.point_in_self, c.point_in_other]);

    for hole in poly_without_first_hole.interiors().iter().cloned() {
        result_poly.interiors_push(hole)
    }

    MultiPolygon::new(vec![result_poly])
}

/// connects a hole with two lines to the exterior. In this process, it's guaranteed that we split
/// up the polygon into two new polygons. The rest of the holes will be re-associated with the two
/// new polygons by containment tests
fn connect_hole_to_ext<F: GeoFloat>(
    PolyAndClosest { poly, closest }: PolyAndClosest<F>,
) -> Retry<F> {
    // winding is important here. It doesn't have to be exactly like this but the exterior and the
    // hole have to have opposite winding orders for this to work
    //
    // If you get why take a look at the sketch further below and play the scenario through.

    let ext = poly.exterior().clone();

    let (poly_without_first_hole, hole) =
        remove_nth_hole(&poly, 0).ok_or(PolyAndClosest { poly, closest })?;

    let ext_iter = iter_between(ext);
    let hole_iter = iter_between(hole);

    //  ┌──────────x───────────┐
    //  │          │           │
    //  │      ┌───x───┐       │
    //  │      │       │       │
    //  │      │       │       │
    //  │      └───────┘       │
    //  │                      │
    //  └──────────────────────┘

    let result_poly_exterior = create_linestring_between_points(ext_iter, hole_iter)([
        closest.point_in_other,
        closest.point_in_self,
    ]);
    let mut result_poly = Polygon::new(result_poly_exterior, vec![]);

    for hole in poly_without_first_hole.interiors().iter().cloned() {
        result_poly.interiors_push(hole)
    }

    Ok(MultiPolygon::new(vec![result_poly]))
}

/// This merges two holes by simply connecting them via a line
fn merge_holes<F: GeoFloat>(PolyAndClosest { poly, closest }: PolyAndClosest<F>) -> Retry<F> {
    // get the index of the other hole (which isn't the first hole)
    let (c, index) = match closest.from_linestring {
        ConnectionKind::Interior(index) => (closest, index),
        _ => return Err(PolyAndClosest { poly, closest }),
    };

    let (mut p, other_hole, first_hole) =
        remove_two_holes(&poly, [index, 0]).ok_or(PolyAndClosest { poly, closest })?;

    let first_iter = iter_between(first_hole);
    let other_iter = iter_between(other_hole);

    let hole = create_linestring_between_points(first_iter, other_iter)([
        c.point_in_self,
        c.point_in_other,
    ]);

    p.interiors_push(hole);

    Ok(MultiPolygon::new(vec![p]))
}

/// splits a MultiPolygon with holes into MultiPolygon without holes.
pub fn split_invalid_multipolygon<F: GeoFloat>(mp: &MultiPolygon<F>) -> MultiPolygon<F> {
    let mut mp = prepare_winding(mp.clone());
    // take one step, this ends in finite time since it'll eliminate exactly one hole per loop
    // iteration. This means we'll run this exactly `num_holes` times
    while let Some(p) = find_and_remove_polygon(&mut mp, |p| {
        !p.interiors().is_empty() || is_banana_polygon(p)
    }) {
        if let Some(new_mp) = find_and_split_outer_banana(&p) {
            mp.0.extend(new_mp);
            continue;
        }

        let closest_connection = find_closest_lines_for_banana(&p)
            .into_iter()
            .chain(find_closest_lines_for_first_hole(&p))
            .fold(None, fold_closest_precise);
        let new_mp = if let Some(closest) = closest_connection {
            handle_closest_points_found(PolyAndClosest {
                poly: p.clone(),
                closest,
            })
        } else {
            match remove_nth_hole(&p, 0) {
                Some((p, _)) => MultiPolygon::new(vec![p]),
                None => {
                    log::error!("Didn't find a hole although one should exist. Returning mid algo");
                    return mp;
                }
            }
        };

        mp.0.extend(new_mp);
    }
    mp
}
