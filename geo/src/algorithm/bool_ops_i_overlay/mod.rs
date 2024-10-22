#[cfg(test)]
mod tests;

use i_overlay::core::fill_rule::FillRule;
use i_overlay::core::overlay::ShapeType;
use i_overlay::core::overlay_rule::OverlayRule;
use i_overlay::f32::graph::F32OverlayGraph;
use i_overlay::f32::overlay::F32Overlay;
use i_overlay::f32::string::{F32StringGraph, F32StringOverlay};
use i_overlay::f64::graph::F64OverlayGraph;
use i_overlay::f64::overlay::F64Overlay;
use i_overlay::f64::string::{F64StringGraph, F64StringOverlay};
use i_overlay::i_float::f32_point::F32Point;
use i_overlay::i_float::f64_point::F64Point;
use i_overlay::string::clip::ClipRule;

use geo_types::{Coord, LineString, MultiLineString, MultiPolygon};

use crate::{GeoNum, Polygon};

/// Boolean Operations on geometry.
///
/// Boolean operations are set operations on geometries considered as a subset
/// of the 2-D plane. The operations supported are: intersection, union, xor or
/// symmetric difference, and set-difference on pairs of 2-D geometries and
/// clipping a 1-D geometry with self.
///
/// These operations are implemented on [`Polygon`] and the [`MultiPolygon`]
/// geometries.
///
/// # Validity
///
/// Note that the operations are strictly well-defined only on *valid*
/// geometries. However, the implementation generally works well as long as the
/// interiors of polygons are contained within their corresponding exteriors.
///
/// Degenerate 2-d geoms with 0 area are handled, and ignored by the algorithm.
/// In particular, taking `union` with an empty geom should remove degeneracies
/// and fix invalid polygons as long the interior-exterior requirement above is
/// satisfied.
pub trait BooleanOps {
    type Scalar: BoolOpsNum;

    fn rings(&self) -> impl Iterator<Item = &LineString<Self::Scalar>>;

    fn boolean_op(&self, other: &impl BooleanOps<Scalar = Self::Scalar>, op: OpType) -> MultiPolygon<Self::Scalar> {
        let mut overlay = <Self::Scalar as BoolOpsNum>::OverlayType::new();

        for ring in self.rings() {
            overlay.add_path(ring_to_path(ring), ShapeType::Subject);
        }
        for ring in other.rings() {
            overlay.add_path(ring_to_path(ring), ShapeType::Clip);
        }

        let graph = overlay.into_graph(FillRule::EvenOdd);
        let shapes = graph.extract_shapes(op.into());

        multi_polygon_from_shapes(shapes)
    }

    fn intersection(&self, other:  &impl BooleanOps<Scalar = Self::Scalar>) -> MultiPolygon<Self::Scalar> {
        self.boolean_op(other, OpType::Intersection)
    }
    fn union(&self, other:  &impl BooleanOps<Scalar = Self::Scalar>) -> MultiPolygon<Self::Scalar> {
        self.boolean_op(other, OpType::Union)
    }
    fn xor(&self, other: &impl BooleanOps<Scalar = Self::Scalar>) -> MultiPolygon<Self::Scalar> {
        self.boolean_op(other, OpType::Xor)
    }
    fn difference(&self, other: &impl BooleanOps<Scalar = Self::Scalar>) -> MultiPolygon<Self::Scalar> {
        self.boolean_op(other, OpType::Difference)
    }

    /// Clip a 1-D geometry with self.
    ///
    /// Returns the portion of `ls` that lies within `self` (known as the set-theoeretic
    /// intersection) if `invert` is false, and the difference (`ls - self`) otherwise.
    fn clip(
        &self,
        multi_line_string: &MultiLineString<Self::Scalar>,
        invert: bool,
    ) -> MultiLineString<Self::Scalar>;
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum OpType {
    Intersection,
    Union,
    Difference,
    Xor,
}

trait Paths {
    type CoordType;
    fn paths(&self) -> impl Iterator<Item = Vec<Self::CoordType>>;
}

pub trait BoolOpsNum: GeoNum {
    type CoordType: BoolOpsCoord<Self>;
    type OverlayType: BoolOpsOverlay<CoordType = Self::CoordType>;
    type StringOverlayType: BoolOpsStringOverlay<CoordType = Self::CoordType>;

    fn to_bops_coord(geo_coord: Coord<Self>) -> Self::CoordType {
        Self::CoordType::new(geo_coord.x, geo_coord.y)
    }

    fn to_geo_coord(bops_coord: Self::CoordType) -> Coord<Self> {
        Coord {
            x: bops_coord.x(),
            y: bops_coord.y(),
        }
    }
}

trait BoolOpsCoord<T>: Copy {
    fn new(x: T, y: T) -> Self;
    fn x(&self) -> T;
    fn y(&self) -> T;
}

trait BoolOpsStringOverlay {
    type CoordType;
    type StringGraph: BoolOpsStringGraph<CoordType = Self::CoordType>;
    fn new() -> Self;
    fn add_shape_path(&mut self, path: Vec<Self::CoordType>);
    fn add_string_line(&mut self, path: [Self::CoordType; 2]);
    fn into_graph(self, fill_rule: FillRule) -> Self::StringGraph;
}

trait BoolOpsStringGraph {
    type CoordType;
    fn clip_string_lines(&self, clip_rule: ClipRule) -> Vec<Vec<Self::CoordType>>;
}

trait BoolOpsOverlay {
    type CoordType;
    type OverlayGraph: BoolOpsOverlayGraph<CoordType = Self::CoordType>;
    fn new() -> Self;
    fn add_path(&mut self, path: Vec<Self::CoordType>, shape_type: ShapeType);
    fn into_graph(self, fill_rule: FillRule) -> Self::OverlayGraph;
}

trait BoolOpsOverlayGraph {
    type CoordType;
    fn extract_shapes(&self, overlay_rule: OverlayRule) -> Vec<Vec<Vec<Self::CoordType>>>;
}

impl BoolOpsNum for f64 {
    type CoordType = F64Point;
    type OverlayType = F64Overlay;
    type StringOverlayType = F64StringOverlay;
}

impl BoolOpsStringOverlay for F64StringOverlay {
    type CoordType = F64Point;
    type StringGraph = F64StringGraph;

    fn new() -> Self {
        Self::new()
    }

    fn add_shape_path(&mut self, path: Vec<Self::CoordType>) {
        self.add_shape_path(path)
    }

    fn add_string_line(&mut self, path: [Self::CoordType; 2]) {
        self.add_string_line(path)
    }

    fn into_graph(self, fill_rule: FillRule) -> Self::StringGraph {
        self.into_graph(fill_rule)
    }
}

impl BoolOpsStringGraph for F64StringGraph {
    type CoordType = F64Point;

    fn clip_string_lines(&self, clip_rule: ClipRule) -> Vec<Vec<Self::CoordType>> {
        self.clip_string_lines(clip_rule)
    }
}

impl BoolOpsOverlay for F64Overlay {
    type CoordType = F64Point;
    type OverlayGraph = F64OverlayGraph;

    fn new() -> Self {
        Self::new()
    }

    fn add_path(&mut self, path: Vec<F64Point>, shape_type: ShapeType) {
        self.add_path(path, shape_type)
    }

    fn into_graph(self, fill_rule: FillRule) -> Self::OverlayGraph {
        self.into_graph(fill_rule)
    }
}

impl BoolOpsOverlayGraph for F64OverlayGraph {
    type CoordType = F64Point;

    fn extract_shapes(&self, overlay_rule: OverlayRule) -> Vec<Vec<Vec<F64Point>>> {
        self.extract_shapes(overlay_rule)
    }
}

impl BoolOpsCoord<f64> for F64Point {
    fn new(x: f64, y: f64) -> Self {
        Self::new(x, y)
    }

    fn x(&self) -> f64 {
        self.x
    }

    fn y(&self) -> f64 {
        self.y
    }
}

impl BoolOpsNum for f32 {
    type CoordType = F32Point;
    type OverlayType = F32Overlay;
    type StringOverlayType = F32StringOverlay;
}

impl BoolOpsStringOverlay for F32StringOverlay {
    type CoordType = F32Point;
    type StringGraph = F32StringGraph;

    fn new() -> Self {
        Self::new()
    }

    fn add_shape_path(&mut self, path: Vec<Self::CoordType>) {
        self.add_shape_path(path)
    }

    fn add_string_line(&mut self, path: [Self::CoordType; 2]) {
        self.add_string_line(path)
    }

    fn into_graph(self, fill_rule: FillRule) -> Self::StringGraph {
        self.into_graph(fill_rule)
    }
}

impl BoolOpsStringGraph for F32StringGraph {
    type CoordType = F32Point;

    fn clip_string_lines(&self, clip_rule: ClipRule) -> Vec<Vec<Self::CoordType>> {
        self.clip_string_lines(clip_rule)
    }
}

impl BoolOpsOverlay for F32Overlay {
    type CoordType = F32Point;
    type OverlayGraph = F32OverlayGraph;

    fn new() -> Self {
        Self::new()
    }

    fn add_path(&mut self, path: Vec<Self::CoordType>, shape_type: ShapeType) {
        self.add_path(path, shape_type)
    }

    fn into_graph(self, fill_rule: FillRule) -> Self::OverlayGraph {
        self.into_graph(fill_rule)
    }
}

impl BoolOpsOverlayGraph for F32OverlayGraph {
    type CoordType = F32Point;

    fn extract_shapes(&self, overlay_rule: OverlayRule) -> Vec<Vec<Vec<F32Point>>> {
        self.extract_shapes(overlay_rule)
    }
}

impl BoolOpsCoord<f32> for F32Point {
    fn new(x: f32, y: f32) -> Self {
        Self::new(x, y)
    }
    fn x(&self) -> f32 {
        self.x
    }
    fn y(&self) -> f32 {
        self.y
    }
}
// TODO impl for other types we support in geonum. Maybe just implement on GeoNum?

impl<T: BoolOpsNum> Paths for Polygon<T> {
    type CoordType = T::CoordType;
    fn paths(&self) -> impl Iterator<Item = Vec<Self::CoordType>> {
        std::iter::once(self.exterior())
            .chain(self.interiors().iter())
            .map(|r| {
                r.into_iter()
                    .map(|c| T::CoordType::new(c.x, c.y))
                    .collect::<Vec<_>>()
            })
    }
}

impl<T: BoolOpsNum> Paths for MultiPolygon<T> {
    type CoordType = T::CoordType;
    fn paths(&self) -> impl Iterator<Item = Vec<Self::CoordType>> {
        self.0.iter().flat_map(|p| p.paths())
    }
}

fn line_string_from_path<T: BoolOpsNum>(path: Vec<T::CoordType>) -> LineString<T> {
    let coords = path.into_iter().map(T::to_geo_coord).collect::<Vec<_>>();

    LineString(coords)
}

fn multi_line_string_from_paths<T: BoolOpsNum>(
    paths: Vec<Vec<T::CoordType>>,
) -> MultiLineString<T> {
    let line_strings: Vec<_> = paths
        .into_iter()
        .map(|p| line_string_from_path(p))
        .collect();

    MultiLineString(line_strings)
}

fn polygon_from_shape<T: BoolOpsNum>(shape: Vec<Vec<T::CoordType>>) -> Polygon<T> {
    let rings: Vec<_> = shape
        .into_iter()
        .map(|p| line_string_from_path(p))
        .collect();

    // TODO: avoid OOB panic, avoid clone
    let exterior = rings[0].clone();
    let interiors = rings[1..].to_vec();
    Polygon::new(exterior, interiors)
}

fn multi_polygon_from_shapes<T: BoolOpsNum>(
    shapes: Vec<Vec<Vec<T::CoordType>>>,
) -> MultiPolygon<T> {
    MultiPolygon(shapes.into_iter().map(|s| polygon_from_shape(s)).collect())
}

fn ring_to_path<T: BoolOpsNum>(line_string: &LineString<T>) -> Vec<T::CoordType> {
    line_string.coords().map(|c| T::to_bops_coord(*c)).collect()
}

impl From<OpType> for OverlayRule {
    fn from(op: OpType) -> Self {
        match op {
            OpType::Intersection => OverlayRule::Intersect,
            OpType::Union => OverlayRule::Union,
            OpType::Difference => OverlayRule::Difference,
            OpType::Xor => OverlayRule::Xor,
        }
    }
}

impl<T: BoolOpsNum> BooleanOps for Polygon<T> {
    type Scalar = T;

    fn rings(&self) -> impl Iterator<Item = &LineString<Self::Scalar>> {
        std::iter::once(self.exterior()).chain(self.interiors().iter())
    }

    fn clip(
        &self,
        multi_line_string: &MultiLineString<Self::Scalar>,
        invert: bool,
    ) -> MultiLineString<Self::Scalar> {
        let mut overlay = T::StringOverlayType::new();
        for path in self.paths() {
            overlay.add_shape_path(path)
        }
        for line_string in multi_line_string {
            for line in line_string.lines() {
                let line = [T::to_bops_coord(line.start), T::to_bops_coord(line.end)];
                overlay.add_string_line(line)
            }
        }

        let graph = overlay.into_graph(FillRule::EvenOdd);
        let paths = graph.clip_string_lines(ClipRule {
            invert,
            boundary_included: true,
        });
        multi_line_string_from_paths(paths)
    }
}

impl<T: BoolOpsNum> BooleanOps for MultiPolygon<T> {
    type Scalar = T;

    fn rings(&self) -> impl Iterator<Item = &LineString<Self::Scalar>> {
        self.0.iter().flat_map(|p| p.rings())
    }

    fn clip(
        &self,
        multi_line_string: &MultiLineString<Self::Scalar>,
        invert: bool,
    ) -> MultiLineString<Self::Scalar> {
        let mut overlay = T::StringOverlayType::new();
        for path in self.paths() {
            overlay.add_shape_path(path)
        }
        for line_string in multi_line_string {
            for line in line_string.lines() {
                let line = [T::to_bops_coord(line.start), T::to_bops_coord(line.end)];
                overlay.add_string_line(line)
            }
        }

        let graph = overlay.into_graph(FillRule::EvenOdd);
        let paths = graph.clip_string_lines(ClipRule {
            invert,
            boundary_included: true,
        });
        multi_line_string_from_paths(paths)
    }
}
