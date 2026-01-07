use geo_types::{Coord, Line, Point, Triangle};
use spade::{
    ConstrainedDelaunayTriangulation, DelaunayTriangulation, Point2, SpadeNum, Triangulation,
};

use crate::{Centroid, Contains};
use crate::{
    CoordsIter, Distance, Euclidean, GeoFloat, LineIntersection, LinesIter,
    line_intersection::line_intersection,
};

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

pub trait SpadeTriangulationFloat: GeoFloat + SpadeNum {}
impl<T: GeoFloat + SpadeNum> SpadeTriangulationFloat for T {}

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
            prepare_intersection_contraint(lines, known_coords, snap_radius)
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
        let points = self.coords();
        points
            .into_iter()
            .map(to_spade_point)
            .try_fold(DelaunayTriangulation::<Point2<T>>::new(), |mut tris, p| {
                tris.insert(p).map_err(TriangulationError::SpadeError)?;
                Ok(tris)
            })
            .map(triangulation_to_triangles)
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
            .map(to_spade_line)
            .try_fold(
                ConstrainedDelaunayTriangulation::<Point2<T>>::new(),
                |mut cdt, [start, end]| {
                    let start = cdt.insert(start).map_err(TriangulationError::SpadeError)?;
                    let end = cdt.insert(end).map_err(TriangulationError::SpadeError)?;
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
    T: Triangulation<Vertex = Point2<F>>,
    F: SpadeTriangulationFloat,
{
    triangulation
        .inner_faces()
        .map(|face| face.positions())
        .map(|points| points.map(|p| Coord::<F> { x: p.x, y: p.y }))
        .map(Triangle::from)
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

fn prepare_intersection_contraint<T: SpadeTriangulationFloat>(
    mut lines: Vec<Line<T>>,
    mut known_points: Vec<Coord<T>>,
    snap_radius: T,
) -> Result<Vec<Line<T>>, TriangulationError> {
    // Rule 2 of "Power of 10" rules (NASA)
    // safety net. We can't prove that the `while let` loop isn't going to run infinitely, so
    // we abort after a fixed amount of iterations. In case that the iteration seems to loop
    // indefinitely this check will return an Error indicating the infinite loop.
    let mut loop_count = 1000;
    let mut loop_check = || {
        loop_count -= 1;
        (loop_count != 0)
            .then_some(())
            .ok_or(TriangulationError::LoopTrap)
    };

    while let Some((indices, intersection)) = {
        let mut iter = iter_line_pairs(&lines);
        iter.find_map(find_intersecting_lines_fn)
    } {
        loop_check()?;
        let [l0, l1] = remove_lines_by_index(indices, &mut lines);
        let new_lines = split_lines([l0, l1], intersection);
        let new_lines = cleanup_filter_lines(new_lines, &lines, &mut known_points, snap_radius);

        lines.extend(new_lines);
    }

    Ok(lines)
}

/// iterates over all combinations (a,b) of lines in a vector where a != b
fn iter_line_pairs<T: SpadeTriangulationFloat>(
    lines: &[Line<T>],
) -> impl Iterator<Item = [(usize, &Line<T>); 2]> {
    lines.iter().enumerate().flat_map(|(idx0, line0)| {
        lines
            .iter()
            .enumerate()
            .skip(idx0 + 1)
            .filter(move |(idx1, line1)| *idx1 != idx0 && line0 != *line1)
            .map(move |(idx1, line1)| [(idx0, line0), (idx1, line1)])
    })
}

/// checks whether two lines are intersecting and if so, checks the intersection to not be ill
/// formed
///
/// returns
/// - [usize;2] : sorted indexes of lines, smaller one comes first
/// - intersection : type of intersection
fn find_intersecting_lines_fn<T: SpadeTriangulationFloat>(
    [(idx0, line0), (idx1, line1)]: [(usize, &Line<T>); 2],
) -> Option<([usize; 2], LineIntersection<T>)> {
    line_intersection(*line0, *line1)
        .filter(|intersection| {
            match intersection {
                // intersection is not located in both lines
                LineIntersection::SinglePoint { is_proper, .. } if !is_proper => false,
                // collinear intersection is length zero line
                LineIntersection::Collinear { intersection }
                    if intersection.start == intersection.end =>
                {
                    false
                }
                _ => true,
            }
        })
        .map(|intersection| ([idx0, idx1], intersection))
}

/// removes two lines by index in a safe way since the second index can be invalidated after
/// the first line was removed (remember `.remove(idx)` returns the element and shifts the tail
/// of the vector in direction of its start to close the gap)
fn remove_lines_by_index<T: SpadeTriangulationFloat>(
    mut indices: [usize; 2],
    lines: &mut Vec<Line<T>>,
) -> [Line<T>; 2] {
    indices.sort();
    let [idx0, idx1] = indices;
    let l1 = lines.remove(idx1);
    let l0 = lines.remove(idx0);
    [l0, l1]
}

/// split lines based on intersection kind:
///
/// - intersection point: create 4 new lines from the existing line's endpoints to the intersection
///   point
/// - collinear: create 3 new lines (before overlap, overlap, after overlap)
fn split_lines<T: SpadeTriangulationFloat>(
    [l0, l1]: [Line<T>; 2],
    intersection: LineIntersection<T>,
) -> Vec<Line<T>> {
    match intersection {
        LineIntersection::SinglePoint { intersection, .. } => [
            (l0.start, intersection),
            (l0.end, intersection),
            (l1.start, intersection),
            (l1.end, intersection),
        ]
        .map(|(a, b)| Line::new(a, b))
        .to_vec(),
        LineIntersection::Collinear { .. } => {
            let mut points = [l0.start, l0.end, l1.start, l1.end];
            // sort points by their coordinate values to resolve ambiguities
            points.sort_by(|a, b| {
                a.x.partial_cmp(&b.x)
                    .expect("sorting points by coordinate x failed")
                    .then_with(|| {
                        a.y.partial_cmp(&b.y)
                            .expect("sorting points by coordinate y failed")
                    })
            });
            // since all points are on one line we can just create new lines from consecutive
            // points after sorting
            points
                .windows(2)
                .map(|win| Line::new(win[0], win[1]))
                .collect::<Vec<_>>()
        }
    }
}

/// new lines from the `split_lines` function may contain a variety of ill formed lines, this
/// function cleans all of these cases up
fn cleanup_filter_lines<T: SpadeTriangulationFloat>(
    lines_need_check: Vec<Line<T>>,
    existing_lines: &[Line<T>],
    known_points: &mut Vec<Coord<T>>,
    snap_radius: T,
) -> Vec<Line<T>> {
    lines_need_check
        .into_iter()
        .map(|mut line| {
            line.start = snap_or_register_point(line.start, known_points, snap_radius);
            line.end = snap_or_register_point(line.end, known_points, snap_radius);
            line
        })
        .filter(|l| l.start != l.end)
        .filter(|l| !existing_lines.contains(l))
        .filter(|l| !existing_lines.contains(&Line::new(l.end, l.start)))
        .collect::<Vec<_>>()
}

/// snap point to the nearest existing point if it's close enough
///
/// snap_radius can be configured via the third parameter of this function
fn snap_or_register_point<T: SpadeTriangulationFloat>(
    point: Coord<T>,
    known_points: &mut Vec<Coord<T>>,
    snap_radius: T,
) -> Coord<T> {
    known_points
        .iter()
        // find closest
        .min_by(|a, b| {
            Euclidean
                .distance(**a, point)
                .partial_cmp(&Euclidean.distance(**b, point))
                .expect("Couldn't compare coordinate distances")
        })
        // only snap if closest is within epsilon range
        .filter(|nearest_point| Euclidean.distance(**nearest_point, point) < snap_radius)
        .cloned()
        // otherwise register and use input point
        .unwrap_or_else(|| {
            known_points.push(point);
            point
        })
}

/// preprocesses lines so that we're less likely to hit issues when using the spade triangulation
fn preprocess_lines<T: SpadeTriangulationFloat>(
    lines: Vec<Line<T>>,
    snap_radius: T,
) -> (Vec<Coord<T>>, Vec<Line<T>>) {
    let mut known_coords: Vec<Coord<T>> = vec![];
    let capacity = lines.len();
    let lines = lines
        .into_iter()
        .fold(Vec::with_capacity(capacity), |mut lines, mut line| {
            // deduplicating:

            // 1. snap coords of lines to existing coords
            line.start = snap_or_register_point(line.start, &mut known_coords, snap_radius);
            line.end = snap_or_register_point(line.end, &mut known_coords, snap_radius);
            if
            // 2. make sure line isn't degenerate (no length when start == end)
            line.start != line.end
                // 3. make sure line or flipped line wasn't already added
                && !lines.contains(&line)
                && !lines.contains(&Line::new(line.end, line.start))
            {
                lines.push(line)
            }

            lines
        });
    (known_coords, lines)
}

/// converts Line to something somewhat similar in the spade world
fn to_spade_line<T: SpadeTriangulationFloat>(line: Line<T>) -> [Point2<T>; 2] {
    [to_spade_point(line.start), to_spade_point(line.end)]
}

/// converts Coord to something somewhat similar in the spade world
fn to_spade_point<T: SpadeTriangulationFloat>(coord: Coord<T>) -> Point2<T> {
    Point2::new(coord.x, coord.y)
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
}
