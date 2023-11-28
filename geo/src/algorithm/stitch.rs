use std::collections::HashMap;

use geo_types::{Coord, Line, LineString, MultiPolygon, Polygon, Triangle};

use crate::{Contains, GeoFloat, LinesIter, Winding};

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

/// Trait to stitch together split up polygons.
pub trait Stitch<T: GeoFloat> {
    /// This stitching only happens along identical edges which are located in two separate
    /// geometries.
    ///
    /// ┌─────x        ┌─────┐
    /// │    /│        │     │
    /// │   / │        │     │
    /// │  /  │  ───►  │     │
    /// │ /   │        │     │
    /// │/    │        │     │
    /// x─────┘        └─────┘
    ///
    /// # Examples
    ///
    /// ```
    /// use geo::Stitch;
    /// use geo::{Coord, Triangle, Rect};
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
    /// let result = vec![tri1, tri2].stitch_together();
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
    /// let expected = Rect::new(Coord { x: 0.0, y: 0.0 }, Coord { x: 1.0, y: 1.0 }).to_polygon();
    ///
    /// assert_eq!(poly, expected);
    /// ```
    ///
    /// # Additional Notes
    ///
    /// If you want to do something more general like a
    /// [`Boolean Operation Union`](https://en.wikipedia.org/wiki/Boolean_operations_on_polygons)
    /// you should use the trait `BooleanOps` or `SpadeBoolops`.
    fn stitch_together(&self) -> PolygonStitchingResult<MultiPolygon<T>>;
}

macro_rules! impl_stitch {
    ($type:ty => $self:ident
     fn stitch_together(&self) -> PolygonStitchingResult<MultiPolygon<T>> {
         $($impls:tt)*
     }) => {
       impl<T> Stitch<T> for &[&$type]
       where
           T: GeoFloat,
       {
           fn stitch_together(&$self) -> PolygonStitchingResult<MultiPolygon<T>> {
                $($impls)*
           }
       }

       impl<T> Stitch<T> for Vec<&$type>
       where
           T: GeoFloat,
       {
           fn stitch_together(&$self) -> PolygonStitchingResult<MultiPolygon<T>> {
               $self.as_slice().stitch_together()
           }
       }

       impl<T> Stitch<T> for &[$type]
       where
           T: GeoFloat,
       {
           fn stitch_together(&$self) -> PolygonStitchingResult<MultiPolygon<T>> {
               $self.iter().collect::<Vec<_>>().stitch_together()
           }
       }

       impl<T> Stitch<T> for Vec<$type>
       where
           T: GeoFloat,
       {
           fn stitch_together(&$self) -> PolygonStitchingResult<MultiPolygon<T>> {
               $self.iter().collect::<Vec<_>>().stitch_together()
           }
       }
     };
}

impl_stitch! {
    Polygon<T> => self
    fn stitch_together(&self) -> PolygonStitchingResult<MultiPolygon<T>> {
        let polys = self
            .iter()
            .map(|&poly| fix_orientation(poly.clone()))
            .collect::<Vec<_>>();
        let lines = polys
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

impl_stitch! {
    Triangle<T> => self
    fn stitch_together(&self) -> PolygonStitchingResult<MultiPolygon<T>> {
        self.iter()
            .map(|tri| tri.to_polygon())
            .collect::<Vec<_>>()
            .stitch_together()
    }
}

impl_stitch! {
    MultiPolygon<T> => self
    fn stitch_together(&self) -> PolygonStitchingResult<MultiPolygon<T>> {
        self.iter().flat_map(|mp| mp.0.iter()).collect::<Vec<_>>().stitch_together()
    }
}

/// makes interiors and exteriors of polygon have ccw orientation
fn fix_orientation<T: GeoFloat>(mut poly: Polygon<T>) -> Polygon<T> {
    poly.exterior_mut(|ls| ls.make_ccw_winding());

    // just commenting this out for the moment. The algorithm only works on exteriors so this
    // shouldn't be needed
    //
    //poly.interiors_mut(|ls| ls.iter_mut().for_each(|ls| ls.make_ccw_winding()));

    poly
}

/// checks whether the to lines are equal or inverted forms of each other
fn same_line<T: GeoFloat>(l1: &Line<T>, l2: &Line<T>) -> bool {
    let flipped_l2 = Line::new(l2.end, l2.start);
    l1 == l2 || l1 == &flipped_l2
}

/// given a collection of lines from multiple polygons, this returns all but the shared lines
fn find_boundary_lines<T: GeoFloat>(lines: Vec<Line<T>>) -> Vec<Line<T>> {
    let enumerated_lines = || lines.iter().enumerate();
    enumerated_lines()
        // only collect lines that don't have a duplicate in the set
        .filter_map(|(i, line)| {
            let same_line_exists = enumerated_lines()
                .filter(|&(j, _)| j != i)
                .any(|(_, l)| same_line(line, l));
            (!same_line_exists).then_some(*line)
        })
        .collect::<Vec<_>>()
}

// Notes for future: This probably belongs into a `Validify` trait or something
/// finds holes in polygon exterior and fixes them
///
/// This is important for scenarios like the banana polygon. Which is considered invalid
/// https://www.postgis.net/workshops/postgis-intro/validity.html#repairing-invalidity
fn find_and_fix_holes_in_exterior<F: GeoFloat>(mut poly: Polygon<F>) -> Polygon<F> {
    fn detect_if_rings_closed_with_point<F: GeoFloat>(
        points: &mut Vec<&Coord<F>>,
        p: &Coord<F>,
    ) -> Option<Vec<Coord<F>>> {
        // early return here if nothing was found
        let pos = points.iter().position(|&c| c == p)?;

        // create ring by collecting the points if something was found
        let ring = points
            .drain(pos..)
            .chain(std::iter::once(p))
            .cloned()
            .collect::<Vec<_>>();
        Some(ring)
    }

    // find rings
    let rings = {
        let (points, mut rings) =
            poly.exterior()
                .into_iter()
                .fold((vec![], vec![]), |(mut points, mut rings), coord| {
                    if let Some(ring) = detect_if_rings_closed_with_point(&mut points, coord) {
                        rings.push(ring);
                    }
                    points.push(coord);
                    (points, rings)
                });

        // add leftover coords as last ring
        let last_ring = points.into_iter().cloned().collect::<Vec<_>>();
        rings.push(last_ring);

        rings
    };

    // convert to polygons for containment checks
    let mut rings = rings
        .into_iter()
        // filter out degenerate polygons which may be produced from the code above
        .filter(|cs| cs.len() >= 3)
        .map(|cs| Polygon::new(LineString::new(cs), vec![]))
        .collect::<Vec<_>>();

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
) -> PolygonStitchingResult<MultiPolygon<F>> {
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
    let parents: HashMap<usize, Vec<usize>> = rings
        .iter()
        .enumerate()
        .map(|(ring_idx, ring)| {
            let parent_idxs = find_parent_idxs(ring_idx, ring, &rings);
            (ring_idx, parent_idxs)
        })
        .collect();

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

        // the direct parent is the parent which has itself the most parents
        fn find_direct_parents(
            parents_indexs: &[usize],
            parents: &HashMap<usize, Vec<usize>>,
        ) -> Option<usize> {
            parents_indexs
                .iter()
                .filter_map(|parent_idx| {
                    parents
                        .get(parent_idx)
                        .map(|grandparents| (parent_idx, grandparents))
                })
                .max_by_key(|(_, grandparents)| grandparents.len())
                .map(|(idx, _)| idx)
                .copied()
        }

        // if it has an odd number of parents, it's an inner ring
        //
        // to find the specific outer ring it is related to, we search for the direct parent.
        if let Some(direct_parent) = find_direct_parents(parent_idxs, &parents) {
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
) -> PolygonStitchingResult<Vec<LineString<F>>> {
    // initial ring parts are just lines which will be stitch together progressively
    let mut ring_parts: Vec<Vec<Coord<F>>> = lines
        .iter()
        .map(|line| vec![line.start, line.end])
        .collect();

    let mut rings: Vec<LineString<F>> = vec![];
    // terminates since every loop we'll merge two elements into one so the total number of
    // elements decreases each loop by 1
    while let Some(last_part) = ring_parts.pop() {
        let (j, compound_part) = ring_parts
            .iter()
            .enumerate()
            .find_map(|(j, other_part)| {
                let new_part = try_stitch(&last_part, other_part)?;
                Some((j, new_part))
            })
            .ok_or(LineStitchingError::IncompleteRing)?;
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

    use crate::{Area, TriangulateEarcut};

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

        let result = tris.stitch_together().unwrap();

        assert!(mp.contains(&result) && result.contains(&mp));
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
        let result_1 = vec![&tri1, &tri2].stitch_together().unwrap();

        tri1.exterior_mut(|ls| ls.make_cw_winding());
        tri2.exterior_mut(|ls| ls.make_ccw_winding());
        let result_2 = vec![&tri1, &tri2].stitch_together().unwrap();

        tri1.exterior_mut(|ls| ls.make_cw_winding());
        tri2.exterior_mut(|ls| ls.make_cw_winding());
        let result_3 = vec![&tri1, &tri2].stitch_together().unwrap();

        tri1.exterior_mut(|ls| ls.make_ccw_winding());
        tri2.exterior_mut(|ls| ls.make_cw_winding());
        let result_4 = vec![&tri1, &tri2].stitch_together().unwrap();

        assert_eq!(result_1.unsigned_area(), result_2.unsigned_area());
        assert_eq!(result_2.unsigned_area(), result_3.unsigned_area());
        assert_eq!(result_3.unsigned_area(), result_4.unsigned_area());
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

        let result = vec![poly1, poly2].stitch_together().unwrap();

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

        let result = vec![poly].stitch_together().unwrap();

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

        let result = vec![poly].stitch_together().unwrap();

        assert_eq!(result.0.len(), 1);
        assert_eq!(result.0[0].interiors().len(), 0);
    }
}
