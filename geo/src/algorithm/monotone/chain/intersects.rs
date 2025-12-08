use super::{MonotoneChain, MonotoneChainSegment};
use crate::GeoNum;
use crate::geometry::*;
use crate::intersects::has_disjoint_bboxes;
use crate::{BoundingRect, Intersects};

// `Geom` intersects `MonotoneChain` and `MonotoneChainSegment` requires adding a lifetime to the macro
// duplicated from `intersects/mod.rs` but with additional lifetime

macro_rules! symmetric_intersects_impl {
    ($t:ty, $k:ty) => {
        impl<'a, T> $crate::Intersects<$k> for $t
        where
            $k: $crate::Intersects<$t>,
            T: GeoNum,
        {
            fn intersects(&self, rhs: &$k) -> bool {
                rhs.intersects(self)
            }
        }
    };
}

// -----------------------------------------------------------------------------
// Chain Implementation
// -----------------------------------------------------------------------------

impl<'a, G, T: GeoNum> Intersects<G> for MonotoneChain<'a, T>
where
    G: BoundingRect<T>,
    // G: Intersects<MonotoneChainSegment<'a, T>>,
    MonotoneChainSegment<'a, T>: Intersects<G>,
{
    fn intersects(&self, rhs: &G) -> bool {
        if has_disjoint_bboxes(self, rhs) {
            return false;
        }

        self.segment_iter().any(|seg| seg.intersects(rhs))
    }
}

// commented out if they are implemented by blanket impl in main `Intersects` trait
symmetric_intersects_impl!(Coord<T>, MonotoneChain<'a, T>);
// symmetric_intersects_impl!(Point<T>,MonotoneChain<'a, T>);
// symmetric_intersects_impl!(MultiPoint<T>,MonotoneChain<'a, T>);

symmetric_intersects_impl!(Line<T>, MonotoneChain<'a, T>);
symmetric_intersects_impl!(LineString<T>, MonotoneChain<'a, T>);
// symmetric_intersects_impl!(MultiLineString<T>,MonotoneChain<'a, T>);

symmetric_intersects_impl!(Polygon<T>, MonotoneChain<'a, T>);
// symmetric_intersects_impl!(MultiPolygon<T>,MonotoneChain<'a, T>);
symmetric_intersects_impl!(Rect<T>, MonotoneChain<'a, T>);
symmetric_intersects_impl!(Triangle<T>, MonotoneChain<'a, T>);

// symmetric_intersects_impl!(Geometry<T>,MonotoneChain<'a, T>);
// symmetric_intersects_impl!(GeometryCollection<T>,MonotoneChain<'a, T>);

// -----------------------------------------------------------------------------
// Chain Segment Implementation
// -----------------------------------------------------------------------------

impl<'a, G, T: GeoNum> Intersects<G> for MonotoneChainSegment<'a, T>
where
    G: BoundingRect<T>,
    LineString<T>: Intersects<G>,
{
    fn intersects(&self, rhs: &G) -> bool {
        if has_disjoint_bboxes(self, rhs) {
            return false;
        }
        // TODO: tune the binary search
        // arbitrary but hard lower bound of 2 for algorithm correctness
        const SEARCH_THRESHOLD: usize = 2;

        if self.ls.len() > SEARCH_THRESHOLD {
            if let (l, Some(r)) = self.divide() {
                return l.intersects(rhs) || r.intersects(rhs);
            }
        }
        LineString::from_iter(self.ls.iter().cloned()).intersects(rhs)
    }
}

// commented out if they are implemented by blanket impl in main `Intersects` trait
symmetric_intersects_impl!(Coord<T>, MonotoneChainSegment<'a, T>);
// symmetric_intersects_impl!(Point<T>,MonotoneChainSegment<'a, T>);
// symmetric_intersects_impl!(MultiPoint<T>,MonotoneChainSegment<'a, T>);

symmetric_intersects_impl!(Line<T>, MonotoneChainSegment<'a, T>);
symmetric_intersects_impl!(LineString<T>, MonotoneChainSegment<'a, T>);
// symmetric_intersects_impl!(MultiLineString<T>,MonotoneChainSegment<'a, T>);

symmetric_intersects_impl!(Polygon<T>, MonotoneChainSegment<'a, T>);
// symmetric_intersects_impl!(MultiPolygon<T>,MonotoneChainSegment<'a, T>);
symmetric_intersects_impl!(Rect<T>, MonotoneChainSegment<'a, T>);
symmetric_intersects_impl!(Triangle<T>, MonotoneChainSegment<'a, T>);

// symmetric_intersects_impl!(Geometry<T>,MonotoneChainSegment<'a, T>);
// symmetric_intersects_impl!(GeometryCollection<T>,MonotoneChainSegment<'a, T>);

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Convert;
    use crate::wkt;

    #[test]
    fn test_intersects_edgecase() {
        let pt: Point<f64> = wkt! {POINT(0 0)}.convert();

        let ls0: LineString<f64> = LineString::empty();
        let ls1: LineString<f64> = wkt! {LINESTRING(0 0)}.convert();
        let ls2: LineString<f64> = wkt! {LINESTRING(0 0,1 1)}.convert();

        let mcs0: MonotoneChain<f64> = (&ls0).into();
        let mcs1: MonotoneChain<f64> = (&ls1).into();
        let mcs2: MonotoneChain<f64> = (&ls2).into();

        assert_eq!(ls0.intersects(&pt), mcs0.intersects(&pt));
        assert_eq!(ls1.intersects(&pt), mcs1.intersects(&pt));
        assert_eq!(ls2.intersects(&pt), mcs2.intersects(&pt));
    }

    #[test]
    fn test_exhaustive_compile_test() {
         use geo_types::*;

         // data
        let c = Coord { x: 0., y: 0. };
        let pt: Point = Point::new(0., 0.);
        let ls = line_string![(0., 0.).into(), (1., 1.).into()];
        let multi_ls = MultiLineString::new(vec![ls.clone()]);
        let ln: Line = Line::new((0., 0.), (1., 1.));

        let poly = Polygon::new(LineString::from(vec![(0., 0.), (1., 1.), (1., 0.)]), vec![]);
        let rect = Rect::new(coord! { x: 10., y: 20. }, coord! { x: 30., y: 10. });
        let tri = Triangle::new(
            coord! { x: 0., y: 0. },
            coord! { x: 10., y: 20. },
            coord! { x: 20., y: -10. },
        );
        let geom = Geometry::Point(pt);
        let gc = GeometryCollection::new_from(vec![geom.clone()]);
        let multi_point = MultiPoint::new(vec![pt]);
        let multi_poly = MultiPolygon::new(vec![poly.clone()]);

        let mc:MonotoneChain<f64> = (&ls).into();

         // forwards
        let _ = mc.intersects(&c);
        let _ = mc.intersects(&pt);
        let _ = mc.intersects(&multi_point);

        let _ = mc.intersects(&ln);
        let _ = mc.intersects(&ls);
        let _ = mc.intersects(&multi_ls);

        let _ = mc.intersects(&poly);
        let _ = mc.intersects(&multi_poly);
        let _ = mc.intersects(&rect);
        let _ = mc.intersects(&tri);

        let _ = mc.intersects(&geom);
        let _ = mc.intersects(&gc);
        
        // backwards
        let _ = c.intersects(&mc);
        let _ = pt.intersects(&mc);
        let _ = multi_point.intersects(&mc);

        let _ = ln.intersects(&mc);
        let _ = ls.intersects(&mc);
        let _ = multi_ls.intersects(&mc);

        let _ = poly.intersects(&mc);
        let _ = multi_poly.intersects(&mc);
        let _ = rect.intersects(&mc);
        let _ = tri.intersects(&mc);

        let _ = geom.intersects(&mc);
        let _ = gc.intersects(&mc);

    }
}
