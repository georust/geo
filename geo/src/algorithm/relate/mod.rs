pub(crate) use edge_end_builder::EdgeEndBuilder;
pub use geomgraph::intersection_matrix::IntersectionMatrix;

use crate::{
    GeoFloat, Geometry, GeometryCollection, GeometryCow, Line, LineString, MultiLineString,
    MultiPoint, MultiPolygon, Point, Polygon, Rect, Triangle,
};

mod edge_end_builder;
mod geomgraph;
mod relate_operation;

/// Topologically relate two geometries based on [DE-9IM](https://en.wikipedia.org/wiki/DE-9IM) semantics.
///
/// See [`IntersectionMatrix`] for details.
///
/// # Examples
///
/// ```
/// use geo::{Coordinate, Line, Rect, line_string};
/// use crate::geo::relate::Relate;
///
/// let line = Line::new(Coordinate { x: 2.0, y: 2.0}, Coordinate { x: 4.0, y: 4.0 });
/// let rect = Rect::new(Coordinate { x: 2.0, y: 2.0}, Coordinate { x: 4.0, y: 4.0 });
/// let intersection_matrix = rect.relate(&line);
///
/// assert!(intersection_matrix.is_intersects());
/// assert!(!intersection_matrix.is_disjoint());
/// assert!(intersection_matrix.is_contains());
/// assert!(!intersection_matrix.is_within());
///
/// let line = Line::new(Coordinate { x: 1.0, y: 1.0}, Coordinate { x: 5.0, y: 5.0 });
/// let rect = Rect::new(Coordinate { x: 2.0, y: 2.0}, Coordinate { x: 4.0, y: 4.0 });
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
pub trait Relate<F, T> {
    fn relate(&self, other: &T) -> IntersectionMatrix;
}

impl<F: GeoFloat> Relate<F, GeometryCow<'_, F>> for GeometryCow<'_, F> {
    fn relate(&self, other: &GeometryCow<F>) -> IntersectionMatrix {
        let mut relate_computer = relate_operation::RelateOperation::new(self, other);
        relate_computer.compute_intersection_matrix()
    }
}

macro_rules! relate_impl {
    ($k:ty, $t:ty) => {
        relate_impl![($k, $t),];
    };
    ($(($k:ty, $t:ty),)*) => {
        $(
            impl<F: GeoFloat> Relate<F, $t> for $k {
                fn relate(&self, other: &$t) -> IntersectionMatrix {
                    GeometryCow::from(self).relate(&GeometryCow::from(other))
                }
            }
        )*
    };
}

/// Call the given macro with every pair of inputs
///
/// # Examples
///
/// ```no_run
/// cartesian_pairs!(foo, [Bar, Baz, Qux]);
/// ```
/// Is akin to calling:
/// ```no_run
/// foo![(Bar, Bar), (Bar, Baz), (Bar, Qux), (Baz, Bar), (Baz, Baz), (Baz, Qux), (Qux, Bar), (Qux, Baz), (Qux, Qux)]);
/// ```
macro_rules! cartesian_pairs {
    ($macro_name:ident, [$($a:ty),*]) => {
        cartesian_pairs_helper! { [] [$($a,)*] [$($a,)*] [$($a,)*] $macro_name}
    };
}

macro_rules! cartesian_pairs_helper {
    // popped all a's - we're done. Use the accumulated output as the input to relate macro.
    ([$($out_pairs:tt)*] [] [$($b:ty,)*] $init_b:tt $macro_name:ident) => {
        $macro_name!{$($out_pairs)*}
    };
    // finished one loop of b, pop next a and reset b
    ($out_pairs:tt [$a_car:ty, $($a_cdr:ty,)*] [] $init_b:tt $macro_name:ident) => {
        cartesian_pairs_helper!{$out_pairs [$($a_cdr,)*] $init_b $init_b $macro_name}
    };
    // pop b through all of b with head of a
    ([$($out_pairs:tt)*] [$a_car:ty, $($a_cdr:ty,)*] [$b_car:ty, $($b_cdr:ty,)*] $init_b:tt $macro_name:ident) => {
        cartesian_pairs_helper!{[$($out_pairs)* ($a_car, $b_car),] [$a_car, $($a_cdr,)*] [$($b_cdr,)*] $init_b $macro_name}
    };
}

// Implement Relate for every combination of Geometry. Alternatively we could do something like
// `impl Relate<Into<GeometryCow>> for Into<GeometryCow> { }`
// but I don't know that we want to make GeometryCow public (yet?).
cartesian_pairs!(relate_impl, [Point<F>, Line<F>, LineString<F>, Polygon<F>, MultiPoint<F>, MultiLineString<F>, MultiPolygon<F>, Rect<F>, Triangle<F>, GeometryCollection<F>]);
relate_impl!(Geometry<F>, Geometry<F>);
