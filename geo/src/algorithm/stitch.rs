use std::collections::HashMap;

use geo_types::{Coord, Line, LineString, MultiPolygon, Polygon};

use crate::{Contains, CoordsIter, GeoFloat, LinesIter};

// ========= Error Type ============

#[derive(Debug)]
pub enum LineStitchingError {
    IncompleteRing,
    InvalidGeometry,
    MissingParent,
    NoExtremum,
}

impl std::fmt::Display for LineStitchingError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{self:?}")
    }
}

impl std::error::Error for LineStitchingError {}

pub(crate) type PolygonStitchingResult<T> = Result<T, LineStitchingError>;

// ========= Main Algo ============

pub trait Stitch<'a, T: GeoFloat> {
    fn stitch_together(&'a self) -> PolygonStitchingResult<MultiPolygon<T>>;
}

impl<'a, 'b, T, G> Stitch<'a, T> for Vec<G>
where
    'b: 'a,
    T: GeoFloat,
    G: LinesIter<'a, Scalar = T>,
{
    fn stitch_together(&'a self) -> PolygonStitchingResult<MultiPolygon<T>> {
        let lines = self
            .iter()
            .flat_map(|part| part.lines_iter())
            .collect::<Vec<_>>();

        let boundary_lines = find_boundary_lines(lines);
        let stitched_multipolygon = stitch_multipolygon_from_lines(boundary_lines)?;

        let polys = stitched_multipolygon
            .into_iter()
            .map(find_and_fix_holes_in_exterior)
            .collect::<Vec<_>>();

        Ok(MultiPolygon::new(polys))
    }
}

impl<'a, 'b, T, G> Stitch<'a, T> for &[G]
where
    'b: 'a,
    T: GeoFloat,
    G: LinesIter<'a, Scalar = T>,
{
    fn stitch_together(&'a self) -> PolygonStitchingResult<MultiPolygon<T>> {
        let lines = self
            .iter()
            .flat_map(|part| part.lines_iter())
            .collect::<Vec<_>>();

        let boundary_lines = find_boundary_lines(lines);
        let stitched_multipolygon = stitch_multipolygon_from_lines(boundary_lines)?;

        let polys = stitched_multipolygon
            .into_iter()
            .map(find_and_fix_holes_in_exterior)
            .collect::<Vec<_>>();

        Ok(MultiPolygon::new(polys))
    }
}

/// checks whether the to lines are equal or inverted forms of each other
fn same_line<T: GeoFloat>(l1: &Line<T>, l2: &Line<T>) -> bool {
    l1 == l2 || l1 == &Line::new(l2.end, l2.start)
}

/// given a collection of lines from multiple polygons, this returns all but the shared lines
fn find_boundary_lines<T: GeoFloat>(lines: Vec<Line<T>>) -> Vec<Line<T>> {
    lines
        .iter()
        .enumerate()
        // only collect lines that don't have a duplicate in the set
        .filter_map(|(i, line)| {
            (!lines
                .iter()
                .enumerate()
                .filter(|&(j, _)| j != i)
                .any(|(_, l)| same_line(line, l)))
            .then_some(*line)
        })
        .collect::<Vec<_>>()
}

/// Inputs to this function is a unordered set of polygons have been split from a multipolygon
/// This function will return an invalid multipolygon if the provided parts were not a result of splitting
pub fn stitch_multipolygon_from_parts<T: GeoFloat>(
    parts: &[Polygon<T>],
) -> PolygonStitchingResult<MultiPolygon<T>> {
    let lines = parts
        .iter()
        .flat_map(|part| part.lines_iter())
        .collect::<Vec<Line<T>>>();

    let boundary_lines = find_boundary_lines(lines);
    stitch_multipolygon_from_lines(boundary_lines).map(|mut mp| {
        mp.iter_mut().for_each(|poly| {
            *poly = find_and_fix_holes_in_exterior(poly.clone());
        });
        mp
    })
}

/// finds holes in polygon exterior and fixes them
///
/// This is important for scenarios like the banana polygon. Which is considered invalid
/// https://www.postgis.net/workshops/postgis-intro/validity.html#repairing-invalidity
pub fn find_and_fix_holes_in_exterior<F: GeoFloat>(mut poly: Polygon<F>) -> Polygon<F> {
    // find rings
    let mut rings = vec![];
    let mut points = vec![];
    for p in poly.exterior() {
        if let Some(i) = points.iter().position(|&c| c == p) {
            rings.push(
                points
                    .drain(i..)
                    .chain(std::iter::once(p))
                    .cloned()
                    .collect::<Vec<_>>(),
            );
        }
        points.push(p);
    }
    rings.push(points.into_iter().cloned().collect::<Vec<_>>());

    let mut rings = rings
        .into_iter()
        .map(|cs| Polygon::new(LineString::new(cs), vec![]))
        // filter out degenerate polygons which may be produced from the code above
        .filter(|p| p.coords_count() >= 3)
        .collect::<Vec<_>>();

    // if exterior ring exists that contains all other rings, recreate the poly with:
    //
    // - exterior ring as exterior
    // - other rings are counted to interiors
    // - previously existing interiors are preserved
    if let Some(outer_index) = rings
        .iter()
        .enumerate()
        .find(|(i, ring)| {
            rings
                .iter()
                .enumerate()
                .filter(|(j, _)| i != j)
                .all(|(_, other)| ring.contains(other))
        })
        .map(|(i, _)| i)
    {
        let exterior = rings.remove(outer_index).exterior().clone();
        let interiors = poly
            .interiors()
            .iter()
            .cloned()
            .chain(rings.into_iter().map(|p| p.exterior().clone()))
            .collect::<Vec<_>>();
        poly = Polygon::new(exterior, interiors);
    }
    poly
}

/// Inputs to this function is a unordered set of lines that must form a valid multipolygon
pub fn stitch_multipolygon_from_lines<F: GeoFloat>(
    lines: Vec<Line<F>>,
) -> PolygonStitchingResult<MultiPolygon<F>> {
    let rings = stitch_rings_from_lines(lines)?;

    // Associates every ring with its parents (the rings that contain it)
    let mut parents: HashMap<usize, Vec<usize>> = HashMap::default();

    for (current_ring_idx, ring) in rings.iter().enumerate() {
        let parent_idxs = rings
            .iter()
            .enumerate()
            .filter(|(other_idx, _)| current_ring_idx != *other_idx)
            .filter_map(|(idx, maybe_parent)| {
                Polygon::new(maybe_parent.clone(), vec![])
                    .contains(ring)
                    .then_some(idx)
            })
            .collect::<Vec<_>>();
        parents.insert(current_ring_idx, parent_idxs);
    }

    // Associates outer rings with their inner rings
    let mut polygons_idxs: HashMap<usize, Vec<usize>> = HashMap::default();

    // For each ring, we check how many parents it has  otherwise it's an outer ring
    //
    // This is important in the scenarios of "donuts" where you have an outer donut shaped
    // polygon which completely contains a smaller polygon inside its hole
    for (ring_index, parent_idxs) in parents.iter() {
        let parent_count = parent_idxs.len();

        // if it has an even number of parents, it's an outer ring so we can just add it if it's
        // missing
        if parent_count % 2 == 0 {
            polygons_idxs.entry(*ring_index).or_default();
            continue;
        }

        // if it has an odd number of parents, it's an inner ring
        //
        // to find the specific outer ring it is related to, we search for the direct parent. The
        // direct parent of the current ring has itself the most parents from all available
        // parents, so find it by max
        if let Some(direct_parent) = parent_idxs
            .iter()
            .filter_map(|parent_idx| {
                parents
                    .get(parent_idx)
                    .map(|grandparents| (parent_idx, grandparents))
            })
            .max_by_key(|(_, grandparents)| grandparents.len())
            .map(|(idx, _)| idx)
        {
            polygons_idxs
                .entry(*direct_parent)
                .or_default()
                .push(*ring_index);
        }
    }

    // lookup rings by index and create polygons
    let polygons = polygons_idxs
        .into_iter()
        .map(|(parent, children)| {
            (
                rings[parent].clone(),
                children
                    .into_iter()
                    .map(|child| rings[child].clone())
                    .collect::<Vec<_>>(),
            )
        })
        .map(|(exterior, interiors)| Polygon::new(exterior, interiors));

    Ok(polygons.collect())
}

// ============== Helpers ================

pub fn stitch_rings_from_lines<F: GeoFloat>(
    lines: Vec<Line<F>>,
) -> PolygonStitchingResult<Vec<LineString<F>>> {
    let mut ring_parts: Vec<Vec<Coord<F>>> = lines
        .iter()
        .map(|line| vec![line.start, line.end])
        .collect();

    let mut rings: Vec<LineString<F>> = vec![];
    while !ring_parts.is_empty() {
        let (j, res) = ring_parts
            .iter()
            .enumerate()
            .skip(1)
            .find_map(|(j, part)| try_stitch(&ring_parts[0], part).map(|res| (j, res)))
            .ok_or(LineStitchingError::IncompleteRing)?;

        if res.first() == res.last() && !res.is_empty() {
            rings.push(LineString::new(res));
        } else {
            ring_parts.push(res);
        }

        ring_parts.remove(j);
        ring_parts.remove(0);
    }

    Ok(rings)
}

fn try_stitch<F: GeoFloat>(a: &[Coord<F>], b: &[Coord<F>]) -> Option<Vec<Coord<F>>> {
    let a_first = a.first()?;
    let a_last = a.last()?;
    let b_first = b.first()?;
    let b_last = b.last()?;

    let a = || a.iter();
    let b = || b.iter();

    // X -> _  |  X -> _
    (a_first == b_first)
        .then(|| a().rev().chain(b().skip(1)).cloned().collect())
        // X -> _  |  _ -> X
        .or_else(|| (a_first == b_last).then(|| b().chain(a().skip(1)).cloned().collect()))
        // _ -> X  |  X -> _
        .or_else(|| (a_last == b_first).then(|| a().chain(b().skip(1)).cloned().collect()))
        // _ -> X  |  _ -> X
        .or_else(|| (a_last == b_last).then(|| a().chain(b().rev().skip(1)).cloned().collect()))
}

// ============= Tests ===========

#[cfg(test)]
mod polygon_stitching_tests {

    use crate::TriangulateEarcut;

    use super::*;
    use geo_types::*;

    #[test]
    fn poly_inside_a_donut() {
        _ = pretty_env_logger::try_init();
        let zero = Coord::zero();
        let one = Point::new(1.0, 1.0).0;
        let outer_outer = Rect::new(zero, one * 5.0);
        let inner_outer = Rect::new(one, one * 4.0);
        let outer = Polygon::new(
            outer_outer.to_polygon().exterior().clone(),
            vec![inner_outer.to_polygon().exterior().clone()],
        );
        let inner = Rect::new(one * 2.0, one * 3.0).to_polygon();

        let mp = MultiPolygon::new(vec![outer.clone(), inner.clone()]);

        let tris = [inner, outer]
            .map(|p| p.earcut_triangles())
            .map(|tris| {
                tris.into_iter()
                    .map(|tri| tri.to_polygon())
                    .collect::<Vec<_>>()
            })
            .concat();

        let result = stitch_multipolygon_from_parts(&tris).unwrap();

        assert!(mp.contains(&result) && result.contains(&mp));
    }
}
