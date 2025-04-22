use crate::geometry::Coord;
use crate::GeoNum;
use i_overlay::i_float::float::compatible::FloatPointCompatible;
use i_overlay::i_float::float::number::FloatNumber;

/// A geometry coordinate scalar suitable for performing geometric boolean operations.
pub trait BoolOpsNum: GeoNum + FloatNumber {}
impl<T: GeoNum + FloatNumber> BoolOpsNum for T {}

/// New type for `Coord` that implements `FloatPointCompatible` for `BoolOpsNum` to
/// circumvent orphan rule, since Coord is defined in geo_types.
#[derive(Copy, Clone, Debug)]
pub struct BoolOpsCoord<T: BoolOpsNum>(pub(crate) Coord<T>);

impl<T: BoolOpsNum> FloatPointCompatible<T> for BoolOpsCoord<T> {
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

pub(super) mod convert {
    use super::super::OpType;
    use super::BoolOpsNum;
    use crate::bool_ops::i_overlay_integration::BoolOpsCoord;
    use crate::geometry::{LineString, MultiLineString, MultiPolygon, Polygon};
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
            // From i_overlay: > Note: Outer boundary paths have a clockwise order, and holes have a counterclockwise order.
            // Which is the opposite convention we use.
            let mut line_string = line_string_from_path(path);
            line_string.close();
            line_string.0.reverse();
            line_string
        });
        let exterior = rings.next().unwrap_or(LineString::new(vec![]));

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
    use geo_types::polygon;

    use crate::algorithm::BooleanOps;
    use crate::geometry::{MultiPolygon, Polygon};
    use crate::winding_order::WindingOrder;
    use crate::{wkt, Winding};

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

            let union = &union.0[0];
            assert!(matches!(
                union.exterior().winding_order(),
                Some(WindingOrder::CounterClockwise)
            ));
        }
        {
            let intersection = poly1.intersection(&poly1);
            assert_eq!(intersection.0.len(), 1);

            let intersection = &intersection.0[0];
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

            let union = &union.0[0];
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
        assert_eq!(&p1.union(&p2), &MultiPolygon(vec![p1.clone()]));
        assert_eq!(&p1.intersection(&p2), &wkt!(MULTIPOLYGON EMPTY));
    }
}
