use crate::geometry::Coord;
use crate::GeoNum;
use i_overlay::core::fill_rule::FillRule;
use i_overlay::core::overlay::ShapeType;
use i_overlay::core::overlay_rule::OverlayRule;
use i_overlay::string::clip::ClipRule;

pub trait BoolOpsCoord<T>: Copy {
    fn new(x: T, y: T) -> Self;
    fn x(&self) -> T;
    fn y(&self) -> T;
}

/// A geometry coordinate number suitable for performing geometric boolean operations.
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

pub trait BoolOpsOverlay {
    type CoordType;
    type OverlayGraph: BoolOpsOverlayGraph<CoordType = Self::CoordType>;
    fn new() -> Self;
    fn add_path(&mut self, path: Vec<Self::CoordType>, shape_type: ShapeType);
    fn into_graph(self, fill_rule: FillRule) -> Self::OverlayGraph;
}

pub(super) trait BoolOpsOverlayGraph {
    type CoordType;
    fn extract_shapes(&self, overlay_rule: OverlayRule) -> Vec<Vec<Vec<Self::CoordType>>>;
}

pub trait BoolOpsStringOverlay {
    type CoordType;
    type StringGraph: BoolOpsStringGraph<CoordType = Self::CoordType>;
    fn new() -> Self;
    fn add_shape_path(&mut self, path: Vec<Self::CoordType>);
    fn add_string_line(&mut self, path: [Self::CoordType; 2]);
    fn into_graph(self, fill_rule: FillRule) -> Self::StringGraph;
}

pub(super) trait BoolOpsStringGraph {
    type CoordType;
    fn clip_string_lines(&self, clip_rule: ClipRule) -> Vec<Vec<Self::CoordType>>;
}

mod f64 {
    use super::{ClipRule, FillRule, OverlayRule, ShapeType};
    use i_overlay::f64::{
        graph::F64OverlayGraph,
        overlay::F64Overlay,
        string::{F64StringGraph, F64StringOverlay},
    };
    use i_overlay::i_float::f64_point::F64Point;

    impl super::BoolOpsNum for f64 {
        type CoordType = F64Point;
        type OverlayType = F64Overlay;
        type StringOverlayType = F64StringOverlay;
    }

    impl super::BoolOpsCoord<f64> for F64Point {
        #[inline]
        fn new(x: f64, y: f64) -> Self {
            Self::new(x, y)
        }

        #[inline]
        fn x(&self) -> f64 {
            self.x
        }

        #[inline]
        fn y(&self) -> f64 {
            self.y
        }
    }

    impl super::BoolOpsOverlay for F64Overlay {
        type CoordType = F64Point;
        type OverlayGraph = F64OverlayGraph;

        #[inline]
        fn new() -> Self {
            Self::new()
        }

        #[inline]
        fn add_path(&mut self, path: Vec<F64Point>, shape_type: ShapeType) {
            self.add_path(path, shape_type)
        }

        #[inline]
        fn into_graph(self, fill_rule: FillRule) -> Self::OverlayGraph {
            self.into_graph(fill_rule)
        }
    }

    impl super::BoolOpsOverlayGraph for F64OverlayGraph {
        type CoordType = F64Point;

        #[inline]
        fn extract_shapes(&self, overlay_rule: OverlayRule) -> Vec<Vec<Vec<F64Point>>> {
            self.extract_shapes(overlay_rule)
        }
    }

    impl super::BoolOpsStringOverlay for F64StringOverlay {
        type CoordType = F64Point;
        type StringGraph = F64StringGraph;

        #[inline]
        fn new() -> Self {
            Self::new()
        }

        #[inline]
        fn add_shape_path(&mut self, path: Vec<Self::CoordType>) {
            self.add_shape_path(path)
        }

        #[inline]
        fn add_string_line(&mut self, path: [Self::CoordType; 2]) {
            self.add_string_line(path)
        }

        #[inline]
        fn into_graph(self, fill_rule: FillRule) -> Self::StringGraph {
            self.into_graph(fill_rule)
        }
    }

    impl super::BoolOpsStringGraph for F64StringGraph {
        type CoordType = F64Point;

        #[inline]
        fn clip_string_lines(&self, clip_rule: ClipRule) -> Vec<Vec<Self::CoordType>> {
            self.clip_string_lines(clip_rule)
        }
    }
}

mod f32 {
    use i_overlay::core::fill_rule::FillRule;
    use i_overlay::core::overlay::ShapeType;
    use i_overlay::core::overlay_rule::OverlayRule;
    use i_overlay::f32::graph::F32OverlayGraph;
    use i_overlay::f32::overlay::F32Overlay;
    use i_overlay::f32::string::{F32StringGraph, F32StringOverlay};
    use i_overlay::i_float::f32_point::F32Point;
    use i_overlay::string::clip::ClipRule;

    impl super::BoolOpsNum for f32 {
        type CoordType = F32Point;
        type OverlayType = F32Overlay;
        type StringOverlayType = F32StringOverlay;
    }

    impl super::BoolOpsCoord<f32> for F32Point {
        #[inline]
        fn new(x: f32, y: f32) -> Self {
            Self::new(x, y)
        }
        #[inline]
        fn x(&self) -> f32 {
            self.x
        }
        #[inline]
        fn y(&self) -> f32 {
            self.y
        }
    }

    impl super::BoolOpsOverlay for F32Overlay {
        type CoordType = F32Point;
        type OverlayGraph = F32OverlayGraph;

        #[inline]
        fn new() -> Self {
            Self::new()
        }

        #[inline]
        fn add_path(&mut self, path: Vec<Self::CoordType>, shape_type: ShapeType) {
            self.add_path(path, shape_type)
        }

        #[inline]
        fn into_graph(self, fill_rule: FillRule) -> Self::OverlayGraph {
            self.into_graph(fill_rule)
        }
    }

    impl super::BoolOpsOverlayGraph for F32OverlayGraph {
        type CoordType = F32Point;

        #[inline]
        fn extract_shapes(&self, overlay_rule: OverlayRule) -> Vec<Vec<Vec<F32Point>>> {
            self.extract_shapes(overlay_rule)
        }
    }

    impl super::BoolOpsStringOverlay for F32StringOverlay {
        type CoordType = F32Point;
        type StringGraph = F32StringGraph;

        #[inline]
        fn new() -> Self {
            Self::new()
        }

        #[inline]
        fn add_shape_path(&mut self, path: Vec<Self::CoordType>) {
            self.add_shape_path(path)
        }

        #[inline]
        fn add_string_line(&mut self, path: [Self::CoordType; 2]) {
            self.add_string_line(path)
        }

        #[inline]
        fn into_graph(self, fill_rule: FillRule) -> Self::StringGraph {
            self.into_graph(fill_rule)
        }
    }

    impl super::BoolOpsStringGraph for F32StringGraph {
        type CoordType = F32Point;

        #[inline]
        fn clip_string_lines(&self, clip_rule: ClipRule) -> Vec<Vec<Self::CoordType>> {
            self.clip_string_lines(clip_rule)
        }
    }
}

pub(super) mod convert {
    use super::super::OpType;
    use super::{BoolOpsNum, OverlayRule};
    use crate::geometry::{LineString, MultiLineString, MultiPolygon, Polygon};

    pub fn line_string_from_path<T: BoolOpsNum>(path: Vec<T::CoordType>) -> LineString<T> {
        let coords = path.into_iter().map(T::to_geo_coord);
        LineString(coords.collect())
    }

    pub fn multi_line_string_from_paths<T: BoolOpsNum>(
        paths: Vec<Vec<T::CoordType>>,
    ) -> MultiLineString<T> {
        let line_strings = paths.into_iter().map(|p| line_string_from_path(p));
        MultiLineString(line_strings.collect())
    }

    pub fn polygon_from_shape<T: BoolOpsNum>(shape: Vec<Vec<T::CoordType>>) -> Polygon<T> {
        let mut rings = shape.into_iter().map(|p| line_string_from_path(p));
        let exterior = rings.next().unwrap_or(LineString::new(vec![]));
        Polygon::new(exterior, rings.collect())
    }

    pub fn multi_polygon_from_shapes<T: BoolOpsNum>(
        shapes: Vec<Vec<Vec<T::CoordType>>>,
    ) -> MultiPolygon<T> {
        let polygons = shapes.into_iter().map(|s| polygon_from_shape(s));
        MultiPolygon(polygons.collect())
    }

    pub fn ring_to_shape_path<T: BoolOpsNum>(line_string: &LineString<T>) -> Vec<T::CoordType> {
        if line_string.0.is_empty() {
            return vec![];
        }
        // In geo, Polygon rings are explicitly closed LineStrings — their final coordinate is the same as their first coordinate,
        // however in i_overlay, shape paths are implicitly closed, so we skip the last coordinate.
        let coords = &line_string.0[..line_string.0.len() - 1];
        coords.iter().copied().map(T::to_bops_coord).collect()
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
}

#[cfg(test)]
mod tests {
    use crate::algorithm::BooleanOps;
    use crate::geometry::{MultiPolygon, Polygon};
    use crate::wkt;

    #[test]
    fn two_empty_polygons() {
        let p1: Polygon = wkt!(POLYGON EMPTY);
        let p2 = wkt!(POLYGON EMPTY);
        assert_eq!(&p1.union(&p2), &wkt!(MULTIPOLYGON EMPTY));
        assert_eq!(&p1.intersection(&p2), &wkt!(MULTIPOLYGON EMPTY));
    }

    #[test]
    fn one_empty_polygon() {
        let p1: Polygon = wkt!(POLYGON((0. 0., 0. 1., 1. 1., 1. 0., 0. 0.)));
        let p2 = wkt!(POLYGON EMPTY);
        assert_eq!(&p1.union(&p2), &MultiPolygon(vec![p1.clone()]));
        assert_eq!(&p1.intersection(&p2), &wkt!(MULTIPOLYGON EMPTY));
    }
}
