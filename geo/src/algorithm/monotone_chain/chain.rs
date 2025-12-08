use super::MonotoneChainSegment;
use super::segment::MonotoneChainSegmentFactory;
use crate::{BoundingRect, CoordNum, GeoNum, LineString, Rect};

/// A [`MonotoneChain`] is a primitive which represents a collection of segments,
/// each of which is a monotonic sequence of [`crate::Coord<T>`]
///
/// This primitive is not be constructed directly, it is created when building a geometry backed by [`MonotoneChain`]
#[derive(Debug, Clone)]
pub struct MonotoneChain<'a, T: CoordNum> {
    bounding_rect: Option<Rect<T>>,
    segments: Vec<MonotoneChainSegment<'a, T>>,
}

impl<'a, T: GeoNum> MonotoneChain<'a, T> {
    /// Iterate over the segments of the chain
    pub fn segment_iter(&self) -> impl Iterator<Item = &MonotoneChainSegment<'a, T>> {
        self.segments.iter()
    }
}

impl<'a, T: GeoNum> From<&'a LineString<T>> for MonotoneChain<'a, T> {
    fn from(linestring: &'a LineString<T>) -> Self {
        // each segment is a series of coordinates which are montonically increasing/decreasing
        Self {
            bounding_rect: linestring.bounding_rect(),
            segments: MonotoneChainSegmentFactory::new(&linestring.0).collect(),
        }
    }
}

impl<'a, T: GeoNum> BoundingRect<T> for MonotoneChain<'a, T> {
    type Output = Option<Rect<T>>;

    fn bounding_rect(&self) -> Self::Output {
        self.bounding_rect
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::LineString;
    use crate::algorithm::bounding_rect::BoundingRect;
    use crate::{Convert, coord, wkt};

    #[test]
    fn test_empty_line_string() {
        let linestring: LineString = LineString::empty();
        let chain: MonotoneChain<f64> = (&linestring).into();
        assert_eq!(chain.bounding_rect(), None);
    }

    #[test]
    fn test_construct_edge_case() {
        let linestring: LineString<f64> = wkt! { LINESTRING(0 0) }.convert();
        let chain: MonotoneChain<f64> = (&linestring).into();
        assert!(chain.segments.len() == 1);

        let linestring: LineString<f64> = wkt! { LINESTRING(0 0,1 1) }.convert();
        let chain: MonotoneChain<f64> = (&linestring).into();
        assert!(chain.segments.len() == 1);
    }

    #[test]
    fn test_vertical_horizontal() {
        let h = LineString::from_iter((0..10).map(|x| coord! { x: x as f64, y: 0. }));
        let h_r = LineString::from_iter((0..10).rev().map(|x| coord! { x: x as f64, y: 0. }));
        let v = LineString::from_iter((0..10).map(|y| coord! { x: 0., y: y as f64 }));
        let v_r = LineString::from_iter((0..10).rev().map(|y| coord! { x: 0., y: y as f64 }));

        let h_chain: MonotoneChain<f64> = (&h).into();
        let h_rev_chain: MonotoneChain<f64> = (&h_r).into();
        let v_chain: MonotoneChain<f64> = (&v).into();
        let v_rev_chain: MonotoneChain<f64> = (&v_r).into();

        assert!(h_chain.segments.len() == 1);
        assert!(h_rev_chain.segments.len() == 1);
        assert!(v_chain.segments.len() == 1);
        assert!(v_rev_chain.segments.len() == 1);
    }

    #[test]
    fn test_duplicates() {
        // duplicated points should not break the chain
        let a: LineString<f64> = wkt! { LINESTRING(0 0, 0 1, 0 1,0 1, 1 1) }.convert();
        let a_chain: MonotoneChain<f64> = (&a).into();
        assert!(a_chain.segments.len() == 1);
    }
}
