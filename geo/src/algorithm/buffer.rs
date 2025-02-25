//! TODO DOCUMENT

use crate::algorithm::bool_ops::i_overlay_integration;
use crate::algorithm::orient::{Direction, Orient};
use crate::bool_ops::i_overlay_integration::convert::ring_to_shape_path;
use crate::bool_ops::BoolOpsNum;
use crate::geometry::{Geometry, MultiPolygon, Polygon};
use crate::BooleanOps;
use i_overlay::mesh::outline::offset::OutlineOffset;
use i_overlay::mesh::style::OutlineStyle;

// TODO: BooleanOps requirement is just to get `rings` - we could extract that, though in practice it
// doesn't affect any functionality of our own implementations.
/// Outer boundary contours are in clockwise order.
/// Holes are in counterclockwise order.
pub trait Buffer {
    type Scalar: BoolOpsNum + 'static;
    // TODO: output might be GeometryCollection?
    // TODO: For point, output will always be Polygon
    //  (what about negative?)
    // TODO: For Polygon, a negative distance could output a MultiPolygong.
    // TODO: Not sure about negative distance for LineString
    // TODO: Buffering MultiPolygon could ouput Polygon, but... maybe simplest to still just output MP
    fn buffer(&self, distance: Self::Scalar) -> MultiPolygon<Self::Scalar> {
        let default_style = OutlineStyle::new(distance);
        self.buffer_with_style(default_style)
    }
    fn buffer_with_style(&self, style: OutlineStyle<Self::Scalar>) -> MultiPolygon<Self::Scalar>;
}

impl<F: BoolOpsNum + 'static> Buffer for Geometry<F> {
    type Scalar = F;

    fn buffer_with_style(&self, style: OutlineStyle<Self::Scalar>) -> MultiPolygon<Self::Scalar> {
        match self {
            Geometry::Point(_) => todo!("Handle buffering Point"),
            Geometry::LineString(_) => todo!("Handle buffering LineString"),
            Geometry::Polygon(polygon) => polygon.buffer_with_style(style),
            Geometry::MultiPoint(_) => todo!("Handle buffering MultiPoint"),
            Geometry::MultiLineString(_) => todo!("Handle buffering MultiLineString"),
            Geometry::MultiPolygon(_) => todo!("Handle buffering MultiPolygon"),
            Geometry::GeometryCollection(_) => todo!("Handle buffering GeometryCollection"),
            Geometry::Line(_) => todo!("Handle buffering Line"),
            Geometry::Rect(_) => todo!("Handle buffering Rect"),
            Geometry::Triangle(_) => todo!("Handle buffering Triangle"),
        }
    }
}

impl<F: BoolOpsNum + 'static> Buffer for Polygon<F> {
    type Scalar = F;
    fn buffer_with_style(&self, style: OutlineStyle<Self::Scalar>) -> MultiPolygon<Self::Scalar> {
        let rewound = self.orient(Direction::Reversed);
        let subject = rewound.rings().map(ring_to_shape_path).collect::<Vec<_>>();
        let shapes = subject.outline(style);
        i_overlay_integration::convert::multi_polygon_from_shapes(shapes)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::wkt;

    #[test]
    fn buffer() {
        let polygon = wkt! { POLYGON((2.0 2.0,2.0 6.0,4.0 6.0)) };
        let buffered = polygon.buffer(0.5);
        let expected = wkt! { MULTIPOLYGON(((1.5 2.0, 2.4472135975956912 1.776393204927444,4.447213597595692 5.7763932049274445,4.0 6.5,2.0 6.5,1.5 6.0,1.5 2.0))) };
        assert_relative_eq!(buffered, expected, epsilon = 1.0e-6);
    }

    fn init_logging() {
        use std::sync::Once;
        static LOG_SETUP: Once = Once::new();
        LOG_SETUP.call_once(|| {
            pretty_env_logger::init();
        });
    }

    #[test]
    fn jts_tests() {
        init_logging();
        use jts_test_runner::assert_jts_tests_succeed;
        assert_jts_tests_succeed("*Buffer*.xml");
    }
}
