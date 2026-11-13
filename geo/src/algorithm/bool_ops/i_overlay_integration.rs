use crate::geometry::{Coord, LineString, MultiLineString, MultiPolygon};
use crate::{GeoFloat, GeoNum};
use i_overlay::core::fill_rule::FillRule;
use i_overlay::core::overlay_rule::OverlayRule;
use i_overlay::i_float::float::compatible::FloatPointCompatible;
use i_overlay::i_float::float::number::FloatNumber;
use i_overlay::string::clip::ClipRule;
use num_traits::FromPrimitive;

/// A floating point geometry coordinate scalar suitable for performing geometric boolean
/// operations and buffering.
pub trait BoolOpsNum: BoolOpsScalar + GeoFloat + FloatNumber + FromPrimitive {}
impl<T: BoolOpsScalar + GeoFloat + FloatNumber + FromPrimitive> BoolOpsNum for T {}

/// A geometry coordinate scalar suitable for performing geometric [boolean operations].
///
/// Implemented for `f32` and `f64`, and for the integer types `i16`,
/// `i32` and `i64`. Integer operations are computed in integer arithmetic, so the vertices of
/// the input are kept exactly; new vertices, where edges cross, are rounded to the nearest
/// integer coordinate.
///
/// # Panics
///
/// Boolean operations on `i64` geometries panic if a coordinate is outside
/// `-2^62 < x, y < 2^62`.
///
/// This trait is sealed: it can't be implemented outside of geo.
///
/// [boolean operations]: crate::BooleanOps
pub trait BoolOpsScalar: GeoNum + private::Sealed {
    #[doc(hidden)]
    fn overlay(
        subject: &[&LineString<Self>],
        clip: &[&LineString<Self>],
        overlay_rule: OverlayRule,
        fill_rule: FillRule,
    ) -> MultiPolygon<Self>;

    #[doc(hidden)]
    fn clip_lines(
        lines: &MultiLineString<Self>,
        clip: &[&LineString<Self>],
        fill_rule: FillRule,
        clip_rule: ClipRule,
    ) -> MultiLineString<Self>;
}

mod private {
    pub trait Sealed {}
    impl Sealed for f32 {}
    impl Sealed for f64 {}
    impl Sealed for i16 {}
    impl Sealed for i32 {}
    impl Sealed for i64 {}
}

mod float {
    use super::BoolOpsCoord;
    use super::BoolOpsNum;
    use super::convert::{
        multi_line_string_from_paths, multi_polygon_from_shapes, ring_to_shape_path,
    };
    use crate::geometry::{LineString, MultiLineString, MultiPolygon};
    use i_overlay::core::fill_rule::FillRule;
    use i_overlay::core::overlay_rule::OverlayRule;
    use i_overlay::float::clip::FloatClip;
    use i_overlay::float::overlay::{FloatOverlay, OverlayOptions};
    use i_overlay::string::clip::ClipRule;

    pub(super) fn overlay<T: BoolOpsNum>(
        subject: &[&LineString<T>],
        clip: &[&LineString<T>],
        overlay_rule: OverlayRule,
        fill_rule: FillRule,
    ) -> MultiPolygon<T> {
        let subject = subject
            .iter()
            .map(|ring| ring_to_shape_path(ring))
            .collect::<Vec<_>>();
        let mut overlay = if clip.is_empty() {
            FloatOverlay::with_subj_custom(&subject, OverlayOptions::ogc(), Default::default())
        } else {
            let clip = clip
                .iter()
                .map(|ring| ring_to_shape_path(ring))
                .collect::<Vec<_>>();
            FloatOverlay::with_subj_and_clip_custom(
                &subject,
                &clip,
                OverlayOptions::ogc(),
                Default::default(),
            )
        };
        multi_polygon_from_shapes(overlay.overlay(overlay_rule, fill_rule))
    }

    pub(super) fn clip_lines<T: BoolOpsNum>(
        lines: &MultiLineString<T>,
        clip: &[&LineString<T>],
        fill_rule: FillRule,
        clip_rule: ClipRule,
    ) -> MultiLineString<T> {
        let subject: Vec<Vec<_>> = lines
            .iter()
            .map(|line_string| line_string.coords().map(|c| BoolOpsCoord(*c)).collect())
            .collect();
        let clip = clip
            .iter()
            .map(|ring| ring_to_shape_path(ring))
            .collect::<Vec<_>>();
        multi_line_string_from_paths(subject.clip_by(&clip, fill_rule, clip_rule))
    }
}

mod int {
    use crate::geometry::{Coord, LineString, MultiLineString, MultiPolygon, Polygon};
    use i_overlay::core::fill_rule::FillRule;
    use i_overlay::core::integer::OverlayInt;
    use i_overlay::core::overlay::{IntOverlayOptions, Overlay};
    use i_overlay::core::overlay_rule::OverlayRule;
    use i_overlay::i_float::int::point::IntPoint;
    use i_overlay::i_shape::int::path::IntPath;
    use i_overlay::i_shape::int::safe_range::IntSafeRange;
    use i_overlay::string::clip::{ClipRule, IntClip};
    use num_traits::NumCast;

    // The input coordinate type is widened to the overlay engine's integer type, and narrowed
    // back for the output. Output coordinates are within the inputs' bounds, so narrowing
    // can't fail.
    fn to_engine<T: NumCast, I: NumCast>(value: T) -> I {
        I::from(value).expect("engine integer type is at least as wide as the coordinate type")
    }

    fn from_engine<T: NumCast, I: NumCast>(value: I) -> T {
        T::from(value).expect("output coordinates are within the bounds of the input")
    }

    fn line_string_to_path<T, I>(line_string: &LineString<T>) -> IntPath<I>
    where
        T: crate::GeoNum,
        I: OverlayInt + NumCast,
    {
        line_string
            .coords()
            .map(|c| IntPoint::new(to_engine(c.x), to_engine(c.y)))
            .collect()
    }

    // geo rings repeat their first coordinate at the end; i_overlay contours are implicitly
    // closed, so drop it.
    fn ring_to_path<T, I>(ring: &LineString<T>) -> IntPath<I>
    where
        T: crate::GeoNum,
        I: OverlayInt + NumCast,
    {
        let mut path = line_string_to_path(ring);
        path.pop();
        path
    }

    fn path_to_line_string<T, I>(path: IntPath<I>) -> LineString<T>
    where
        T: crate::GeoNum,
        I: OverlayInt + NumCast,
    {
        path.into_iter()
            .map(|p| Coord {
                x: from_engine(p.x),
                y: from_engine(p.y),
            })
            .collect()
    }

    fn check_range<I: OverlayInt>(paths: &[IntPath<I>]) {
        assert!(
            paths.is_in_safe_range(),
            "boolean operation coordinates must be within -2^{bits} < x, y < 2^{bits}",
            bits = I::BITS - 2,
        );
    }

    pub(super) fn overlay<T, I>(
        subject: &[&LineString<T>],
        clip: &[&LineString<T>],
        overlay_rule: OverlayRule,
        fill_rule: FillRule,
    ) -> MultiPolygon<T>
    where
        T: crate::GeoNum,
        I: OverlayInt + NumCast,
    {
        let subject = subject
            .iter()
            .map(|ring| ring_to_path::<T, I>(ring))
            .collect::<Vec<_>>();
        let clip = clip
            .iter()
            .map(|ring| ring_to_path::<T, I>(ring))
            .collect::<Vec<_>>();
        check_range(&subject);
        check_range(&clip);
        let shapes = Overlay::from_subj_and_clip_custom(
            &subject,
            &clip,
            IntOverlayOptions::ogc(),
            Default::default(),
        )
        .overlay(overlay_rule, fill_rule);
        shapes
            .into_iter()
            .map(|shape| {
                let mut rings = shape.into_iter().map(|path| {
                    let mut ring = path_to_line_string(path);
                    ring.close();
                    ring
                });
                let exterior = rings.next().unwrap_or(LineString::empty());
                Polygon::new(exterior, rings.collect())
            })
            .collect()
    }

    pub(super) fn clip_lines<T, I>(
        lines: &MultiLineString<T>,
        clip: &[&LineString<T>],
        fill_rule: FillRule,
        clip_rule: ClipRule,
    ) -> MultiLineString<T>
    where
        T: crate::GeoNum,
        I: OverlayInt + NumCast,
    {
        let lines = lines
            .iter()
            .map(line_string_to_path::<T, I>)
            .collect::<Vec<_>>();
        let clip = clip
            .iter()
            .map(|ring| ring_to_path::<T, I>(ring))
            .collect::<Vec<_>>();
        check_range(&lines);
        check_range(&clip);
        let paths = clip.clip_paths(&lines, fill_rule, clip_rule);
        MultiLineString(paths.into_iter().map(path_to_line_string).collect())
    }
}

// `BoolOpsScalar` can't be implemented for every `BoolOpsNum` alongside the integer impls, as
// coherence assumes i_float may implement `FloatNumber` for integers in the future.
macro_rules! impl_bool_ops_scalar_for_float {
    ($t:ty) => {
        impl BoolOpsScalar for $t {
            fn overlay(
                subject: &[&LineString<$t>],
                clip: &[&LineString<$t>],
                overlay_rule: OverlayRule,
                fill_rule: FillRule,
            ) -> MultiPolygon<$t> {
                float::overlay(subject, clip, overlay_rule, fill_rule)
            }

            fn clip_lines(
                lines: &MultiLineString<$t>,
                clip: &[&LineString<$t>],
                fill_rule: FillRule,
                clip_rule: ClipRule,
            ) -> MultiLineString<$t> {
                float::clip_lines(lines, clip, fill_rule, clip_rule)
            }
        }
    };
}

impl_bool_ops_scalar_for_float!(f32);
impl_bool_ops_scalar_for_float!(f64);

// Each integer coordinate type uses an overlay engine wide enough that all of its values are in
// the engine's safe range, except `i64`, which has no wider engine and is range checked.
macro_rules! impl_bool_ops_scalar_for_int {
    ($t:ty, $engine:ty) => {
        impl BoolOpsScalar for $t {
            fn overlay(
                subject: &[&LineString<$t>],
                clip: &[&LineString<$t>],
                overlay_rule: OverlayRule,
                fill_rule: FillRule,
            ) -> MultiPolygon<$t> {
                int::overlay::<$t, $engine>(subject, clip, overlay_rule, fill_rule)
            }

            fn clip_lines(
                lines: &MultiLineString<$t>,
                clip: &[&LineString<$t>],
                fill_rule: FillRule,
                clip_rule: ClipRule,
            ) -> MultiLineString<$t> {
                int::clip_lines::<$t, $engine>(lines, clip, fill_rule, clip_rule)
            }
        }
    };
}

impl_bool_ops_scalar_for_int!(i16, i32);
impl_bool_ops_scalar_for_int!(i32, i64);
impl_bool_ops_scalar_for_int!(i64, i64);

/// New type for `Coord` that implements `FloatPointCompatible` for `BoolOpsNum` to
/// circumvent orphan rule, since Coord is defined in geo_types.
#[derive(Copy, Clone, Debug)]
pub struct BoolOpsCoord<T: BoolOpsNum>(pub(crate) Coord<T>);

impl<T: BoolOpsNum> FloatPointCompatible for BoolOpsCoord<T> {
    type Scalar = T;

    fn from_xy(x: T, y: T) -> Self {
        Self(Coord { x, y })
    }

    fn x(&self) -> T {
        self.0.x
    }

    fn y(&self) -> T {
        self.0.y
    }
}

pub(crate) mod convert {
    use super::super::OpType;
    use super::BoolOpsNum;
    use crate::bool_ops::i_overlay_integration::BoolOpsCoord;
    use crate::geometry::{LineString, MultiLineString, MultiPolygon, Polygon};
    use geo_types::Coord;
    use i_overlay::core::overlay_rule::OverlayRule;

    pub fn line_string_from_path<T: BoolOpsNum>(path: Vec<BoolOpsCoord<T>>) -> LineString<T> {
        let coords = path.into_iter().map(|bops_coord| bops_coord.0).collect();
        LineString(coords)
    }

    pub fn multi_line_string_from_paths<T: BoolOpsNum>(
        paths: Vec<Vec<BoolOpsCoord<T>>>,
    ) -> MultiLineString<T> {
        let line_strings = paths.into_iter().map(|p| line_string_from_path(p));
        MultiLineString(line_strings.collect())
    }

    pub fn polygon_from_shape<T: BoolOpsNum>(shape: Vec<Vec<BoolOpsCoord<T>>>) -> Polygon<T> {
        let mut rings = shape.into_iter().map(|path| {
            let mut line_string = line_string_from_path(path);
            line_string.close();
            line_string
        });
        let exterior = rings.next().unwrap_or(LineString::empty());

        Polygon::new(exterior, rings.collect())
    }

    pub fn multi_polygon_from_shapes<T: BoolOpsNum>(
        shapes: Vec<Vec<Vec<BoolOpsCoord<T>>>>,
    ) -> MultiPolygon<T> {
        let polygons = shapes.into_iter().map(|s| polygon_from_shape(s));
        MultiPolygon(polygons.collect())
    }

    pub fn ring_to_shape_path<T: BoolOpsNum>(line_string: &LineString<T>) -> Vec<BoolOpsCoord<T>> {
        if line_string.0.is_empty() {
            return vec![];
        }
        // In geo, Polygon rings are explicitly closed LineStrings — their final coordinate is the same as their first coordinate,
        // however in i_overlay, shape paths are implicitly closed, so we skip the last coordinate.
        let coords = &line_string.0[..line_string.0.len() - 1];
        coords.iter().copied().map(BoolOpsCoord).collect()
    }

    pub fn line_string_to_shape_path<T: BoolOpsNum>(
        line_string: &LineString<T>,
    ) -> Vec<BoolOpsCoord<T>> {
        line_string.coords().copied().map(BoolOpsCoord).collect()
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

    impl<T: BoolOpsNum> From<Coord<T>> for BoolOpsCoord<T> {
        fn from(value: Coord<T>) -> Self {
            BoolOpsCoord(value)
        }
    }
}

#[cfg(test)]
mod tests {
    use geo_types::polygon;

    use crate::algorithm::{BooleanOps, Relate, Winding};
    use crate::geometry::{MultiPolygon, Polygon};
    use crate::winding_order::WindingOrder;
    use crate::wkt;

    #[test]
    // see https://github.com/georust/geo/issues/1309
    fn test_winding_order() {
        let poly1 = polygon!((x: 0.0, y: 0.0), (x: 1.0, y: 0.0), (x: 1.0, y: 1.0));
        assert!(matches!(
            poly1.exterior().winding_order(),
            Some(WindingOrder::CounterClockwise)
        ));

        {
            let union = poly1.union(&polygon!());
            assert_eq!(union.0.len(), 1);

            let union = &union[0];
            assert!(matches!(
                union.exterior().winding_order(),
                Some(WindingOrder::CounterClockwise)
            ));
        }
        {
            let intersection = poly1.intersection(&poly1);
            assert_eq!(intersection.0.len(), 1);

            let intersection = &intersection[0];
            assert!(matches!(
                intersection.exterior().winding_order(),
                Some(WindingOrder::CounterClockwise)
            ));
        }

        let poly2 = polygon!((x: 0.0, y: 0.0), (x: 1.0, y: 1.0), (x: 0.0, y: 1.0));
        assert!(matches!(
            poly2.exterior().winding_order(),
            Some(WindingOrder::CounterClockwise)
        ));

        {
            let union = poly1.union(&poly2);
            assert_eq!(union.0.len(), 1);

            let union = &union[0];
            assert!(union.interiors().is_empty());
            assert!(matches!(
                union.exterior().winding_order(),
                Some(WindingOrder::CounterClockwise)
            ));
        }
    }

    #[test]
    fn two_empty_polygons() {
        let p1: Polygon = wkt!(POLYGON EMPTY);
        let p2: Polygon = wkt!(POLYGON EMPTY);
        assert_eq!(&p1.union(&p2), &wkt!(MULTIPOLYGON EMPTY));
        assert_eq!(&p1.intersection(&p2), &wkt!(MULTIPOLYGON EMPTY));
    }

    #[test]
    fn one_empty_polygon() {
        let p1: Polygon = wkt!(POLYGON((0.0 0.0,1.0 0.0,1.0 1.0,0.0 1.0,0.0 0.0)));
        let p2: Polygon = wkt!(POLYGON EMPTY);
        {
            let unioned = p1.union(&p2);
            let expected = MultiPolygon(vec![p1.clone()]);
            let im = unioned.relate(&expected);
            assert!(im.is_equal_topo());
        }
        assert_eq!(&p1.intersection(&p2), &wkt!(MULTIPOLYGON EMPTY));
    }
}
