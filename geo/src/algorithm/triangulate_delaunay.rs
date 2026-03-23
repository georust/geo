use geo_types::{Coord, Line, Point, Triangle};
use rstar::{RTree, RTreeNum};
use spade::{ConstrainedDelaunayTriangulation, DelaunayTriangulation, SpadeNum, Triangulation};

use crate::{Centroid, Contains};
use crate::{CoordsIter, Distance, Euclidean, GeoFloat, LineIntersection, LinesIter, Vector2DOps};

// ======== Config ============

/// Collection of parameters that influence the precision of the algorithm in some sense (see
/// explanations on fields of this struct)
///
/// This implements the `Default` trait and you can just use it most of the time
#[derive(Debug, Clone)]
pub struct DelaunayTriangulationConfig<T: SpadeTriangulationFloat> {
    /// Coordinates within this radius are snapped to the same position. For any two `Coords` there's
    /// no real way to influence the decision when choosing the snapper and the snappee
    pub snap_radius: T,
}

impl<T> Default for DelaunayTriangulationConfig<T>
where
    T: SpadeTriangulationFloat,
{
    fn default() -> Self {
        Self {
            snap_radius: <T as std::convert::From<f32>>::from(0.000_1),
        }
    }
}

// ====== Error ========

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum TriangulationError {
    SpadeError(spade::InsertionError),
    LoopTrap,
    ConstraintFailure,
}

impl std::fmt::Display for TriangulationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{self:?}")
    }
}

impl std::error::Error for TriangulationError {}

pub type TriangulationResult<T> = Result<T, TriangulationError>;

// ======= Float trait ========

pub trait SpadeTriangulationFloat: GeoFloat + SpadeNum + RTreeNum {}
impl<T: GeoFloat + SpadeNum + RTreeNum> SpadeTriangulationFloat for T {}

// ======= Triangulation trait =========

pub type Triangles<T> = Vec<Triangle<T>>;

// Sealed traits for triangulation requirements. Split into two:
// - UnconstrainedRequirementTrait: only needs coords (for point collections)
// - ConstrainedRequirementTrait: needs coords + lines + contains (for polygons etc.)
mod private {
    use super::*;

    pub(crate) type CoordsIter<'a, T> = Box<dyn Iterator<Item = Coord<T>> + 'a>;

    /// Requirement trait for unconstrained triangulation: only needs coordinates
    pub trait UnconstrainedRequirementTrait<'a, T>
    where
        T: SpadeTriangulationFloat,
    {
        /// Collect all coords from the geometry
        fn coords(&'a self) -> CoordsIter<'a, T>;
    }

    /// Requirement trait for constrained triangulation: needs coords, lines, and containment
    pub trait ConstrainedRequirementTrait<'a, T>: UnconstrainedRequirementTrait<'a, T>
    where
        T: SpadeTriangulationFloat,
    {
        /// Collect all lines from the geometry (intersecting lines are allowed)
        fn lines(&'a self) -> Vec<Line<T>>;

        /// Check if a point is inside the geometry (used for constrained triangulation)
        fn contains_point(&'a self, p: Point<T>) -> bool;

        // Processing of the lines that prepare the lines for triangulation.
        //
        // Spade has the limitation that constraint lines cannot intersect or else it
        // will panic. This is why we need to manually split up the lines into smaller
        // parts at the intersection point.
        //
        // There's also a preprocessing step which tries to minimize the risk of failure
        // through edge cases (thin/flat triangles are prevented as much as possible,
        // lines are deduped, etc.)
        fn cleanup_lines(lines: Vec<Line<T>>, snap_radius: T) -> TriangulationResult<Vec<Line<T>>> {
            let (known_coords, lines) = preprocess_lines(lines, snap_radius);
            split_segments_at_intersections(lines, known_coords, snap_radius)
        }
    }
}

/// Unconstrained Delaunay triangulation for geometries that provide coordinates.
///
/// This trait provides triangulation methods based only on point coordinates,
/// making it suitable for point collections like `MultiPoint`, `Vec<Point>`, etc.
///
/// For constrained triangulation methods that respect geometries' existing boundaries,
/// see [`TriangulateDelaunay`].
pub trait TriangulateDelaunayUnconstrained<'a, T>:
    private::UnconstrainedRequirementTrait<'a, T>
where
    T: SpadeTriangulationFloat,
{
    /// Returns a triangulation based solely on the points of the geometric object.
    ///
    /// The triangulation is guaranteed to be Delaunay.
    ///
    /// Note that the lines of the triangulation don't necessarily follow the lines of the input
    /// geometry. If you wish to achieve that, see [`TriangulateDelaunay::constrained_triangulation`]
    /// and [`TriangulateDelaunay::constrained_outer_triangulation`].
    ///
    /// ```rust
    /// use geo::TriangulateDelaunayUnconstrained;
    /// use geo::{MultiPoint, Point};
    ///
    /// let points = MultiPoint::new(vec![
    ///     Point::new(0.0, 0.0),
    ///     Point::new(1.0, 0.0),
    ///     Point::new(0.5, 1.0),
    /// ]);
    /// let triangulation = points.unconstrained_triangulation().unwrap();
    /// assert_eq!(triangulation.len(), 1);
    /// ```
    fn unconstrained_triangulation(&'a self) -> TriangulationResult<Triangles<T>> {
        self.unconstrained_triangulation_raw()
            .map(triangulation_to_triangles)
    }

    /// Same as `unconstrained_triangulation`, but returns Spade's triangulation result directly.
    fn unconstrained_triangulation_raw(
        &'a self,
    ) -> TriangulationResult<DelaunayTriangulation<Coord<T>>> {
        let points = self.coords();
        points.into_iter().try_fold(
            DelaunayTriangulation::<Coord<T>>::new(),
            |mut tris, coord| {
                tris.insert(coord).map_err(TriangulationError::SpadeError)?;
                Ok(tris)
            },
        )
    }

    /// Same as `unconstrained_triangulation_raw`, but with configuration for snapping
    /// nearby points together before triangulation.
    fn unconstrained_triangulation_raw_with_config(
        &'a self,
        config: DelaunayTriangulationConfig<T>,
    ) -> TriangulationResult<DelaunayTriangulation<Coord<T>>> {
        let mut known_coords: RTree<Coord<T>> = RTree::new();
        // Snap points to nearby known coordinates within snap_radius
        for coord in self.coords() {
            snap_or_register_point(coord, &mut known_coords, config.snap_radius);
        }
        // Insert deduplicated points into triangulation
        known_coords.into_iter().try_fold(
            DelaunayTriangulation::<Coord<T>>::new(),
            |mut tris, coord| {
                tris.insert(coord).map_err(TriangulationError::SpadeError)?;
                Ok(tris)
            },
        )
    }
}

/// Triangulate polygons using a [Delaunay Triangulation](https://en.wikipedia.org/wiki/Delaunay_triangulation)
///
/// This trait extends [`TriangulateDelaunayUnconstrained`] with constrained triangulation methods
/// that respect geometry boundaries. To read more about the differences between constrained and
/// unconstrained methods, see the [Wikipedia article on constrained Delaunay triangulation](https://en.wikipedia.org/wiki/Constrained_Delaunay_triangulation).
pub trait TriangulateDelaunay<'a, T>:
    TriangulateDelaunayUnconstrained<'a, T> + private::ConstrainedRequirementTrait<'a, T>
where
    T: SpadeTriangulationFloat,
{
    /// Returns triangulation that's based on the points of the geometric object and also
    /// incorporates the lines of the input geometry.
    ///
    /// The triangulation is not guaranteed to be Delaunay because of the constraint lines.
    ///
    /// This outer triangulation also contains triangles that are not included in the input
    /// geometry if it wasn't convex. Here's an example:
    ///
    /// ```text
    /// ┌──────────────────┐
    /// │\              __/│
    /// │ \          __/ / │
    /// │  \      __/   /  │
    /// │   \  __/     /   │
    /// │    \/       /    │
    /// │     ┌──────┐     │
    /// │    /│\:::::│\    │
    /// │   / │:\::::│ \   │
    /// │  /  │::\:::│  \  │
    /// │ /   │:::\::│   \ │
    /// │/    │::::\:│    \│
    /// └─────┘______└─────┘
    /// ```
    ///
    /// ```rust
    /// use geo::TriangulateDelaunay;
    /// use geo::{Polygon, LineString, Coord};
    /// let u_shape = Polygon::new(
    ///     LineString::new(vec![
    ///         Coord { x: 0.0, y: 0.0 },
    ///         Coord { x: 1.0, y: 0.0 },
    ///         Coord { x: 1.0, y: 1.0 },
    ///         Coord { x: 2.0, y: 1.0 },
    ///         Coord { x: 2.0, y: 0.0 },
    ///         Coord { x: 3.0, y: 0.0 },
    ///         Coord { x: 3.0, y: 3.0 },
    ///         Coord { x: 0.0, y: 3.0 },
    ///     ]),
    ///     vec![],
    /// );
    /// // we use the default [`SpadeTriangulationConfig`] here
    /// let constrained_outer_triangulation =
    /// u_shape.constrained_outer_triangulation(Default::default()).unwrap();
    /// let num_triangles = constrained_outer_triangulation.len();
    /// assert_eq!(num_triangles, 8);
    /// ```
    ///
    /// The outer triangulation of the top down U-shape contains extra triangles marked
    /// with ":". If you want to exclude those, take a look at `constrained_triangulation`
    fn constrained_outer_triangulation(
        &'a self,
        config: DelaunayTriangulationConfig<T>,
    ) -> TriangulationResult<Triangles<T>> {
        let lines = self.lines();
        let lines = Self::cleanup_lines(lines, config.snap_radius)?;
        lines
            .into_iter()
            .try_fold(
                ConstrainedDelaunayTriangulation::<Coord<T>>::new(),
                |mut cdt, line| {
                    let start = cdt
                        .insert(line.start)
                        .map_err(TriangulationError::SpadeError)?;
                    let end = cdt
                        .insert(line.end)
                        .map_err(TriangulationError::SpadeError)?;
                    // safety check (to prevent panic) whether we can add the line
                    if !cdt.can_add_constraint(start, end) {
                        return Err(TriangulationError::ConstraintFailure);
                    }
                    cdt.add_constraint(start, end);
                    Ok(cdt)
                },
            )
            .map(triangulation_to_triangles)
    }

    /// Returns triangulation that's based on the points of the geometric object and also
    /// incorporates the lines of the input geometry.
    ///
    /// The triangulation is not guaranteed to be Delaunay because of the constraint lines.
    ///
    /// This triangulation only contains triangles that are included in the input geometry.
    /// Here's an example:
    ///
    /// ```text
    /// ┌──────────────────┐
    /// │\              __/│
    /// │ \          __/ / │
    /// │  \      __/   /  │
    /// │   \  __/     /   │
    /// │    \/       /    │
    /// │     ┌──────┐     │
    /// │    /│      │\    │
    /// │   / │      │ \   │
    /// │  /  │      │  \  │
    /// │ /   │      │   \ │
    /// │/    │      │    \│
    /// └─────┘      └─────┘
    /// ```
    ///
    /// ```rust
    /// use geo::TriangulateDelaunay;
    /// use geo::{Polygon, LineString, Coord};
    /// let u_shape = Polygon::new(
    ///     LineString::new(vec![
    ///         Coord { x: 0.0, y: 0.0 },
    ///         Coord { x: 1.0, y: 0.0 },
    ///         Coord { x: 1.0, y: 1.0 },
    ///         Coord { x: 2.0, y: 1.0 },
    ///         Coord { x: 2.0, y: 0.0 },
    ///         Coord { x: 3.0, y: 0.0 },
    ///         Coord { x: 3.0, y: 3.0 },
    ///         Coord { x: 0.0, y: 3.0 },
    ///     ]),
    ///     vec![],
    /// );
    /// // we use the default [`DelaunayTriangulationConfig`] here
    /// let constrained_triangulation = u_shape.constrained_triangulation(Default::default()).unwrap();
    /// let num_triangles = constrained_triangulation.len();
    /// assert_eq!(num_triangles, 6);
    /// ```
    ///
    /// Compared to the `constrained_outer_triangulation` it only includes the triangles
    /// inside of the input geometry
    fn constrained_triangulation(
        &'a self,
        config: DelaunayTriangulationConfig<T>,
    ) -> TriangulationResult<Triangles<T>> {
        self.constrained_outer_triangulation(config)
            .map(|triangles| {
                triangles
                    .into_iter()
                    .filter(|triangle| {
                        let center = triangle.centroid();
                        self.contains_point(center)
                    })
                    .collect::<Vec<_>>()
            })
    }
}

/// conversion from spade triangulation back to geo triangles
fn triangulation_to_triangles<T, F>(triangulation: T) -> Triangles<F>
where
    T: Triangulation<Vertex = Coord<F>>,
    F: SpadeTriangulationFloat,
{
    triangulation
        .inner_faces()
        .map(|face| {
            let [v0, v1, v2] = face.vertices();
            Triangle::new(*v0.data(), *v1.data(), *v2.data())
        })
        .collect::<Vec<_>>()
}

// ========== Triangulation trait impls ============

// Blanket impl: anything with UnconstrainedRequirementTrait gets TriangulateDelaunayUnconstrained
impl<'a, T, G> TriangulateDelaunayUnconstrained<'a, T> for G
where
    T: SpadeTriangulationFloat,
    G: private::UnconstrainedRequirementTrait<'a, T>,
{
}

// Blanket impl: anything with ConstrainedRequirementTrait gets TriangulateDelaunay
impl<'a, T, G> TriangulateDelaunay<'a, T> for G
where
    T: SpadeTriangulationFloat,
    G: private::ConstrainedRequirementTrait<'a, T>,
{
}

// Impl UnconstrainedRequirementTrait for types with CoordsIter
impl<'a, T, G> private::UnconstrainedRequirementTrait<'a, T> for G
where
    T: SpadeTriangulationFloat,
    G: CoordsIter<Scalar = T>,
{
    fn coords(&'a self) -> private::CoordsIter<'a, T> {
        Box::new(self.coords_iter())
    }
}

// Impl ConstrainedRequirementTrait for types with LinesIter + CoordsIter + Contains
impl<'a, 'l, T, G> private::ConstrainedRequirementTrait<'a, T> for G
where
    'a: 'l,
    T: SpadeTriangulationFloat,
    G: LinesIter<'l, Scalar = T> + CoordsIter<Scalar = T> + Contains<Point<T>>,
{
    fn lines(&'a self) -> Vec<Line<T>> {
        self.lines_iter().collect()
    }

    fn contains_point(&'a self, p: Point<T>) -> bool {
        self.contains(&p)
    }
}

impl<'a, T, G> private::UnconstrainedRequirementTrait<'a, T> for Vec<G>
where
    T: SpadeTriangulationFloat + 'a,
    G: TriangulateDelaunayUnconstrained<'a, T>,
{
    fn coords(&'a self) -> private::CoordsIter<'a, T> {
        Box::new(self.iter().flat_map(|g| g.coords()))
    }
}

impl<'a, T, G> private::ConstrainedRequirementTrait<'a, T> for Vec<G>
where
    T: SpadeTriangulationFloat + 'a,
    G: TriangulateDelaunay<'a, T>,
{
    fn lines(&'a self) -> Vec<Line<T>> {
        self.iter().flat_map(|g| g.lines()).collect::<Vec<_>>()
    }

    fn contains_point(&'a self, p: Point<T>) -> bool {
        self.iter().any(|g| g.contains_point(p))
    }
}

// Note: We add LinesIter bound to disambiguate from &[Coord] which implements CoordsIter.
// Coord doesn't implement LinesIter, so this prevents the coherence conflict.
impl<'a, 'l, T, G> private::UnconstrainedRequirementTrait<'a, T> for &[G]
where
    'a: 'l,
    T: SpadeTriangulationFloat + 'a,
    G: TriangulateDelaunayUnconstrained<'a, T> + LinesIter<'l, Scalar = T>,
{
    fn coords(&'a self) -> private::CoordsIter<'a, T> {
        Box::new(self.iter().flat_map(|g| g.coords()))
    }
}

impl<'a, 'l, T, G> private::ConstrainedRequirementTrait<'a, T> for &[G]
where
    'a: 'l,
    T: SpadeTriangulationFloat + 'a,
    G: TriangulateDelaunay<'a, T> + LinesIter<'l, Scalar = T>,
{
    fn lines(&'a self) -> Vec<Line<T>> {
        self.iter().flat_map(|g| g.lines()).collect::<Vec<_>>()
    }

    fn contains_point(&'a self, p: Point<T>) -> bool {
        self.iter().any(|g| g.contains_point(p))
    }
}

// Note: Point collections like MultiPoint, Point, and &[Coord] get
// TriangulateDelaunayUnconstrained automatically via the blanket impl for CoordsIter.
// They only support unconstrained triangulation because points don't have edges
// or an interior: constrained triangulation requires boundaries to constrain against.

// ========== Triangulation trait impl helpers ============

/// Find all intersections in `lines` via a sweep-line pass and split every
/// segment at its intersection points.  Newly created vertices are snapped
/// into `known_points` using `snap_radius`.
///
/// This replaces the previous iterative find-one-split-loop approach that
/// was O(k * n^2) (an all-pairs scan per intersection, up to 1000 times).
/// The sweep finds all intersections in O((n + m) log n), then splits
/// every segment in a single pass, where:
///
/// - n = number of input line segments
/// - m = number of x-overlapping segment pairs (candidates checked by the
///   sweep; m >= k but typically m ~ O(n) for polygon edges)
/// - k = number of actual intersections found
///
/// A single pass suffices because splitting at all intersection points
/// guarantees the resulting sub-segments share only endpoints, never
/// interior crossings.
fn split_segments_at_intersections<T: SpadeTriangulationFloat>(
    lines: Vec<Line<T>>,
    mut known_points: RTree<Coord<T>>,
    snap_radius: T,
) -> Result<Vec<Line<T>>, TriangulationError> {
    use crate::algorithm::sweep::Intersections;

    if lines.len() < 2 {
        return Ok(lines);
    }

    // Phase 1: Find all intersections via sweep
    let indexed: Vec<SweepIndexedLine<T>> = lines
        .iter()
        .enumerate()
        .map(|(i, l)| SweepIndexedLine { index: i, line: *l })
        .collect();

    // Collect intersections lazily: defer the per-segment split-point
    // storage until the first proper intersection is found.  For the
    // common case (valid polygon edges that share only endpoints after
    // snapping), the sweep finds no proper intersections and we skip
    // Phase 2 and 3 entirely -- returning the input unchanged without
    // allocating or iterating over n segments.
    let mut split_points: Option<Vec<Vec<Coord<T>>>> = None;
    for (seg_a, seg_b, intersection) in Intersections::from_iter(indexed) {
        match intersection {
            LineIntersection::SinglePoint {
                intersection: pt,
                is_proper,
            } => {
                if is_proper {
                    let sp = split_points.get_or_insert_with(|| vec![Vec::new(); lines.len()]);
                    let pt = snap_or_register_point(pt, &mut known_points, snap_radius);
                    sp[seg_a.index].push(pt);
                    sp[seg_b.index].push(pt);
                }
            }
            LineIntersection::Collinear {
                intersection: overlap,
            } => {
                if overlap.start != overlap.end {
                    let sp = split_points.get_or_insert_with(|| vec![Vec::new(); lines.len()]);
                    let s = snap_or_register_point(overlap.start, &mut known_points, snap_radius);
                    let e = snap_or_register_point(overlap.end, &mut known_points, snap_radius);
                    for idx in [seg_a.index, seg_b.index] {
                        sp[idx].push(s);
                        sp[idx].push(e);
                    }
                }
            }
        }
    }

    // Fast path: no proper intersections found.  The input segments
    // already share only endpoints (the common case after preprocess_lines
    // has snapped and deduped), so return them unchanged.
    let Some(mut split_points) = split_points else {
        return Ok(lines);
    };

    // Phase 2: Split each segment at its collected intersection points
    let extra: usize = split_points.iter().map(|v| v.len()).sum();
    let mut result = Vec::with_capacity(lines.len() + extra);
    for (i, line) in lines.iter().enumerate() {
        let pts = &mut split_points[i];
        pts.retain(|p| *p != line.start && *p != line.end);

        if pts.is_empty() {
            result.push(*line);
            continue;
        }

        // Sort split points along the segment direction using dot-product
        // projection.  This correctly handles diagonal segments where sorting
        // by x or y alone would be ambiguous.
        let dir = line.end - line.start;
        pts.sort_by(|a, b| {
            let ta = (*a - line.start).dot_product(dir);
            let tb = (*b - line.start).dot_product(dir);
            ta.total_cmp(&tb)
        });
        pts.dedup();

        // Create sub-segments between consecutive split points
        let mut prev = line.start;
        for &pt in pts.iter() {
            if pt != prev {
                result.push(Line::new(prev, pt));
            }
            prev = pt;
        }
        if prev != line.end {
            result.push(Line::new(prev, line.end));
        }
    }

    // Phase 3: Dedup (normalize direction then sort + dedup)
    result.retain(|l| l.start != l.end);
    for line in result.iter_mut() {
        *line = sweep_normalize_line(*line);
    }
    result.sort_by(sweep_line_ord);
    result.dedup();

    Ok(result)
}

/// A line segment tagged with its original index, for the sweep iterator.
#[derive(Clone, Debug)]
struct SweepIndexedLine<T: GeoFloat> {
    index: usize,
    line: Line<T>,
}

impl<T: GeoFloat> crate::algorithm::sweep::Cross for SweepIndexedLine<T> {
    type Scalar = T;
    fn line(&self) -> Line<T> {
        self.line
    }
}

/// Canonical direction for a line: lexicographically smaller endpoint first.
fn sweep_normalize_line<T: GeoFloat>(line: Line<T>) -> Line<T> {
    let cmp = line
        .start
        .x
        .total_cmp(&line.end.x)
        .then_with(|| line.start.y.total_cmp(&line.end.y));
    if cmp == std::cmp::Ordering::Greater {
        Line::new(line.end, line.start)
    } else {
        line
    }
}

/// Lexicographic comparison of two lines by (start.x, start.y, end.x, end.y).
fn sweep_line_ord<T: GeoFloat>(a: &Line<T>, b: &Line<T>) -> std::cmp::Ordering {
    a.start
        .x
        .total_cmp(&b.start.x)
        .then_with(|| a.start.y.total_cmp(&b.start.y))
        .then_with(|| a.end.x.total_cmp(&b.end.x))
        .then_with(|| a.end.y.total_cmp(&b.end.y))
}

/// Snap point to the nearest existing point if it's close enough.
///
/// Uses an R-tree for O(log n) nearest-neighbour lookup instead of O(n) linear scan,
/// which is critical for performance with large point sets.
fn snap_or_register_point<T: SpadeTriangulationFloat>(
    point: Coord<T>,
    known_points: &mut RTree<Coord<T>>,
    snap_radius: T,
) -> Coord<T> {
    known_points
        .nearest_neighbor(&point)
        // only snap if closest is within snap radius
        .filter(|nearest_point| Euclidean.distance(**nearest_point, point) < snap_radius)
        .cloned()
        // otherwise register and use input point
        .unwrap_or_else(|| {
            known_points.insert(point);
            point
        })
}

/// Snap endpoints and deduplicate lines before intersection splitting.
///
/// This function interleaves two concerns in a specific order:
///
/// 1. **Snap** every endpoint to the nearest known coordinate within
///    `snap_radius`, using an R-tree for O(log n) nearest-neighbour
///    lookup.  Snapping must happen first because it can make two
///    previously distinct coordinates equal, which affects dedup.
///    The R-tree is built incrementally so that later lines snap to
///    points introduced by earlier ones.
///
/// 2. **Deduplicate** lines that are identical after snapping.  Both
///    forward (A->B) and reverse (B->A) are considered duplicates.
///    This is done by normalising each line so the lexicographically
///    smaller endpoint comes first, then sorting and deduplicating
///    in O(n log n).  The previous implementation used `Vec::contains`
///    which was O(n) per insertion (O(n^2) total).
///
/// Degenerate lines (start == end after snapping) are also removed.
fn preprocess_lines<T: SpadeTriangulationFloat>(
    lines: Vec<Line<T>>,
    snap_radius: T,
) -> (RTree<Coord<T>>, Vec<Line<T>>) {
    // Pass 1: snap all endpoints via R-tree (O(n log n))
    let mut known_coords: RTree<Coord<T>> = RTree::new();
    let mut snapped: Vec<Line<T>> = lines
        .into_iter()
        .map(|mut line| {
            line.start = snap_or_register_point(line.start, &mut known_coords, snap_radius);
            line.end = snap_or_register_point(line.end, &mut known_coords, snap_radius);
            line
        })
        .filter(|l| l.start != l.end)
        .collect();

    // Pass 2: normalise direction, sort, dedup (O(n log n))
    for line in snapped.iter_mut() {
        *line = sweep_normalize_line(*line);
    }
    snapped.sort_by(sweep_line_ord);
    snapped.dedup();

    (known_coords, snapped)
}

#[cfg(test)]
mod spade_triangulation {
    use super::*;
    use geo_types::*;

    fn assert_num_triangles<T: SpadeTriangulationFloat>(
        triangulation: &TriangulationResult<Triangles<T>>,
        num: usize,
    ) {
        assert_eq!(
            triangulation
                .as_ref()
                .map(|tris| tris.len())
                .expect("triangulation success"),
            num
        )
    }

    #[test]
    fn basic_triangle_triangulates() {
        let triangulation = Triangle::new(
            Coord { x: 0.0, y: 0.0 },
            Coord { x: 1.0, y: 0.0 },
            Coord { x: 0.0, y: 1.0 },
        )
        .unconstrained_triangulation();

        assert_num_triangles(&triangulation, 1);
    }

    #[test]
    fn basic_rectangle_triangulates() {
        let triangulation = Rect::new(Coord { x: 0.0, y: 0.0 }, Coord { x: 1.0, y: 1.0 })
            .unconstrained_triangulation();

        assert_num_triangles(&triangulation, 2);
    }

    #[test]
    fn basic_polygon_triangulates() {
        let triangulation = Polygon::new(
            LineString::new(vec![
                Coord { x: 0.0, y: 1.0 },
                Coord { x: -1.0, y: 0.0 },
                Coord { x: -0.5, y: -1.0 },
                Coord { x: 0.5, y: -1.0 },
                Coord { x: 1.0, y: 0.0 },
            ]),
            vec![],
        )
        .unconstrained_triangulation();

        assert_num_triangles(&triangulation, 3);
    }

    #[test]
    fn overlapping_triangles_triangulate_unconstrained() {
        let triangles = vec![
            Triangle::new(
                Coord { x: 0.0, y: 0.0 },
                Coord { x: 2.0, y: 0.0 },
                Coord { x: 0.0, y: 2.0 },
            ),
            Triangle::new(
                Coord { x: 1.0, y: 1.0 },
                Coord { x: -1.0, y: 1.0 },
                Coord { x: 1.0, y: -1.0 },
            ),
        ];

        let unconstrained_triangulation = triangles.unconstrained_triangulation();
        assert_num_triangles(&unconstrained_triangulation, 4);
    }

    #[test]
    fn overlapping_triangles_triangulate_constrained_outer() {
        let triangles = vec![
            Triangle::new(
                Coord { x: 0.0, y: 0.0 },
                Coord { x: 2.0, y: 0.0 },
                Coord { x: 0.0, y: 2.0 },
            ),
            Triangle::new(
                Coord { x: 1.0, y: 1.0 },
                Coord { x: -1.0, y: 1.0 },
                Coord { x: 1.0, y: -1.0 },
            ),
        ];

        let constrained_outer_triangulation =
            triangles.constrained_outer_triangulation(Default::default());
        assert_num_triangles(&constrained_outer_triangulation, 8);
    }

    #[test]
    fn overlapping_triangles_triangulate_constrained() {
        let triangles = vec![
            Triangle::new(
                Coord { x: 0.0, y: 0.0 },
                Coord { x: 2.0, y: 0.0 },
                Coord { x: 0.0, y: 2.0 },
            ),
            Triangle::new(
                Coord { x: 1.0, y: 1.0 },
                Coord { x: -1.0, y: 1.0 },
                Coord { x: 1.0, y: -1.0 },
            ),
        ];

        let constrained_outer_triangulation =
            triangles.constrained_triangulation(Default::default());
        assert_num_triangles(&constrained_outer_triangulation, 6);
    }

    #[test]
    fn u_shaped_polygon_triangulates_unconstrained() {
        let u_shape = Polygon::new(
            LineString::new(vec![
                Coord { x: 0.0, y: 0.0 },
                Coord { x: 1.0, y: 0.0 },
                Coord { x: 1.0, y: 1.0 },
                Coord { x: 2.0, y: 1.0 },
                Coord { x: 2.0, y: 0.0 },
                Coord { x: 3.0, y: 0.0 },
                Coord { x: 3.0, y: 3.0 },
                Coord { x: 0.0, y: 3.0 },
            ]),
            vec![],
        );

        let unconstrained_triangulation = u_shape.unconstrained_triangulation();
        assert_num_triangles(&unconstrained_triangulation, 8);
    }

    #[test]
    fn u_shaped_polygon_triangulates_constrained_outer() {
        let u_shape = Polygon::new(
            LineString::new(vec![
                Coord { x: 0.0, y: 0.0 },
                Coord { x: 1.0, y: 0.0 },
                Coord { x: 1.0, y: 1.0 },
                Coord { x: 2.0, y: 1.0 },
                Coord { x: 2.0, y: 0.0 },
                Coord { x: 3.0, y: 0.0 },
                Coord { x: 3.0, y: 3.0 },
                Coord { x: 0.0, y: 3.0 },
            ]),
            vec![],
        );

        let constrained_outer_triangulation =
            u_shape.constrained_outer_triangulation(Default::default());
        assert_num_triangles(&constrained_outer_triangulation, 8);
    }

    #[test]
    fn u_shaped_polygon_triangulates_constrained_inner() {
        let u_shape = Polygon::new(
            LineString::new(vec![
                Coord { x: 0.0, y: 0.0 },
                Coord { x: 1.0, y: 0.0 },
                Coord { x: 1.0, y: 1.0 },
                Coord { x: 2.0, y: 1.0 },
                Coord { x: 2.0, y: 0.0 },
                Coord { x: 3.0, y: 0.0 },
                Coord { x: 3.0, y: 3.0 },
                Coord { x: 0.0, y: 3.0 },
            ]),
            vec![],
        );

        let constrained_triangulation = u_shape.constrained_triangulation(Default::default());
        assert_num_triangles(&constrained_triangulation, 6);
    }

    #[test]
    fn various_snap_radius_works() {
        let u_shape = Polygon::new(
            LineString::new(vec![
                Coord { x: 0.0, y: 0.0 },
                Coord { x: 1.0, y: 0.0 },
                Coord { x: 1.0, y: 1.0 },
                Coord { x: 2.0, y: 1.0 },
                Coord { x: 2.0, y: 0.0 },
                Coord { x: 3.0, y: 0.0 },
                Coord { x: 3.0, y: 3.0 },
                Coord { x: 0.0, y: 3.0 },
            ]),
            vec![],
        );

        for snap_with in (1..6).map(|pow| 0.1_f64.powi(pow)) {
            let constrained_triangulation =
                u_shape.constrained_triangulation(DelaunayTriangulationConfig {
                    snap_radius: snap_with,
                });
            assert_num_triangles(&constrained_triangulation, 6);
        }
    }

    #[test]
    fn multipoint_triangulates_unconstrained() {
        use super::TriangulateDelaunayUnconstrained;

        let points: MultiPoint<f64> = wkt!(MULTIPOINT(0. 0., 1. 0., 0.5 1., 0.5 0.5));

        let triangulation = points.unconstrained_triangulation();
        // 4 points with one inside the triangle formed by the others produce 3 triangles
        assert_num_triangles(&triangulation, 3);
    }

    #[test]
    fn point_triangulates_unconstrained() {
        use super::TriangulateDelaunayUnconstrained;

        // Single point produces no triangles (need at least 3 points)
        let point: Point<f64> = wkt!(POINT(0. 0.));
        let triangulation = point.unconstrained_triangulation();
        assert_num_triangles(&triangulation, 0);
    }

    #[test]
    fn coord_slice_triangulates_unconstrained() {
        use super::TriangulateDelaunayUnconstrained;

        let coords: &[Coord<f64>] = &[
            Coord { x: 0.0, y: 0.0 },
            Coord { x: 1.0, y: 0.0 },
            Coord { x: 0.5, y: 1.0 },
        ];
        let triangulation = coords.unconstrained_triangulation();
        assert_num_triangles(&triangulation, 1);
    }

    #[test]
    fn polygon_slice_triangulates() {
        use super::{TriangulateDelaunay, TriangulateDelaunayUnconstrained};

        let polygons: Vec<Polygon<f64>> = vec![
            wkt!(POLYGON((0. 0., 1. 0., 0.5 1., 0. 0.))),
            wkt!(POLYGON((2. 0., 3. 0., 2.5 1., 2. 0.))),
        ];
        let slice: &[Polygon<f64>] = &polygons;

        // Slice of polygons should support unconstrained triangulation
        // 6 points from 2 triangular polygons -> 4 triangles in convex hull
        let triangulation = slice.unconstrained_triangulation();
        assert_num_triangles(&triangulation, 4);

        // Slice of polygons should support constrained triangulation
        // Respects polygon boundaries -> 2 triangles (one per polygon)
        let constrained = slice.constrained_triangulation(Default::default());
        assert_num_triangles(&constrained, 2);
    }

    // ====== Tests: prepare_intersection_constraint (sweep-based) ======

    #[test]
    fn prepare_intersection_no_crossings() {
        let lines = vec![
            Line::new(Coord { x: 0.0, y: 0.0 }, Coord { x: 10.0, y: 0.0 }),
            Line::new(Coord { x: 0.0, y: 5.0 }, Coord { x: 10.0, y: 5.0 }),
        ];
        let result = split_segments_at_intersections(lines, RTree::new(), 0.0001).unwrap();
        assert_eq!(result.len(), 2);
    }

    #[test]
    fn prepare_intersection_single_crossing() {
        // X pattern: two lines crossing at (5, 5)
        let lines = vec![
            Line::new(Coord { x: 0.0, y: 0.0 }, Coord { x: 10.0, y: 10.0 }),
            Line::new(Coord { x: 0.0, y: 10.0 }, Coord { x: 10.0, y: 0.0 }),
        ];
        let result = split_segments_at_intersections(lines, RTree::new(), 0.0001).unwrap();
        assert_eq!(result.len(), 4, "two crossing lines -> 4 sub-segments");
    }

    #[test]
    fn prepare_intersection_multiple_splits() {
        // Horizontal line crossed by two vertical lines
        let lines = vec![
            Line::new(Coord { x: 0.0, y: 5.0 }, Coord { x: 10.0, y: 5.0 }),
            Line::new(Coord { x: 3.0, y: 0.0 }, Coord { x: 3.0, y: 10.0 }),
            Line::new(Coord { x: 7.0, y: 0.0 }, Coord { x: 7.0, y: 10.0 }),
        ];
        let result = split_segments_at_intersections(lines, RTree::new(), 0.0001).unwrap();
        // Horizontal: 3 parts, each vertical: 2 parts = 7
        assert_eq!(result.len(), 7);
    }

    #[test]
    fn prepare_intersection_diagonal_sort() {
        // Diagonal crossed by two horizontals -- tests parametric sorting
        let lines = vec![
            Line::new(Coord { x: 0.0, y: 0.0 }, Coord { x: 10.0, y: 10.0 }),
            Line::new(Coord { x: 0.0, y: 3.0 }, Coord { x: 10.0, y: 3.0 }),
            Line::new(Coord { x: 0.0, y: 7.0 }, Coord { x: 10.0, y: 7.0 }),
        ];
        let result = split_segments_at_intersections(lines, RTree::new(), 0.0001).unwrap();
        assert_eq!(result.len(), 7);
        for line in &result {
            assert_ne!(line.start, line.end, "sub-segment should not be degenerate");
        }
    }

    #[test]
    fn prepare_intersection_endpoint_touch() {
        // Two lines sharing an endpoint should not be split
        let lines = vec![
            Line::new(Coord { x: 0.0, y: 0.0 }, Coord { x: 5.0, y: 5.0 }),
            Line::new(Coord { x: 5.0, y: 5.0 }, Coord { x: 10.0, y: 0.0 }),
        ];
        let result = split_segments_at_intersections(lines, RTree::new(), 0.0001).unwrap();
        assert_eq!(
            result.len(),
            2,
            "shared endpoint should not cause splitting"
        );
    }
}
