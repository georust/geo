pub(crate) use edge_end_builder::EdgeEndBuilder;
pub use geomgraph::intersection_matrix::IntersectionMatrix;

use crate::geometry::*;
#[deprecated(
    since = "0.31.1",
    note = "PreparedGeometry has moved to geo::indexed::PreparedGeometry"
)]
pub use crate::indexed::PreparedGeometry;
#[deprecated(
    since = "0.34.0",
    note = "part of the legacy graph-based relate engine, superseded by RelateNG; will be removed in a future release"
)]
pub use crate::relate::geomgraph::GeometryGraph;
use crate::{BoundingRect, GeoFloat, GeometryCow, HasDimensions};

// The legacy graph-based engine is kept in-tree (still reachable through
// `Relate::geometry_graph` and `PreparedGeometry`) until a future breaking
// release removes it; `relate` itself now evaluates through `relateng`.
#[allow(dead_code)]
mod edge_end_builder;
pub(crate) mod geomgraph;
#[allow(dead_code, deprecated)]
mod relate_operation;
pub(crate) mod relateng;

/// Topologically relate two geometries based on [DE-9IM](https://en.wikipedia.org/wiki/DE-9IM) semantics.
///
/// See [`IntersectionMatrix`] for details. All predicates are available on the calculated matrix.
///
/// # Examples
///
/// ```
/// use geo::{coord, Line, Rect, line_string};
/// use crate::geo::relate::Relate;
///
/// let line = Line::new(coord! { x: 2.0, y: 2.0}, coord! { x: 4.0, y: 4.0 });
/// let rect = Rect::new(coord! { x: 2.0, y: 2.0}, coord! { x: 4.0, y: 4.0 });
/// let intersection_matrix = rect.relate(&line);
///
/// assert!(intersection_matrix.is_intersects());
/// assert!(!intersection_matrix.is_disjoint());
/// assert!(intersection_matrix.is_contains());
/// assert!(!intersection_matrix.is_within());
///
/// let line = Line::new(coord! { x: 1.0, y: 1.0}, coord! { x: 5.0, y: 5.0 });
/// let rect = Rect::new(coord! { x: 2.0, y: 2.0}, coord! { x: 4.0, y: 4.0 });
/// let intersection_matrix = rect.relate(&line);
/// assert!(intersection_matrix.is_intersects());
/// assert!(!intersection_matrix.is_disjoint());
/// assert!(!intersection_matrix.is_contains());
/// assert!(!intersection_matrix.is_within());
///
/// let rect_boundary = line_string![
///     (x: 2.0, y: 2.0),
///     (x: 4.0, y: 2.0),
///     (x: 4.0, y: 4.0),
///     (x: 2.0, y: 4.0),
///     (x: 2.0, y: 2.0)
/// ];
/// let intersection_matrix = rect.relate(&rect_boundary);
/// assert!(intersection_matrix.is_intersects());
/// assert!(!intersection_matrix.is_disjoint());
/// // According to DE-9IM, polygons don't contain their own boundary
/// assert!(!intersection_matrix.is_contains());
/// assert!(!intersection_matrix.is_within());
/// ```
///
/// Note: `Relate` must not be called on geometries containing `NaN` coordinates.
pub trait Relate<F: GeoFloat>: BoundingRect<F> + HasDimensions {
    /// Returns a noded topology graph for the geometry.
    ///
    /// Used by the legacy graph-based relate engine; `relate` itself no
    /// longer calls this.
    ///
    /// # Params
    ///
    /// `idx`: 0 or 1, designating A or B (respectively) in the role this geometry plays
    ///        in the relation. e.g. in `a.relate(b)`
    #[deprecated(
        since = "0.34.0",
        note = "part of the legacy graph-based relate engine, superseded by RelateNG; will be removed in a future release"
    )]
    #[allow(deprecated)]
    fn geometry_graph(&self, idx: usize) -> GeometryGraph<'_, F> {
        GeometryGraph::new(idx, self.geometry_cow())
    }

    /// Returns a borrowed view of the geometry for relate evaluation.
    fn geometry_cow(&self) -> GeometryCow<'_, F>;

    fn relate(&self, other: &impl Relate<F>) -> IntersectionMatrix
    where
        Self: Sized,
    {
        relateng::relate_ng::relate(&self.geometry_cow(), &other.geometry_cow())
    }
}

macro_rules! relate_impl {
    ($($t:ty ,)*) => {
        $(
            impl<F: GeoFloat> Relate<F> for $t {
                fn geometry_cow(&self) -> GeometryCow<'_, F> {
                    GeometryCow::from(self)
                }
            }
            impl<F: GeoFloat> From<$t> for PreparedGeometry<'static, $t, F> {
                fn from(geometry: $t) -> Self {
                    $crate::indexed::prepared_geometry::prepare_geometry(geometry)
                }
            }
            impl<'a, F: GeoFloat> From<&'a $t> for PreparedGeometry<'a, &'a $t, F> {
                fn from(geometry: &'a $t) -> Self {
                    $crate::indexed::prepared_geometry::prepare_geometry(geometry)
                }
            }
        )*
    };
}

relate_impl![
    Point<F>,
    Line<F>,
    LineString<F>,
    Polygon<F>,
    MultiPoint<F>,
    MultiLineString<F>,
    MultiPolygon<F>,
    Rect<F>,
    Triangle<F>,
    GeometryCollection<F>,
    Geometry<F>,
];

#[cfg(test)]
mod tests {
    #[test]
    fn run_jts_relate_tests() {
        jts_test_runner::assert_jts_tests_succeed("*Relate*.xml");
    }
}
