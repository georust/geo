//! Repair invalid polygons using constrained Delaunay triangulation.
//!
//! This module implements the polygon repair algorithm described in:
//!
//! > Ledoux, H., Arroyo Ohori, K., and Meijers, M. (2014). A
//! > triangulation-based approach to automatically repair GIS polygons.
//! > *Computers & Geosciences* 66:121--131.
//! > <http://dx.doi.org/10.1016/j.cageo.2014.01.009>
//!
//! The algorithm builds a constrained Delaunay triangulation from the
//! polygon's edges, labels each triangle as interior or exterior using an
//! odd-even flood-fill, then reconstructs valid boundaries from the
//! labelled triangulation. The result is always a valid [crate::geometry::MultiPolygon],
//! since a single invalid input may decompose into several valid components.
//!
//! # When to use this
//!
//! [`MakeValid`] is useful when you receive polygon data from an untrusted
//! or error-prone source (shapefiles, user input, format conversions) and
//! need geometrically valid output for downstream operations like boolean
//! ops, buffering, or triangulation.
//!
//! # Fill rule
//!
//! This implementation uses the **odd-even** (even-odd) fill rule from
//! the Ledoux et al. paper. Each time the flood-fill crosses a
//! constraint edge, it flips between interior and exterior. This means
//! edges appearing an even number of times cancel out (shared edges,
//! dangling protrusions), and nested interior rings become islands
//! (interior again) rather than remaining holes.
//!
//! The paper also describes an alternative **set-difference** rule that
//! treats each interior ring as a subtraction regardless of nesting
//! depth. This is not currently implemented; under set-difference,
//! doubly-nested holes stay subtracted rather than becoming islands.
//!
//! # What kinds of invalidity are handled
//!
//! The odd-even approach naturally handles a wide variety of geometric
//! errors. The examples below show common cases; all use the same
//! [`MakeValid::make_valid`] entry point.
//!
//! ## Self-intersecting polygon (bowtie)
//!
//! A polygon whose boundary crosses itself decomposes into separate
//! components at the crossing point.
//!
//! ```
//! # use geo::{MakeValid, wkt, Area};
//! // Boundary crosses at (5, 5), forming two triangles
//! let bowtie = wkt!(POLYGON((0. 0., 0. 10., 10. 0., 10. 10., 0. 0.)));
//! let repaired = bowtie.make_valid().expect("repair failed");
//! assert_eq!(repaired.0.len(), 2);
//! ```
//!
//! ## Dangling edge
//!
//! An edge that protrudes outward and doubles back contributes zero area.
//! Under the odd-even rule the two copies of the edge cancel, leaving only
//! the valid portion of the polygon.
//!
//! ```
//! # use geo::{MakeValid, wkt, Area};
//! # use approx::assert_relative_eq;
//! // The protrusion to (15, 5) and back has zero area
//! let poly = wkt!(POLYGON((0. 0., 10. 0., 15. 5., 10. 0., 10. 10., 0. 10., 0. 0.)));
//! let repaired = poly.make_valid().expect("repair failed");
//! assert_relative_eq!(repaired.unsigned_area(), 100.0);
//! ```
//!
//! ## Inner ring sharing an edge with the outer boundary
//!
//! When a hole shares an edge with the exterior ring, the shared edge
//! appears twice (once from each ring) and cancels under odd-even counting.
//! The result is a polygon with the hole correctly subtracted.
//!
//! ```
//! # use geo::{MakeValid, wkt, Area};
//! # use approx::assert_relative_eq;
//! let poly = wkt!(POLYGON(
//!     (0. 0., 10. 0., 10. 10., 0. 10., 0. 0.),
//!     (5. 2., 5. 7., 10. 7., 10. 2., 5. 2.)
//! ));
//! let repaired = poly.make_valid().expect("repair failed");
//! assert_relative_eq!(repaired.unsigned_area(), 75.0);
//! ```
//!
//! ## Hole equal to shell
//!
//! When the interior ring is identical to the exterior ring, every edge
//! appears twice and cancels completely. The result is empty.
//!
//! ```
//! # use geo::{MakeValid, wkt, Area};
//! # use approx::assert_relative_eq;
//! let poly = wkt!(POLYGON(
//!     (10. 90., 90. 90., 90. 10., 10. 10., 10. 90.),
//!     (10. 90., 90. 90., 90. 10., 10. 10., 10. 90.)
//! ));
//! let repaired = poly.make_valid().expect("repair failed");
//! assert_relative_eq!(repaired.unsigned_area(), 0.0);
//! ```
//!
//! ## Nested shells (MultiPolygon)
//!
//! Two concentric polygons in a `MultiPolygon` overlap entirely. Under
//! odd-even, the inner region is crossed by edges from both shells (even
//! count) so it becomes exterior, leaving an annular result.
//!
//! ```
//! # use geo::{MakeValid, wkt, Area};
//! # use approx::assert_relative_eq;
//! let mp = wkt!(MULTIPOLYGON(
//!     ((30. 70., 70. 70., 70. 30., 30. 30., 30. 70.)),
//!     ((10. 90., 90. 90., 90. 10., 10. 10., 10. 90.))
//! ));
//! let repaired = mp.make_valid().expect("repair failed");
//! // Outer 80x80 minus inner 40x40
//! assert_relative_eq!(repaired.unsigned_area(), 4800.0);
//! ```
//!
//! ## Self-touching ring (banana polygon)
//!
//! When a hole touches the exterior ring at exactly one vertex, the repair
//! algorithm traces a single boundary ring that visits the shared vertex
//! twice. Pinch-point splitting detects the repeated vertex and separates
//! the ring into the exterior and the hole.
//!
//! ```
//! # use geo::{MakeValid, wkt, Area};
//! # use geo::validation::Validation;
//! let poly = wkt!(POLYGON(
//!     (0. 0., 10. 0., 10. 10., 0. 10., 0. 0.),
//!     (5. 0., 8. 3., 2. 3., 5. 0.)
//! ));
//! let repaired = poly.make_valid().expect("repair failed");
//! for p in repaired.iter() {
//!     assert!(p.is_valid());
//! }
//! ```
//!
//! # Complexity
//!
//! Let:
//! - **n** = number of input edges (from all rings of all input polygons)
//! - **m** = number of x-overlapping edge pairs considered by the sweep
//!   (m >= k, but typically m ~ O(n) for polygon edges)
//! - **k** = number of actual edge-edge intersections
//! - **f** = number of triangulation faces (f = O(n + k))
//! - **e** = number of exterior rings in the output
//! - **h** = number of holes in the output
//! - **r** = maximum ring size in the output
//! - **p** = total number of pinch-point vertices across all traced rings
//!
//! | Phase | Typical | Worst case |
//! |---|---|---|
//! | Snap endpoints (R-tree) | O(n log n) | O(n log n) |
//! | Odd-even dedup (sort) | O(n log n) | O(n log n) |
//! | Sweep intersection detection | O((n + m) log n) | O((n + m) log n) |
//! | Segment splitting | O(k log k) | O(k log k) |
//! | CDT construction | O(f log f) | O(f^2) |
//! | Flood-fill labelling (BFS) | O(f) | O(f) |
//! | Boundary tracing | O(f) | O(f) |
//! | Pinch-point splitting | O(p) | O(r^2) per ring |
//! | Polygon assembly (hole assignment) | O(h * r) | O(h * e * r) |
//!
//! For typical polygon repair inputs (sparse intersections, few pinch
//! points, few output components), the overall complexity is dominated
//! by the sweep at **O(n log n)**.
//!
//! # Errors
//!
//! [`MakeValid::make_valid`] returns [`repair_polygon::RepairPolygonError`] if the input
//! contains coordinates that the triangulation kernel cannot handle (NaN,
//! too large, or too small) or if an internal constraint edge insertion
//! fails.

use std::collections::{HashSet, VecDeque};

use geo_types::{Coord, Line, LineString, MultiPolygon, Polygon};
use spade::handles::{FixedFaceHandle, InnerTag};
use spade::{ConstrainedDelaunayTriangulation, Triangulation};

use crate::algorithm::area::get_linestring_area;
use crate::algorithm::triangulate_delaunay::{
    DelaunayTriangulationConfig, SpadeTriangulationFloat, TriangulationError,
    split_segments_at_intersections, sweep_line_ord, sweep_normalize_line,
};
use crate::coordinate_position::{CoordPos, coord_pos_relative_to_ring};
use crate::{GeoFloat, LinesIter};

#[cfg(test)]
mod tests;

// ====== Error ======

/// Errors that can occur during polygon repair.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum RepairPolygonError {
    /// A coordinate value was NaN.
    CoordinateIsNaN,
    /// A coordinate value was too large for the triangulation kernel.
    CoordinateTooLarge,
    /// A coordinate value was too small (non-zero but below the
    /// triangulation kernel's minimum representable value).
    CoordinateTooSmall,
    /// An internal constraint edge could not be inserted into the
    /// triangulation. This typically indicates a numerical precision
    /// issue in the input geometry.
    ConstraintFailure,
}

impl std::fmt::Display for RepairPolygonError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::CoordinateIsNaN => write!(f, "coordinate value is NaN"),
            Self::CoordinateTooLarge => {
                write!(f, "coordinate value exceeds triangulation kernel limit")
            }
            Self::CoordinateTooSmall => write!(
                f,
                "coordinate value is non-zero but below triangulation kernel minimum"
            ),
            Self::ConstraintFailure => {
                write!(f, "internal constraint edge insertion failed")
            }
        }
    }
}

impl std::error::Error for RepairPolygonError {}

impl From<TriangulationError> for RepairPolygonError {
    fn from(err: TriangulationError) -> Self {
        match err {
            TriangulationError::SpadeError(spade::InsertionError::NAN) => Self::CoordinateIsNaN,
            TriangulationError::SpadeError(spade::InsertionError::TooLarge) => {
                Self::CoordinateTooLarge
            }
            TriangulationError::SpadeError(spade::InsertionError::TooSmall) => {
                Self::CoordinateTooSmall
            }
            TriangulationError::ConstraintFailure | TriangulationError::LoopTrap => {
                Self::ConstraintFailure
            }
        }
    }
}

// ====== Public API ======

/// Repair an invalid polygon using constrained Delaunay triangulation.
///
/// See the [module-level documentation](self) for algorithm details,
/// examples, complexity analysis, and supported invalidity types.
pub trait MakeValid<T: GeoFloat> {
    /// Repair this geometry, producing a valid
    /// [`MultiPolygon`].
    ///
    /// Returns [`RepairPolygonError`] if the input contains non-finite
    /// coordinates or if constraint insertion fails.
    fn make_valid(&self) -> Result<MultiPolygon<T>, RepairPolygonError>;
}

impl<T> MakeValid<T> for Polygon<T>
where
    T: SpadeTriangulationFloat,
{
    fn make_valid(&self) -> Result<MultiPolygon<T>, RepairPolygonError> {
        repair_polygon(self)
    }
}

impl<T> MakeValid<T> for MultiPolygon<T>
where
    T: SpadeTriangulationFloat,
{
    fn make_valid(&self) -> Result<MultiPolygon<T>, RepairPolygonError> {
        repair_multi_polygon(self)
    }
}

// ====== Core pipeline ======

fn repair_polygon<T: SpadeTriangulationFloat>(
    polygon: &Polygon<T>,
) -> Result<MultiPolygon<T>, RepairPolygonError> {
    let lines = extract_lines(polygon);
    if lines.is_empty() {
        return Ok(MultiPolygon::new(vec![]));
    }
    repair_from_lines(lines)
}

fn repair_multi_polygon<T: SpadeTriangulationFloat>(
    mp: &MultiPolygon<T>,
) -> Result<MultiPolygon<T>, RepairPolygonError> {
    let lines: Vec<Line<T>> = mp.iter().flat_map(extract_lines).collect();
    if lines.is_empty() {
        return Ok(MultiPolygon::new(vec![]));
    }
    repair_from_lines(lines)
}

fn repair_from_lines<T: SpadeTriangulationFloat>(
    lines: Vec<Line<T>>,
) -> Result<MultiPolygon<T>, RepairPolygonError> {
    // Check for non-finite coordinates before they reach the R-tree or
    // triangulation kernel (both of which can panic on NaN).
    for line in &lines {
        for coord in [line.start, line.end] {
            if coord.x.is_nan() || coord.y.is_nan() {
                return Err(RepairPolygonError::CoordinateIsNaN);
            }
            if coord.x.is_infinite() || coord.y.is_infinite() {
                return Err(RepairPolygonError::CoordinateTooLarge);
            }
        }
    }

    let snap_radius = DelaunayTriangulationConfig::<T>::default().snap_radius;
    let cdt = build_cdt(lines, snap_radius)?;

    if cdt.num_inner_faces() == 0 {
        return Ok(MultiPolygon::empty());
    }

    let interior = label_faces(&cdt);
    if interior.is_empty() {
        return Ok(MultiPolygon::new(vec![]));
    }

    let rings = trace_rings(&cdt, &interior);
    let rings: Vec<LineString<T>> = rings
        .into_iter()
        .flat_map(|r| split_ring_at_pinch_points(r.0))
        .map(LineString::new)
        .collect();
    Ok(assemble_polygons(rings))
}

// ====== Step 1: Extract edges from polygon ======

fn extract_lines<T: GeoFloat>(polygon: &Polygon<T>) -> Vec<Line<T>> {
    polygon.lines_iter().collect()
}

// ====== Step 2: Build CDT ======

/// Snap endpoints, then apply odd-even edge counting before and after
/// splitting at intersections.
///
/// Odd-even filtering is specific to polygon repair: edges appearing an even
/// number of times cancel out (shared edges between rings, dangling edges that
/// double back). We do NOT call `preprocess_lines` here because its simple
/// dedup would reduce duplicate-pair counts to 1 before odd-even sees them.
///
/// The second `odd_even_filter` call is needed because
/// `split_segments_at_intersections` can produce new duplicates from collinear
/// overlapping segments. Two segments that partially overlap are different
/// lines, so the first filter won't catch them. After splitting, the
/// overlapping portion appears as an identical sub-segment from both original
/// segments — the second filter cancels these.
fn prepare_constraint_lines<T: SpadeTriangulationFloat>(
    lines: Vec<Line<T>>,
    snap_radius: T,
) -> Result<Vec<Line<T>>, RepairPolygonError> {
    use crate::algorithm::triangulate_delaunay::snap_or_register_point;
    use rstar::RTree;

    let mut known_coords: RTree<Coord<T>> = RTree::new();
    let mut lines: Vec<Line<T>> = lines
        .into_iter()
        .map(|mut line| {
            line.start = snap_or_register_point(line.start, &mut known_coords, snap_radius);
            line.end = snap_or_register_point(line.end, &mut known_coords, snap_radius);
            line
        })
        .filter(|l| l.start != l.end)
        .collect();
    odd_even_filter(&mut lines);
    let mut lines = split_segments_at_intersections(lines, known_coords, snap_radius)?;
    odd_even_filter(&mut lines);
    Ok(lines)
}

fn build_cdt<T: SpadeTriangulationFloat>(
    lines: Vec<Line<T>>,
    snap_radius: T,
) -> Result<ConstrainedDelaunayTriangulation<Coord<T>>, RepairPolygonError> {
    let lines = prepare_constraint_lines(lines, snap_radius)?;

    lines.into_iter().try_fold(
        ConstrainedDelaunayTriangulation::<Coord<T>>::new(),
        |mut cdt, line| {
            let start = cdt
                .insert(line.start)
                .map_err(TriangulationError::SpadeError)?;
            let end = cdt
                .insert(line.end)
                .map_err(TriangulationError::SpadeError)?;
            if start != end && cdt.can_add_constraint(start, end) {
                cdt.add_constraint(start, end);
            }
            Ok(cdt)
        },
    )
}

/// Normalize direction, sort, then apply odd-even filtering: edges that appear
/// an even number of times cancel out (as in prepair's constraint insertion
/// semantics), while edges appearing an odd number of times are kept once.
fn odd_even_filter<T: GeoFloat>(lines: &mut Vec<Line<T>>) {
    for line in lines.iter_mut() {
        *line = sweep_normalize_line(*line);
    }
    lines.sort_by(sweep_line_ord);

    let mut result = Vec::new();
    let mut i = 0;
    while i < lines.len() {
        let start = i;
        while i < lines.len() && lines[i] == lines[start] {
            i += 1;
        }
        if (i - start) % 2 == 1 {
            result.push(lines[start]);
        }
    }
    *lines = result;
}

// ====== Step 3: Flood-fill face labelling ======

/// Label each inner face of the CDT as interior or exterior using an odd-even
/// flood-fill from the outer (infinite) face.
///
/// Returns the set of fixed face handles that are interior.
fn label_faces<T: SpadeTriangulationFloat>(
    cdt: &ConstrainedDelaunayTriangulation<Coord<T>>,
) -> HashSet<FixedFaceHandle<InnerTag>> {
    let mut interior: HashSet<FixedFaceHandle<InnerTag>> = HashSet::new();
    let mut visited: HashSet<FixedFaceHandle<InnerTag>> = HashSet::new();

    // (face_handle, is_interior)
    let mut queue: VecDeque<(FixedFaceHandle<InnerTag>, bool)> = VecDeque::new();

    // Seed: find inner faces adjacent to the outer face.
    // Each directed edge whose face() is the outer face has a reversed edge
    // whose face() is an inner face on the triangulation boundary.
    for edge in cdt.directed_edges() {
        if !edge.face().is_outer() {
            continue;
        }
        let opposite = edge.rev().face();
        let Some(inner) = opposite.as_inner() else {
            debug_assert!(
                false,
                "Each directed edge whose face() is the outer face has a reversed edge whose face() is an inner face"
            );
            continue;
        };
        let handle = inner.fix();
        if visited.insert(handle) {
            let is_interior = edge.is_constraint_edge();
            if is_interior {
                interior.insert(handle);
            }
            queue.push_back((handle, is_interior));
        }
    }

    // BFS: propagate labels across edges
    while let Some((face_handle, face_is_interior)) = queue.pop_front() {
        let face = cdt.face(face_handle);
        for edge in face.adjacent_edges() {
            let neighbour = edge.rev().face();
            if neighbour.is_outer() {
                continue;
            }
            let Some(inner_neighbour) = neighbour.as_inner() else {
                // neighbor.is_outer
                continue;
            };
            let n_handle = inner_neighbour.fix();
            if !visited.insert(n_handle) {
                continue;
            }
            let crosses_constraint = edge.is_constraint_edge();
            let neighbour_is_interior = if crosses_constraint {
                !face_is_interior
            } else {
                face_is_interior
            };
            if neighbour_is_interior {
                interior.insert(n_handle);
            }
            queue.push_back((n_handle, neighbour_is_interior));
        }
    }

    interior
}

// ====== Step 4: Boundary tracing ======

/// Trace boundary rings from the labelled triangulation.
///
/// A boundary directed edge has an interior face on its left and an exterior
/// (or outer) face on its right. Following these edges around produces closed
/// rings.
fn trace_rings<T: SpadeTriangulationFloat>(
    cdt: &ConstrainedDelaunayTriangulation<Coord<T>>,
    interior: &HashSet<FixedFaceHandle<InnerTag>>,
) -> Vec<LineString<T>> {
    use spade::handles::FixedDirectedEdgeHandle;

    let is_face_interior = |face: spade::handles::FaceHandle<
        '_,
        spade::handles::PossiblyOuterTag,
        Coord<T>,
        _,
        _,
        _,
    >| {
        face.as_inner()
            .map(|f| interior.contains(&f.fix()))
            .unwrap_or(false)
    };

    // Collect all boundary directed edge handles (interior on left, exterior on right)
    let boundary_edges: HashSet<FixedDirectedEdgeHandle> = cdt
        .directed_edges()
        .filter(|e| is_face_interior(e.face()) && !is_face_interior(e.rev().face()))
        .map(|e| e.fix())
        .collect();

    let mut used: HashSet<FixedDirectedEdgeHandle> = HashSet::new();
    let mut rings = Vec::new();

    for &start_handle in &boundary_edges {
        if used.contains(&start_handle) {
            continue;
        }

        let mut ring_coords: Vec<Coord<T>> = Vec::new();
        let mut current_fix = start_handle;
        // Outer safety: a ring can have at most as many edges as there are
        // boundary edges, so we use that as an upper bound to guard against
        // infinite loops caused by degenerate triangulations.
        let mut outer_safety = boundary_edges.len();

        loop {
            used.insert(current_fix);
            let current = cdt.directed_edge(current_fix);
            let pos = current.from().position();
            ring_coords.push(Coord { x: pos.x, y: pos.y });

            // Find next boundary edge by rotating around current.to()
            let mut candidate = current.next();
            // Inner safety: bounded by the finite number of edges around the vertex
            let mut inner_safety = cdt.num_directed_edges();
            loop {
                if boundary_edges.contains(&candidate.fix()) && !used.contains(&candidate.fix()) {
                    break;
                }
                // If we've come back to the start edge, the ring is complete
                if candidate.fix() == start_handle {
                    break;
                }
                candidate = candidate.rev().next();
                inner_safety = inner_safety.saturating_sub(1);
                if inner_safety == 0 {
                    debug_assert!(
                        false,
                        "inner rotation safety limit reached: could not find next boundary edge around vertex"
                    );
                    break;
                }
            }

            current_fix = candidate.fix();
            if current_fix == start_handle {
                break;
            }
            outer_safety = outer_safety.saturating_sub(1);
            if outer_safety == 0 {
                debug_assert!(
                    false,
                    "outer ring safety limit reached: ring has more edges than boundary edges"
                );
                break;
            }
        }

        // Close the ring
        if let Some(&first) = ring_coords.first() {
            ring_coords.push(first);
        }

        if ring_coords.len() >= 4 {
            rings.push(LineString::new(ring_coords));
        }
    }

    rings
}

// ====== Step 4b: Split rings at pinch points ======

/// Split a closed ring at vertices that appear more than once ("pinch points").
///
/// A pinch point arises when a boundary ring self-touches at a single vertex,
/// for example when a hole shares a vertex with the exterior ring. The single
/// ring is split into multiple closed sub-rings at each repeated vertex.
///
/// Sub-rings with fewer than 3 distinct vertices (e.g. degenerate spikes)
/// are dropped.
fn split_ring_at_pinch_points<T: GeoFloat>(ring: Vec<Coord<T>>) -> Vec<Vec<Coord<T>>> {
    // Open the ring (remove closing duplicate)
    let mut coords = ring;
    debug_assert!(
        coords.len() >= 2 && coords.first() == coords.last(),
        "ring passed to split_ring_at_pinch_points must be closed"
    );
    if coords.len() >= 2 && coords.first() == coords.last() {
        coords.pop();
    }

    if coords.len() < 3 {
        return vec![];
    }

    // Repeatedly find and extract the first loop caused by a repeated vertex.
    // Each iteration removes one loop, so this terminates.
    //
    // TODO: this is O(r^2) per ring (nested scan to find the first repeated
    // vertex, restarted after each extraction). It could be reduced to O(r)
    // by maintaining a HashMap keyed on the bit-representation of each Coord
    // (e.g. (f64::to_bits(x), f64::to_bits(y))) to detect repeats in a
    // single pass. In practice pinch points are rare and rings are short,
    // so the quadratic cost is unlikely to dominate, but it would matter for
    // very large rings with many pinch points.
    let mut result: Vec<Vec<Coord<T>>> = Vec::new();
    loop {
        let repeat = coords.iter().enumerate().find_map(|(i, c)| {
            coords[i + 1..]
                .iter()
                .position(|other| other == c)
                .map(|j| (i, i + 1 + j))
        });

        match repeat {
            Some((first, second)) => {
                // coords[first..=second] is a closed sub-ring
                let sub: Vec<Coord<T>> = coords[first..=second].to_vec();
                // Remove the loop body from coords, keeping the vertex at `first`
                coords.drain((first + 1)..=second);
                // Recursively split the extracted sub-ring -- it may itself
                // contain further pinch points (e.g. self-overlapping shells
                // produce nested repeated vertices).
                if sub.len() >= 4 {
                    result.extend(split_ring_at_pinch_points(sub));
                }
            }
            None => break,
        }
    }

    // Close the remaining ring
    if coords.len() >= 3 {
        let first = coords[0];
        coords.push(first);
        result.push(coords);
    }

    result
}

// ====== Step 5: Assemble polygons ======

/// Classify rings as exterior or interior (holes) by signed area, then group
/// holes inside their parent exterior ring.
fn assemble_polygons<T: SpadeTriangulationFloat>(rings: Vec<LineString<T>>) -> MultiPolygon<T> {
    let mut exteriors: Vec<LineString<T>> = Vec::new();
    let mut holes: Vec<LineString<T>> = Vec::new();

    for ring in rings {
        // Positive signed area -> CCW -> exterior ring
        // Negative signed area -> CW -> hole
        let area = get_linestring_area(&ring);
        if area > T::zero() {
            exteriors.push(ring);
        } else if area < T::zero() {
            holes.push(ring);
        }
        // zero-area rings are degenerate, skip them
    }

    if exteriors.is_empty() {
        return MultiPolygon::new(vec![]);
    }

    // Build polygons: each exterior ring gets a vec of holes
    let mut polygons: Vec<(LineString<T>, Vec<LineString<T>>)> =
        exteriors.into_iter().map(|ext| (ext, Vec::new())).collect();

    // Assign each hole to its containing exterior ring.
    // We try multiple hole vertices because the first one may lie exactly
    // on the exterior boundary (e.g. a pinch point), which returns
    // OnBoundary rather than Inside.
    //
    // TODO: this is O(h * e * r) where h = number of holes, e = number of
    // exterior rings, and r = max ring size (for the point-in-ring test).
    // For polygons that decompose into many separate components (large e and
    // h), this could be improved by building an R-tree over the exterior ring
    // bounding boxes and querying it for each hole vertex, reducing the
    // per-hole cost from O(e * r) to O(log(e) + r).
    'next_hole: for hole in holes {
        for pt in &hole.0 {
            for (ext, ext_holes) in &mut polygons {
                if coord_pos_relative_to_ring(*pt, ext) == CoordPos::Inside {
                    ext_holes.push(hole);
                    continue 'next_hole;
                }
            }
        }
        // If no vertex is unambiguously inside an exterior, the hole is dropped
    }

    MultiPolygon::new(
        polygons
            .into_iter()
            .map(|(ext, holes)| Polygon::new(ext, holes))
            .collect(),
    )
}
