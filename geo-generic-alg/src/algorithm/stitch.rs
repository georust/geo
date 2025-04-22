use std::collections::HashMap;

use geo_types::{Coord, Line, LineString, MultiPolygon, Polygon, Triangle};

use crate::winding_order::{triangle_winding_order, WindingOrder};
use crate::{Contains, GeoFloat};

// ========= Error Type ============

#[derive(Debug)]
pub enum LineStitchingError {
    IncompleteRing(&'static str),
}

impl std::fmt::Display for LineStitchingError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{self:?}")
    }
}

impl std::error::Error for LineStitchingError {}

pub(crate) type TriangleStitchingResult<T> = Result<T, LineStitchingError>;

// ========= Main Algo ============

/// Trait to stitch together split up triangles.
pub trait StitchTriangles<T: GeoFloat>: private::Stitchable<T> {
    /// This stitching only happens along identical edges which are located in two separate
    /// geometries. Please read about the required pre conditions of the inputs!
    ///
    /// ```text
    /// ┌─────x        ┌─────┐
    /// │    /│        │     │
    /// │   / │        │     │
    /// │  /  │  ───►  │     │
    /// │ /   │        │     │
    /// │/    │        │     │
    /// x─────┘        └─────┘
    /// ```
    ///
    /// # Pre Conditions
    ///
    /// - The triangles in the input must not overlap! This also forbids identical triangles in the
    ///   input set. If you want to do a union on overlapping triangles then c.f. `SpadeBoolops`.
    /// - Input triangles should be valid polygons. For a definition of validity
    ///   c.f. <https://www.postgis.net/workshops/postgis-intro/validity.html>
    ///
    /// # Examples
    ///
    /// ```text
    /// use geo::StitchTriangles;
    /// use geo::{Coord, Triangle, polygon};
    ///
    /// let tri1 = Triangle::from([
    ///     Coord { x: 0.0, y: 0.0 },
    ///     Coord { x: 1.0, y: 0.0 },
    ///     Coord { x: 0.0, y: 1.0 },
    /// ]);
    /// let tri2 = Triangle::from([
    ///     Coord { x: 1.0, y: 1.0 },
    ///     Coord { x: 1.0, y: 0.0 },
    ///     Coord { x: 0.0, y: 1.0 },
    /// ]);
    ///
    /// let result = vec![tri1, tri2].stitch_triangulation();
    ///
    /// assert!(result.is_ok());
    ///
    /// let mp = result.unwrap();
    ///
    /// assert_eq!(mp.0.len(), 1);
    ///
    /// let poly = mp.0[0].clone();
    /// // 4 coords + 1 duplicate for closed-ness
    /// assert_eq!(poly.exterior().0.len(), 4 + 1);
    ///
    /// let expected = polygon![
    ///     Coord { x: 1.0, y: 1.0 },
    ///     Coord { x: 0.0, y: 1.0 },
    ///     Coord { x: 0.0, y: 0.0 },
    ///     Coord { x: 1.0, y: 0.0 },
    /// ];
    ///
    /// assert_eq!(poly, expected);
    /// ```
    ///
    /// # Additional Notes
    ///
    /// Stitching triangles which result in a polygon with a hole which touches the outline
    /// (mentioned here [banana polygon](https://postgis.net/workshops/postgis-intro/validity.html#repairing-invalidity))
    /// will result in a single polygon without interiors instead of a polygon with a single
    /// interior
    ///
    /// ```text
    /// ┌────────x────────┐
    /// │\....../ \....../│
    /// │.\..../   \..../.│
    /// │..\../     \../..│
    /// │...\/       \/...│
    /// │...───────────...│
    /// │../\....^..../\..│
    /// │./..\../.\../..\.│
    /// │/....\/...\/....\│
    /// └─────────────────┘
    ///
    ///     │    │    │
    ///     ▼    ▼    ▼
    ///
    /// ┌────────x────────┐
    /// │       / \       │
    /// │      /   \      │
    /// │     /     \     │
    /// │    /       \    │
    /// │   ───────────   │
    /// │                 │
    /// │                 │
    /// │                 │
    /// └─────────────────┘
    /// ```
    ///
    /// ---
    ///
    /// If you want to do something more general like a
    /// [`Boolean Operation Union`](https://en.wikipedia.org/wiki/Boolean_operations_on_polygons)
    /// you should use the trait `BooleanOps` or `SpadeBoolops`.
    fn stitch_triangulation(&self) -> TriangleStitchingResult<MultiPolygon<T>>;
}

mod private {
    use super::*;

    pub trait Stitchable<T: GeoFloat>: AsRef<[Triangle<T>]> {}
    impl<S, T> Stitchable<T> for S
    where
        S: AsRef<[Triangle<T>]>,
        T: GeoFloat,
    {
    }
}

impl<S, T> StitchTriangles<T> for S
where
    S: private::Stitchable<T>,
    T: GeoFloat,
{
    fn stitch_triangulation(&self) -> TriangleStitchingResult<MultiPolygon<T>> {
        stitch_triangles(self.as_ref().iter())
    }
}

// main stitching algorithm
fn stitch_triangles<'a, T, S>(triangles: S) -> TriangleStitchingResult<MultiPolygon<T>>
where
    T: GeoFloat + 'a,
    S: Iterator<Item = &'a Triangle<T>>,
{
    let lines = triangles.flat_map(ccw_lines).collect::<Vec<_>>();

    let boundary_lines = find_boundary_lines(lines);
    let stitched_multipolygon = stitch_multipolygon_from_lines(boundary_lines)?;

    let polys = stitched_multipolygon
        .into_iter()
        .map(find_and_fix_holes_in_exterior)
        .collect::<Vec<_>>();

    Ok(MultiPolygon::new(polys))
}

/// returns the triangle's lines with ccw orientation
fn ccw_lines<T: GeoFloat>(tri: &Triangle<T>) -> [Line<T>; 3] {
    match triangle_winding_order(tri) {
        Some(WindingOrder::CounterClockwise) => tri.to_lines(),
        _ => {
            let [a, b, c] = tri.to_array();
            [(b, a), (a, c), (c, b)].map(|(start, end)| Line::new(start, end))
        }
    }
}

/// checks whether the two lines are equal or inverted forms of each other
#[inline]
fn same_line<T: GeoFloat>(l1: &Line<T>, l2: &Line<T>) -> bool {
    (l1.start == l2.start && l1.end == l2.end) || (l1.start == l2.end && l2.start == l1.end)
}

/// given a collection of lines from multiple polygons which partition an area we can have two
/// kinds of lines:
///
/// - boundary lines: these are the unique lines on the boundary of the compound shape which is
///   formed by the collection of polygons
/// - inner lines: these are all non-boundary lines. They are not unique and have exactly one
///   duplicate on one adjacent polygon in the collection (as long as the input is valid!)
fn find_boundary_lines<T: GeoFloat>(lines: Vec<Line<T>>) -> Vec<Line<T>> {
    lines.into_iter().fold(Vec::new(), |mut lines, new_line| {
        if let Some(idx) = lines.iter().position(|line| same_line(line, &new_line)) {
            lines.remove(idx);
        } else {
            lines.push(new_line);
        }
        lines
    })
}

// Notes for future: This probably belongs into a `Validify` trait or something
/// finds holes in polygon exterior and fixes them
///
/// This is important for scenarios like the banana polygon. Which is considered invalid
/// https://www.postgis.net/workshops/postgis-intro/validity.html#repairing-invalidity
fn find_and_fix_holes_in_exterior<F: GeoFloat>(mut poly: Polygon<F>) -> Polygon<F> {
    fn detect_if_rings_closed_with_point<F: GeoFloat>(
        points: &mut Vec<Coord<F>>,
        p: Coord<F>,
    ) -> Option<Vec<Coord<F>>> {
        // early return here if nothing was found
        let pos = points.iter().position(|&c| c == p)?;

        // create ring by collecting the points if something was found
        let ring = points
            .drain(pos..)
            .chain(std::iter::once(p))
            .collect::<Vec<_>>();
        Some(ring)
    }

    // find rings
    let rings = {
        let (points, mut rings) =
            poly.exterior()
                .into_iter()
                .fold((vec![], vec![]), |(mut points, mut rings), coord| {
                    rings.extend(detect_if_rings_closed_with_point(&mut points, *coord));
                    points.push(*coord);
                    (points, rings)
                });

        // add leftover coords as last ring
        rings.push(points);

        rings
    };

    // convert to polygons for containment checks
    let mut rings = rings
        .into_iter()
        // filter out degenerate polygons which may be produced from the code above
        .filter(|cs| cs.len() >= 3)
        .map(|cs| Polygon::new(LineString::new(cs), vec![]))
        .collect::<Vec<_>>();

    // PERF: O(n^2) maybe someone can reduce this. Please benchmark!
    fn find_outmost_ring<F: GeoFloat>(rings: &[Polygon<F>]) -> Option<usize> {
        let enumerated_rings = || rings.iter().enumerate();
        enumerated_rings()
            .find(|(i, ring)| {
                enumerated_rings()
                    .filter(|(j, _)| i != j)
                    .all(|(_, other)| ring.contains(other))
            })
            .map(|(i, _)| i)
    }

    // if exterior ring exists that contains all other rings, recreate the poly with:
    //
    // - exterior ring as exterior
    // - other rings are counted to interiors
    // - previously existing interiors are preserved
    if let Some(outer_index) = find_outmost_ring(&rings) {
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
fn stitch_multipolygon_from_lines<F: GeoFloat>(
    lines: Vec<Line<F>>,
) -> TriangleStitchingResult<MultiPolygon<F>> {
    let rings = stitch_rings_from_lines(lines)?;

    fn find_parent_idxs<F: GeoFloat>(
        ring_idx: usize,
        ring: &LineString<F>,
        all_rings: &[LineString<F>],
    ) -> Vec<usize> {
        all_rings
            .iter()
            .enumerate()
            .filter(|(other_idx, _)| ring_idx != *other_idx)
            .filter_map(|(idx, maybe_parent)| {
                Polygon::new(maybe_parent.clone(), vec![])
                    .contains(ring)
                    .then_some(idx)
            })
            .collect()
    }

    // Associates every ring with its parents (the rings that contain it)
    let parents_of: HashMap<usize, Vec<usize>> = rings
        .iter()
        .enumerate()
        .map(|(ring_idx, ring)| {
            let parent_idxs = find_parent_idxs(ring_idx, ring, &rings);
            (ring_idx, parent_idxs)
        })
        .collect();

    // Associates outer rings with their inner rings
    let mut polygons_idxs: HashMap<usize, Vec<usize>> = HashMap::default();

    // the direct parent is the parent ring which has itself the most parent rings
    fn find_direct_parent(
        parent_rings: &[usize],
        parents_of: &HashMap<usize, Vec<usize>>,
    ) -> Option<usize> {
        parent_rings
            .iter()
            .filter_map(|ring_idx| {
                parents_of
                    .get(ring_idx)
                    .map(|grandparent_rings| (ring_idx, grandparent_rings))
            })
            .max_by_key(|(_, grandparent_rings)| grandparent_rings.len())
            .map(|(idx, _)| idx)
            .copied()
    }

    // For each ring, we check how many parents it has  otherwise it's an outer ring
    //
    // This is important in the scenarios of "donuts" where you have an outer donut shaped
    // polygon which completely contains a smaller polygon inside its hole
    for (ring_index, parent_idxs) in parents_of.iter() {
        let parent_count = parent_idxs.len();

        // if it has an even number of parents, it's an outer ring so we can just add it if it's
        // missing
        if parent_count % 2 == 0 {
            polygons_idxs.entry(*ring_index).or_default();
            continue;
        }

        // if it has an odd number of parents, it's an inner ring

        // to find the specific outer ring it is related to, we search for the direct parent.
        let maybe_direct_parent = find_direct_parent(parent_idxs, &parents_of);

        // As stated above the amount of parents here is odd, so it's at least one.
        // Since every ring is registered in the `parents` hashmap, we find at least one element
        // while iterating. Hence the `max_by_key` will always return `Some` since the iterator
        // is never empty
        debug_assert!(
            maybe_direct_parent.is_some(),
            "A direct parent has to exist"
        );

        // I'm not unwrapping here since I'm scared of panics
        if let Some(direct_parent) = maybe_direct_parent {
            polygons_idxs
                .entry(direct_parent)
                .or_default()
                .push(*ring_index);
        }
    }

    // lookup rings by index and create polygons
    let polygons = polygons_idxs
        .into_iter()
        .map(|(parent_idx, children_idxs)| {
            // PERF: extensive cloning here, maybe someone can improve this. Please benchmark!
            let exterior = rings[parent_idx].clone();
            let interiors = children_idxs
                .into_iter()
                .map(|child_idx| rings[child_idx].clone())
                .collect::<Vec<_>>();
            (exterior, interiors)
        })
        .map(|(exterior, interiors)| Polygon::new(exterior, interiors));

    Ok(polygons.collect())
}

// ============== Helpers ================

fn stitch_rings_from_lines<F: GeoFloat>(
    lines: Vec<Line<F>>,
) -> TriangleStitchingResult<Vec<LineString<F>>> {
    // initial ring parts are just lines which will be stitch together progressively
    let mut ring_parts: Vec<Vec<Coord<F>>> = lines
        .iter()
        .map(|line| vec![line.start, line.end])
        .collect();

    let mut rings: Vec<LineString<F>> = vec![];
    // terminates since every loop we'll merge two elements into one so the total number of
    // elements decreases each loop by at least one (two in the case of completing a ring)
    while let Some(last_part) = ring_parts.pop() {
        let (j, compound_part) = ring_parts
            .iter()
            .enumerate()
            .find_map(|(j, other_part)| {
                let new_part = try_stitch(&last_part, other_part)?;
                Some((j, new_part))
            })
            .ok_or(LineStitchingError::IncompleteRing("Couldn't reconstruct polygons from the inputs. Please check them for invalidities."))?;
        ring_parts.remove(j);

        let is_ring = compound_part.first() == compound_part.last() && !compound_part.is_empty();

        if is_ring {
            let new_ring = LineString::new(compound_part);
            rings.push(new_ring);
        } else {
            ring_parts.push(compound_part);
        }
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

    // _ -> X  |  X -> _
    (a_last == b_first)
        .then(|| a().chain(b().skip(1)).cloned().collect())
        // X -> _  |  _ -> X
        .or_else(|| (a_first == b_last).then(|| b().chain(a().skip(1)).cloned().collect()))
}

// ============= Tests ===========

#[cfg(test)]
mod polygon_stitching_tests {

    use crate::{Relate, TriangulateEarcut, Winding};

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

        let tris = [inner, outer].map(|p| p.earcut_triangles()).concat();

        let result = tris.stitch_triangulation().unwrap();

        assert!(mp.relate(&result).is_equal_topo());
    }

    #[test]
    fn stitch_independent_of_orientation() {
        _ = pretty_env_logger::try_init();
        let mut tri1 = Triangle::from([
            Coord { x: 0.0, y: 0.0 },
            Coord { x: 1.0, y: 0.0 },
            Coord { x: 0.0, y: 1.0 },
        ])
        .to_polygon();
        let mut tri2 = Triangle::from([
            Coord { x: 1.0, y: 1.0 },
            Coord { x: 1.0, y: 0.0 },
            Coord { x: 0.0, y: 1.0 },
        ])
        .to_polygon();

        tri1.exterior_mut(|ls| ls.make_ccw_winding());
        tri2.exterior_mut(|ls| ls.make_ccw_winding());
        let result_1 = [tri1.clone(), tri2.clone()]
            .map(|tri| tri.earcut_triangles())
            .concat()
            .stitch_triangulation()
            .unwrap();

        tri1.exterior_mut(|ls| ls.make_cw_winding());
        tri2.exterior_mut(|ls| ls.make_ccw_winding());
        let result_2 = [tri1.clone(), tri2.clone()]
            .map(|tri| tri.earcut_triangles())
            .concat()
            .stitch_triangulation()
            .unwrap();

        tri1.exterior_mut(|ls| ls.make_cw_winding());
        tri2.exterior_mut(|ls| ls.make_cw_winding());
        let result_3 = [tri1.clone(), tri2.clone()]
            .map(|tri| tri.earcut_triangles())
            .concat()
            .stitch_triangulation()
            .unwrap();

        tri1.exterior_mut(|ls| ls.make_ccw_winding());
        tri2.exterior_mut(|ls| ls.make_cw_winding());
        let result_4 = [tri1, tri2]
            .map(|tri| tri.earcut_triangles())
            .concat()
            .stitch_triangulation()
            .unwrap();

        assert!(result_1.relate(&result_2).is_equal_topo());
        assert!(result_2.relate(&result_3).is_equal_topo());
        assert!(result_3.relate(&result_4).is_equal_topo());
    }

    #[test]
    fn stitch_creating_hole() {
        let poly1 = Polygon::new(
            LineString::new(vec![
                Coord { x: 0.0, y: 0.0 },
                Coord { x: 1.0, y: 0.0 },
                Coord { x: 1.0, y: 1.0 },
                Coord { x: 1.0, y: 2.0 },
                Coord { x: 2.0, y: 2.0 },
                Coord { x: 2.0, y: 1.0 },
                Coord { x: 2.0, y: 0.0 },
                Coord { x: 3.0, y: 0.0 },
                Coord { x: 3.0, y: 3.0 },
                Coord { x: 0.0, y: 3.0 },
            ]),
            vec![],
        );
        let poly2 = Polygon::new(
            LineString::new(vec![
                Coord { x: 1.0, y: 0.0 },
                Coord { x: 2.0, y: 0.0 },
                Coord { x: 2.0, y: 1.0 },
                Coord { x: 1.0, y: 1.0 },
            ]),
            vec![],
        );

        let result = [poly1, poly2]
            .map(|p| p.earcut_triangles())
            .concat()
            .stitch_triangulation()
            .unwrap();

        assert_eq!(result.0.len(), 1);
        assert_eq!(result.0[0].interiors().len(), 1);
    }

    #[test]
    fn inner_banana_produces_hole() {
        let poly = Polygon::new(
            LineString::new(vec![
                Coord { x: 0.0, y: 0.0 },
                Coord { x: 4.0, y: 0.0 },
                Coord { x: 3.0, y: 2.0 },
                Coord { x: 5.0, y: 2.0 },
                Coord { x: 4.0, y: 0.0 },
                Coord { x: 8.0, y: 0.0 },
                Coord { x: 4.0, y: 4.0 },
            ]),
            vec![],
        );

        let result = [poly]
            .map(|p| p.earcut_triangles())
            .concat()
            .stitch_triangulation()
            .unwrap();

        assert_eq!(result.0.len(), 1);
        assert_eq!(result.0[0].interiors().len(), 1);
    }

    #[test]
    fn outer_banana_doesnt_produce_hole() {
        let poly = Polygon::new(
            LineString::new(vec![
                Coord { x: 0.0, y: 0.0 },
                Coord { x: 4.0, y: 0.0 },
                Coord { x: 3.0, y: -2.0 },
                Coord { x: 5.0, y: -2.0 },
                Coord { x: 4.0, y: 0.0 },
                Coord { x: 8.0, y: 0.0 },
                Coord { x: 4.0, y: 4.0 },
            ]),
            vec![],
        );

        let result = [poly]
            .map(|p| p.earcut_triangles())
            .concat()
            .stitch_triangulation()
            .unwrap();

        assert_eq!(result.0.len(), 2);
        assert_eq!(result.0[0].interiors().len(), 0);
    }
}
