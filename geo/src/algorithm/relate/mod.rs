pub(crate) use edge_end_builder::EdgeEndBuilder;
pub use geomgraph::intersection_matrix::IntersectionMatrix;
use relate_operation::RelateOperation;

use crate::geometry::*;
#[deprecated(
    since = "0.31.1",
    note = "PreparedGeometry has moved to geo::indexed::PreparedGeometry"
)]
pub use crate::indexed::PreparedGeometry;
pub use crate::relate::geomgraph::GeometryGraph;
use crate::{BoundingRect, GeoFloat, GeometryCow, HasDimensions};

mod edge_end_builder;
pub(crate) mod geomgraph;
mod relate_operation;

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
    /// # Params
    ///
    /// `idx`: 0 or 1, designating A or B (respectively) in the role this geometry plays
    ///        in the relation. e.g. in `a.relate(b)`
    fn geometry_graph(&self, idx: usize) -> GeometryGraph<'_, F>;

    fn relate(&self, other: &impl Relate<F>) -> IntersectionMatrix
    where
        Self: Sized,
    {
        RelateOperation::new(self, other).compute_intersection_matrix()
    }
}

macro_rules! relate_impl {
    ($($t:ty ,)*) => {
        $(
            impl<F: GeoFloat> Relate<F> for $t {
                fn geometry_graph(&self, arg_index: usize) -> GeometryGraph<'_, F> {
                    $crate::relate::GeometryGraph::new(arg_index, GeometryCow::from(self))
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
    use super::Relate;
    use crate::{Rect, Triangle, coord, wkt};

    #[test]
    fn run_jts_relate_tests() {
        jts_test_runner::assert_jts_tests_succeed("*Relate*.xml");
    }

    // https://github.com/georust/geo/issues/1611
    #[test]
    fn degenerate_rect_relates_as_a_line() {
        let square = Rect::new(coord! { x: 0.0, y: 0.0 }, coord! { x: 1.0, y: 1.0 });
        let flat = Rect::new(coord! { x: 0.0, y: 0.25 }, coord! { x: 1.0, y: 0.25 });
        let line_string = wkt!(LINESTRING(0.0 0.25,1.0 0.25));
        assert_eq!(square.relate(&flat), square.relate(&line_string));
        assert!(square.relate(&flat).is_contains());
    }

    // https://github.com/georust/geo/issues/1611
    #[test]
    fn degenerate_rect_relates_as_a_point() {
        let origin = wkt!(POINT(0.0 0.0));
        let dot = Rect::new(coord! { x: 0.0, y: 0.0 }, coord! { x: 0.0, y: 0.0 });
        assert_eq!(origin.relate(&dot), origin.relate(&origin));
        assert!(origin.relate(&dot).is_contains());
    }

    #[test]
    fn degenerate_triangle_relates_as_a_line() {
        let square = Rect::new(coord! { x: 0.0, y: 0.0 }, coord! { x: 1.0, y: 1.0 });
        // The middle vertex lies between the other two, so the point set is one segment.
        let flat = Triangle::new(
            coord! { x: 0.0, y: 0.25 },
            coord! { x: 1.0, y: 0.25 },
            coord! { x: 0.5, y: 0.25 },
        );
        let line_string = wkt!(LINESTRING(0.0 0.25,1.0 0.25));
        assert_eq!(square.relate(&flat), square.relate(&line_string));
    }

    #[test]
    fn degenerate_triangle_relates_as_a_point() {
        let origin = wkt!(POINT(0.0 0.0));
        let dot = Triangle::new(
            coord! { x: 0.0, y: 0.0 },
            coord! { x: 0.0, y: 0.0 },
            coord! { x: 0.0, y: 0.0 },
        );
        assert_eq!(origin.relate(&dot), origin.relate(&origin));
    }
}
